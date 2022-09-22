use std::{io::{BufRead, BufReader, Read}, io, process::{Command, Stdio}};
use std::io::{Cursor, ErrorKind, Seek, SeekFrom};
use std::process::{Child, ChildStdout};
use std::sync::{Arc, Mutex};

use byteorder::{LittleEndian, WriteBytesExt};
use serde_json::Value;
use songbird::driver::opus::{Channels, SampleRate};
use songbird::driver::opus::coder::Decoder;
use songbird::input::{Codec, Container, Input, Metadata, Reader};
use songbird::input::error::{Error, Result};
use symphonia_core::codecs::CODEC_TYPE_NULL;
use symphonia_core::formats::{FormatOptions, FormatReader};
use symphonia_core::io::{MediaSource, MediaSourceStream};
use symphonia_core::meta::MetadataOptions;
use symphonia_core::probe::Hint;
use tokio::task;

pub async fn ytdl_native(uri: impl AsRef<str>) -> Result<Input> {
    _ytdl(uri.as_ref()).await
}

pub(crate) async fn _ytdl(uri: &str) -> Result<Input> {
    let ytdl_args = [
        "--print-json",
        "-f",
        "webm[abr>0]/bestaudio/best",
        "-R",
        "infinite",
        "--no-playlist",
        "--ignore-config",
        "--no-warnings",
        uri,
        "-o",
        "-",
    ];

    let mut youtube_dl = Command::new("yt-dlp")
        .args(&ytdl_args)
        .stdin(Stdio::null())
        .stderr(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;

    // This rigmarole is required due to the inner synchronous reading context.
    let stderr = youtube_dl.stderr.take();
    let (returned_stderr, json) = task::spawn_blocking(move || {
        let mut s = stderr.unwrap();
        let out: Result<Value> = {
            let mut o_vec = vec![];
            let mut serde_read = BufReader::new(s.by_ref());
            // Newline...
            if let Ok(len) = serde_read.read_until(0xA, &mut o_vec) {
                serde_json::from_slice(&o_vec[..len]).map_err(|err| Error::Json {
                    error: err,
                    parsed_text: std::str::from_utf8(&o_vec).unwrap_or_default().to_string(),
                })
            } else {
                Err(Error::Metadata)
            }
        };

        (s, out)
    })
        .await
        .map_err(|_| Error::Metadata)?;

    youtube_dl.stderr = Some(returned_stderr);

    let buf = BufReader::with_capacity(64*1024, youtube_dl.stdout.take().unwrap());
    let mss = MediaSourceStream::new(Box::new(PipeWrapper{inner: buf}), Default::default());
    let mut hint = Hint::new();
    hint.with_extension("webm");

    let meta_opts: MetadataOptions = Default::default();
    let fmt_opts: FormatOptions = Default::default();
    let probed = symphonia::default::get_probe()
        .format(&hint, mss, &fmt_opts, &meta_opts)
        .expect("unsupported format");

    let format = probed.format;
    let dms = Box::new(DecodingMediaSource::new(youtube_dl, format));

    Ok(Input::new(
        true,
        Reader::Extension(dms),
        Codec::FloatPcm,
        Container::Raw,
        Some(Metadata::from_ytdl_output(json?)),
    ))
}

struct DecodingMediaSource {
    child: Child,
    format: Box<dyn FormatReader>,
    track_id: u32,
    decoder: Arc<Mutex<Decoder>>,
    cursor: Cursor<[u8; 7680]>
}

impl DecodingMediaSource {
    pub fn new(child: Child, format: Box<dyn FormatReader>) -> Self {
        let track_id = format
            .tracks()
            .iter()
            .find(|t| t.codec_params.codec != CODEC_TYPE_NULL)
            .expect("no supported audio tracks")
            .id;

        DecodingMediaSource {
            child,
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

impl Read for DecodingMediaSource {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if self.cursor.position() == 7680 {
            self.decode_next_packet()?;
        }
        Read::read(&mut self.cursor, buf)
    }
}

impl Seek for DecodingMediaSource {
    fn seek(&mut self, _: SeekFrom) -> io::Result<u64> {
        panic!("Seeking unsupported")
    }
}

impl MediaSource for DecodingMediaSource {
    fn is_seekable(&self) -> bool {
        false
    }

    fn byte_len(&self) -> Option<u64> {
        None
    }
}

impl Drop for DecodingMediaSource {
    fn drop(&mut self) {
        self.child.kill().unwrap();
    }
}

struct PipeWrapper {
    inner: BufReader<ChildStdout>
}

impl Read for PipeWrapper {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.inner.read(buf)
    }
}

impl Seek for PipeWrapper {
    fn seek(&mut self, _: SeekFrom) -> io::Result<u64> {
        panic!()
    }
}

impl MediaSource for PipeWrapper {
    fn is_seekable(&self) -> bool {
        false
    }

    fn byte_len(&self) -> Option<u64> {
        None
    }
}