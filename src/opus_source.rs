use std::io::{Cursor, ErrorKind, Read, Seek, SeekFrom};
use std::io;
use std::process::Child;
use std::sync::{Arc, Mutex};

use byteorder::{LittleEndian, WriteBytesExt};
use songbird::driver::opus::{Channels, SampleRate};
use songbird::driver::opus::coder::Decoder;
use symphonia_core::codecs::CODEC_TYPE_NULL;
use symphonia_core::formats::FormatReader;
use symphonia_core::io::MediaSource;

pub struct OpusStreamSource {
    process: Child,
    format: Box<dyn FormatReader>,
    track_id: u32,
    decoder: Arc<Mutex<Decoder>>,
    cursor: Cursor<[u8; 7680]>
}

impl OpusStreamSource {
    pub fn new(process: Child, format: Box<dyn FormatReader>) -> Self {
        let track_id = format
            .tracks()
            .iter()
            .find(|t| t.codec_params.codec != CODEC_TYPE_NULL)
            .expect("no supported audio tracks")
            .id;

        OpusStreamSource {
            process,
            format,
            track_id,
            decoder: Arc::new(Mutex::new(Decoder::new(SampleRate::Hz48000, Channels::Stereo).unwrap())),
            cursor: Cursor::new([0u8; 7680])
        }
    }

    fn decode_next_packet(&mut self) -> io::Result<()> {
        loop {
            let packet = match self.format.next_packet() {
                Ok(packet) => packet,
                Err(symphonia::core::errors::Error::IoError(io_err)) => return Err(io_err),
                Err(err) => return Err(io::Error::new(ErrorKind::Other, err))
            };

            while !self.format.metadata().is_latest() {
                self.format.metadata().pop();
            }

            if packet.track_id() != self.track_id {
                continue;
            }

            let mut pcm = [0f32; 1920];
            self.decoder.lock().unwrap().decode_float(Some(packet.buf().try_into().unwrap()), (&mut pcm[..]).try_into().unwrap(), false).unwrap();

            self.cursor.set_position(0);
            for &sample in pcm.iter() {
                self.cursor.write_f32::<LittleEndian>(sample)?;
            }
            self.cursor.set_position(0);

            return Ok(())
        }
    }
}

impl Read for OpusStreamSource {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if self.cursor.position() == 7680 {
            self.decode_next_packet()?;
        }
        Read::read(&mut self.cursor, buf)
    }
}

impl Seek for OpusStreamSource {
    fn seek(&mut self, _: SeekFrom) -> io::Result<u64> {
        panic!("Seeking unsupported")
    }
}

impl Drop for OpusStreamSource {
    fn drop(&mut self) {
        self.process.kill().unwrap()
    }
}

impl MediaSource for OpusStreamSource {
    fn is_seekable(&self) -> bool {
        false
    }

    fn byte_len(&self) -> Option<u64> {
        None
    }
}
