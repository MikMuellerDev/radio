use bytes::{BufMut, BytesMut};
use core::time;
use futures_util::StreamExt;
use rodio::{decoder::DecoderError, DevicesError, OutputStream, PlayError, Sink, StreamError};
use std::{fmt::Display, io::Cursor, time::SystemTime};

pub struct Player;

#[derive(Debug)]
pub enum Error {
    RodioPlay(PlayError),
    RodioStream(StreamError),
    RodioDevices(DevicesError),
    RodioDecode(DecoderError),

    Reqwest(reqwest::Error),
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

impl Player {
    pub fn new() -> Result<Self, Error> {
        Ok(Self {})
    }

    pub async fn play(&self, url: &str) -> Result<(), Error> {
        let chunk_size = 64_000;

        let (_stream, stream_handle) = OutputStream::try_default()?;
        let sink = Sink::try_new(&stream_handle).unwrap();

        // Open a network stream which uses chunked encoding
        let mut stream = reqwest::get(url).await?.bytes_stream();
        // Create a rolling playback buffer which contains multiple chunks of the stream
        let mut buffer: BytesMut = BytesMut::with_capacity(chunk_size);

        while let Some(current_chunk) = stream.next().await {
            println!("{}: {:?}", sink.len(), SystemTime::now());
            if sink.len() == 0 {
                eprintln!("WARNING: no cached chunks");
            }

            // Check if the buffer has reached the chunk_size
            if buffer.len() < chunk_size {
                // The buffer has not already reached its desired size
                // Append the current stream chunk to the buffer
                buffer.put(current_chunk?);
                println!("Filling buffer...");
                // Buffer is not full, omit appending it to the sink queue
                continue;
            }

            println!("Buffer full");

            // Play the current buffer because it is ready
            let cursor = Cursor::new(buffer);
            let source = rodio::Decoder::new(cursor)?;
            sink.append(source);

            // Flush the current buffer after it has been transformed into a cursor
            buffer = BytesMut::with_capacity(chunk_size);
            // Append the current chunk in order to avoid an audio gap in the next buffer
            buffer.put(current_chunk?)
        }
        Ok(sink.sleep_until_end())
    }
}
