// Initial version from Rodio APACHE LICENSE 2.0
use crate::sound::source::Source;
use log::{error, info, trace, warn};
use minimp3::{Decoder, Frame};
use std::io::{Read, Seek};
use std::time::Duration;

pub struct Mp3Decoder<R>
where
    R: Read + Seek,
{
    decoder: Decoder<R>,
    current_frame: Frame,
    current_frame_offset: usize,
}

impl<R> Mp3Decoder<R>
where
    R: Read + Seek,
{
    pub fn new(data: R) -> Result<Self, ()> {
        let mut decoder = Decoder::new(data);
        let current_frame = decoder.next_frame().map_err(|_| ())?;

        Ok(Mp3Decoder {
            decoder,
            current_frame,
            current_frame_offset: 0,
        })
    }

    pub fn total_duration_mut<T>(&self, reader: &mut T) -> Option<Duration>
    where
        T: std::io::Read,
    {
        let duration = mp3_duration::from_read(reader);
        if let Ok(duration) = duration {
            Some(duration)
        } else {
            trace!("Could not read mp3 tag {:?}", duration.err());
            None
        }
    }
}

impl<R> Source for Mp3Decoder<R>
where
    R: Read + Seek,
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

impl<R> Iterator for Mp3Decoder<R>
where
    R: Read + Seek,
{
    type Item = i16;

    #[inline]
    fn next(&mut self) -> Option<i16> {
        if self.current_frame_offset == self.current_frame.data.len() {
            self.current_frame_offset = 0;
            match self.decoder.next_frame() {
                Ok(frame) => self.current_frame = frame,
                _ => return None,
            }
        }

        let v = self.current_frame.data[self.current_frame_offset];
        self.current_frame_offset += 1;

        Some(v)
    }
}
