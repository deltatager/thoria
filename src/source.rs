use std::io::{BufRead, BufReader, Read};
use std::process::{Command, Stdio};

use serde_json::Value;
use songbird::input::{Codec, Container, Input, Metadata, Reader};
use songbird::input::error::{Error, Result};
use symphonia_core::formats::FormatOptions;
use symphonia_core::io::{MediaSourceStream, ReadOnlySource};
use symphonia_core::meta::MetadataOptions;
use symphonia_core::probe::Hint;
use tokio::task;

use crate::opus_source::OpusStreamSource;

pub async fn ytdl_native(uri: impl AsRef<str>) -> Result<Input> {
    let ytdl_args = [
        "--print-json",
        "-f",
        "webm[abr>0]/bestaudio/best",
        "-R",
        "infinite",
        "--no-playlist",
        "--ignore-config",
        "--no-warnings",
        uri.as_ref(),
        "-o",
        "-",
    ];

    let mut youtube_dl = Command::new("yt-dlp")
        .args(&ytdl_args)
        .stdin(Stdio::null())
        .stderr(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;

    let stderr = youtube_dl.stderr.take();
    let (returned_stderr, json) = task::spawn_blocking(move || {
        let mut s = stderr.unwrap();
        let out: Result<Value> = {
            let mut o_vec = vec![];
            let mut serde_read = BufReader::new(s.by_ref());

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

    let mss = MediaSourceStream::new(
        Box::new(ReadOnlySource::new(youtube_dl.stdout.take().unwrap())),
        Default::default()
    );

    let mut hint = Hint::new();
    hint.with_extension("webm");

    let meta_opts: MetadataOptions = Default::default();
    let fmt_opts: FormatOptions = Default::default();
    let probed = symphonia::default::get_probe()
        .format(&hint, mss, &fmt_opts, &meta_opts)
        .expect("unsupported format");

    let source = Box::new(OpusStreamSource::new(youtube_dl, probed.format));

    Ok(Input::new(
        true,
        Reader::Extension(source),
        Codec::FloatPcm,
        Container::Raw,
        Some(Metadata::from_ytdl_output(json?)),
    ))
}