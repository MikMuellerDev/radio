use minimp3::{Decoder, Frame};
use std::fmt::Display;
use std::io::Read;
use std::time::Duration;

use rodio::Source;

#[derive(Debug)]
pub enum Mp3Error {
    //Eof,
    NotMp3,
}

impl Display for Mp3Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Mp3Error::NotMp3 => "source data is not valid MP3",
            }
        )
    }
}

/// This is a modified version of [rodio's Mp3Decoder](https://github.com/RustAudio/rodio/blob/55d957f8b40c59fccea4162c4b03f6dd87a7a4d9/src/decoder/mp3.rs)
/// which removes the "Seek" trait bound for streaming network audio.
///
/// Related GitHub issue:
/// https://github.com/RustAudio/rodio/issues/333
pub struct Mp3StreamDecoder<R>
where
    R: Read,
{
    decoder: Decoder<R>,
    current_frame: Frame,
    current_frame_offset: usize,
}

impl<R> Mp3StreamDecoder<R>
where
    R: Read,
{
    pub fn new(mut data: R) -> Result<Self, Mp3Error> {
        match is_mp3(data.by_ref()) {
            Ok(current_frame) => {
                debug!("Stream is valid MP3, starting decoder");
                let decoder = Decoder::new(data);

                Ok(Self {
                    decoder,
                    current_frame,
                    current_frame_offset: 0,
                })
            }
            Err(_) => {
                debug!("Stream is not MP3, cannot decode");
                Err(Mp3Error::NotMp3)
            }
        }
    }
}

impl<R> Source for Mp3StreamDecoder<R>
where
    R: Read,
{
    #[inline]
    fn current_frame_len(&self) -> Option<usize> {
        Some(self.current_frame.data.len())
    }

    #[inline]
    fn channels(&self) -> u16 {
        self.current_frame.channels as _
    }

    #[inline]
    fn sample_rate(&self) -> u32 {
        self.current_frame.sample_rate as _
    }

    #[inline]
    fn total_duration(&self) -> Option<Duration> {
        None
    }
}

impl<R> Iterator for Mp3StreamDecoder<R>
where
    R: Read,
{
    type Item = i16;

    #[inline]
    fn next(&mut self) -> Option<i16> {
        if self.current_frame_offset == self.current_frame.data.len() {
            match self.decoder.next_frame() {
                Ok(frame) => self.current_frame = frame,
                _ => return None,
            }
            self.current_frame_offset = 0;
        }

        let v = self.current_frame.data[self.current_frame_offset];
        self.current_frame_offset += 1;

        Some(v)
    }
}

fn is_mp3<R>(mut data: R) -> Result<Frame, ()>
where
    R: Read,
{
    let mut decoder = Decoder::new(data.by_ref());
    let curr_frame = decoder.next_frame();
    match curr_frame {
        Ok(frame) => Ok(frame),
        Err(_) => Err(()),
    }
}
