use rodio::{
    cpal::{
        self,
        traits::{DeviceTrait, HostTrait},
        DeviceNameError,
    },
    decoder::DecoderError,
    DevicesError, OutputStream, OutputStreamHandle, PlayError, Sink, StreamError,
};
use std::{
    fmt::Display,
    sync::mpsc::{self, Receiver, Sender, TryRecvError},
    thread,
    time::Duration,
};
use thiserror::Error;
use tokio::time;

use crate::{
    config::Station,
    decoder::{Mp3Error, Mp3StreamDecoder},
};

pub enum PlayerMsg {
    Stop,
    SetVolume(u8),
}

const STREAM_CONNECT_TIMEOUT_SECS: u8 = 5;

pub struct Player {
    player_tx: Sender<PlayerMsg>,
    player_rx: Option<Receiver<PlayerMsg>>,
    stopped_tx: Option<Sender<()>>,
    stopped_rx: Receiver<()>,
    curr_station: Option<Station>,
    volume_percent: u8,
    alsa_device_idx: usize,
}

#[derive(Error, Debug)]
pub enum Error {
    RodioPlay(PlayError),
    RodioStream(StreamError),
    RodioDevices(DevicesError),
    RodioDecode(DecoderError),
    CPALDeviceName(DeviceNameError),

    Mp3(Mp3Error),
    Reqwest(reqwest::Error),
    NoSuchDevice,
    NoDefaultAudioDevice,

    NotPlaying,
    StreamConnectTimeout(u8),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::RodioPlay(err) => write!(f, "{err}"),
            Error::RodioStream(err) => write!(f, "{err}"),
            Error::RodioDecode(err) => write!(f, "{err}"),
            Error::RodioDevices(err) => write!(f, "{err}"),
            Error::CPALDeviceName(err) => write!(f, "{err}"),
            Error::Reqwest(err) => write!(f, "{err}"),
            Error::NoSuchDevice => write!(f, "this device does not exist"),
            Error::NoDefaultAudioDevice => write!(f, "no default audio device could be identified"),
            Error::Mp3(err) => write!(f, "mp3 decode error: {err}"),
            Error::NotPlaying => write!(f, "the player is currently not playing anything"),
            Error::StreamConnectTimeout(secs) => {
                write!(f, "stream did not connect after {secs} second timeout")
            }
        }
    }
}

impl From<PlayError> for Error {
    fn from(err: PlayError) -> Self {
        Self::RodioPlay(err)
    }
}

impl From<StreamError> for Error {
    fn from(err: StreamError) -> Self {
        Self::RodioStream(err)
    }
}

impl From<DevicesError> for Error {
    fn from(err: DevicesError) -> Self {
        Self::RodioDevices(err)
    }
}

impl From<DecoderError> for Error {
    fn from(err: DecoderError) -> Self {
        Self::RodioDecode(err)
    }
}

impl From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Self {
        Self::Reqwest(err)
    }
}

impl From<Mp3Error> for Error {
    fn from(err: Mp3Error) -> Self {
        Self::Mp3(err)
    }
}

impl From<DeviceNameError> for Error {
    fn from(err: DeviceNameError) -> Self {
        Self::CPALDeviceName(err)
    }
}

pub(crate) fn list_host_devices() -> Result<Vec<String>, DeviceNameError> {
    let host = cpal::default_host();
    let devices = host.output_devices().unwrap();
    devices.into_iter().map(|dev| dev.name()).collect()
}

pub(crate) fn default_device_idx() -> Result<usize, Error> {
    let host = cpal::default_host();
    let devices = host.output_devices()?;
    let Some(default_device) = host.default_output_device() else {
            return Err(Error::NoDefaultAudioDevice);
    };
    for (idx, device) in devices.into_iter().enumerate() {
        if device.name()? == default_device.name()? {
            return Ok(idx);
        }
    }
    unreachable!("the audio devices include the default audio device")
}

fn output_stream_by_device_idx(idx: usize) -> Result<(OutputStream, OutputStreamHandle), Error> {
    let host = cpal::default_host();
    let devices = host.output_devices().unwrap();
    match devices.into_iter().nth(idx) {
        Some(device) => Ok(OutputStream::try_from_device(&device)?),
        None => Err(Error::NoSuchDevice),
    }
}

fn create_sink(
    url: String,
    default_volume: u8,
    device_idx: usize,
) -> Result<(Sink, OutputStream), Error> {
    let stream = reqwest::blocking::get(url)?;

    let source = Mp3StreamDecoder::new(stream)?;
    let (_stream, stream_handle) = output_stream_by_device_idx(device_idx)?;

    let sink = rodio::Sink::try_new(&stream_handle)?;
    sink.set_volume(default_volume as f32 / 100.0);
    sink.append(source);

    Ok((sink, _stream))
}

impl Player {
    pub fn new(volume_percent: u8, alsa_device_idx: usize) -> Result<Self, Error> {
        let (terminate_tx, terminate_rx) = mpsc::channel();
        let (stopped_tx, stopped_rx) = mpsc::channel();

        Ok(Self {
            player_tx: terminate_tx,
            player_rx: Some(terminate_rx),
            stopped_tx: Some(stopped_tx),
            stopped_rx,
            curr_station: None,
            volume_percent,
            alsa_device_idx,
        })
    }

    pub fn curr_station_id(&mut self) -> Option<String> {
        match self.stopped_rx.try_recv() {
            Ok(_) => {
                self.stop(false).unwrap();
                None
            }
            Err(TryRecvError::Empty) => self.curr_station.as_ref().map(|s| s.id.clone()),
            Err(TryRecvError::Disconnected) => unreachable!("the sender is never disconnected"),
        }
    }

    pub fn set_volume(&mut self, volume_percent: u8) {
        self.player_tx
            .send(PlayerMsg::SetVolume(volume_percent))
            .unwrap();
        self.volume_percent = volume_percent
    }

    pub async fn set_output_device(&mut self, idx: usize) -> Result<(), Error> {
        self.alsa_device_idx = idx;

        debug!("Restarting player...");
        if let Some(station) = self.curr_station.clone() {
            self.stop(true)?;
            self.play(station.clone()).await?
        };
        debug!("Player restarted successfully");

        Ok(())
    }

    pub async fn play(&mut self, station: Station) -> Result<(), Error> {
        debug!("Attempting to play station `{}`", station.name);

        let player_rx = match self.player_rx.take() {
            Some(rx) => rx,
            None => {
                self.stop(true)?;
                self.player_rx
                    .take()
                    .expect("the player was just stopped, there must be a receiver now")
            }
        };

        let (outcome_tx, outcome_rx) = mpsc::channel();
        let player_stopped_tx = self.stopped_tx.take().unwrap();

        let thread_url = station.url.to_string();
        let player_volume = self.volume_percent;
        let device_idx = self.alsa_device_idx;

        thread::spawn(
            move || match create_sink(thread_url, player_volume, device_idx) {
                Ok((sink, _output_handle)) => {
                    outcome_tx.send(Ok(()))?;
                    loop {
                        if sink.empty() {
                            debug!("Sink is empty, playback has ended: sending signal");
                            player_stopped_tx
                                .send(())
                                .expect("the receiver is always valid");
                            return Ok(());
                        }
                        match player_rx.try_recv() {
                            Ok(PlayerMsg::Stop) | Err(TryRecvError::Disconnected) => {
                                debug!("Player is terminating...");
                                return Ok(());
                            }
                            Ok(PlayerMsg::SetVolume(volume)) => {
                                debug!("Set player volume to {volume}%");
                                sink.set_volume(volume as f32 / 100.0)
                            }
                            Err(TryRecvError::Empty) => thread::sleep(Duration::from_millis(500)),
                        }
                    }
                }
                Err(err) => outcome_tx.send(Err(err)),
            },
        );

        for i in 0..=STREAM_CONNECT_TIMEOUT_SECS * 2 {
            time::sleep(Duration::from_millis(500)).await;

            match outcome_rx.try_recv() {
                Ok(Ok(())) => {
                    info!("Stream connected: playing `{}`", station.url);
                    self.curr_station = Some(station.clone());
                    break;
                }
                Ok(Err(err)) => {
                    self.stop(false)?;
                    return Err(err);
                }
                Err(TryRecvError::Empty) if i == STREAM_CONNECT_TIMEOUT_SECS => {
                    drop(outcome_rx);
                    return Err(Error::StreamConnectTimeout(STREAM_CONNECT_TIMEOUT_SECS));
                }
                Err(TryRecvError::Empty) => {
                    debug!("Waiting for stream connection...")
                }
                Err(TryRecvError::Disconnected) => unreachable!("this cannot happen"),
            }
        }

        Ok(())
    }

    pub fn stop(&mut self, send_signal: bool) -> Result<(), Error> {
        // if `self.rx` is none, there is currently a thread which is playing something
        match self.player_rx.is_none() {
            true => {
                if send_signal {
                    // terminate the player
                    self.player_tx
                        .send(PlayerMsg::Stop)
                        .expect("can always send the termination signal");
                }

                // place a new set of channels into the player
                let (tx, rx) = mpsc::channel();
                self.player_tx = tx;
                self.player_rx = Some(rx);

                let (stopped_tx, stopped_rx) = mpsc::channel();
                self.stopped_tx = Some(stopped_tx);
                self.stopped_rx = stopped_rx;

                // set the current status to `not playing`
                self.curr_station = None;

                Ok(())
            }
            false => Err(Error::NotPlaying),
        }
    }
}
