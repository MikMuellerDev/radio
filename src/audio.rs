use rodio::{decoder::DecoderError, DevicesError, OutputStream, PlayError, StreamError};
use thiserror::Error;
use std::{
    fmt::Display,
    sync::mpsc::{self, Receiver, Sender, TryRecvError},
    thread,
    time::Duration,
};

use crate::decoder::{Mp3Error, Mp3StreamDecoder};

pub struct Player {
    tx: Sender<()>,
    rx: Option<Receiver<()>>,
}

#[derive(Error, Debug)]
pub enum Error {
    RodioPlay(PlayError),
    RodioStream(StreamError),
    RodioDevices(DevicesError),
    RodioDecode(DecoderError),

    Mp3(Mp3Error),
    Reqwest(reqwest::Error),

    NotPlaying,
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Error::RodioPlay(err) => err.to_string(),
                Error::RodioStream(err) => err.to_string(),
                Error::RodioDevices(err) => err.to_string(),
                Error::RodioDecode(err) => err.to_string(),
                Error::Reqwest(err) => err.to_string(),
                Error::Mp3(err) => format!("mp3 decode error: {err}"),
                Error::NotPlaying => "the player is currently not playing anything".to_string(),
            }
        )
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

impl Player {
    pub fn new() -> Result<Self, Error> {
        let (tx, rx) = mpsc::channel();
        Ok(Self { tx, rx: Some(rx) })
    }

    pub async fn play(&mut self, url: String) -> Result<(), Error> {
        debug!("Playing URL: `{url}`");

        let rx = match self.rx.take() {
            Some(rx) => rx,
            None => {
                self.stop()?;
                self.rx
                    .take()
                    .expect("the player was just stopped, there must be a receiver now")
            }
        };

        thread::spawn(move || -> Result<(), Error> {
            let stream = reqwest::blocking::get(url)?;

            let source = Mp3StreamDecoder::new(stream)?;
            let (_stream, stream_handle) = OutputStream::try_default()?;

            let sink = rodio::Sink::try_new(&stream_handle)?;
            sink.append(source);

            loop {
                match rx.try_recv() {
                    Ok(_) | Err(TryRecvError::Disconnected) => {
                        debug!("player is terminating...");
                        return Ok(());
                    }
                    Err(TryRecvError::Empty) => thread::sleep(Duration::from_millis(500)),
                }
            }
        });

        Ok(())
    }

    pub fn stop(&mut self) -> Result<(), Error> {
        // if `self.rx` is none, there is crrently a thread which is playing something
        match self.rx.is_none() {
            true => {
                let _ = self.tx.send(());

                // place a new set of channels into the player
                let (tx, rx) = mpsc::channel();
                self.tx = tx;
                self.rx = Some(rx);

                Ok(())
            }
            false => Err(Error::NotPlaying),
        }
    }
}
