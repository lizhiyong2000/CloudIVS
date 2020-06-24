// mod h264;
// mod stream;
// mod streamer;
// mod camera;
#[allow(dead_code)]

extern crate ffmpeg;
// use ffmpeg;
// use std::error::Error;
use std::ffi::CString;
use std::result::Result;
use log::{debug, info, warn};
use failure::{Error, Fail, bail};
use cstr::*;

pub enum Source<'a> {
    /// A filename, for testing.
    // #[cfg(test)]
    File(&'a str),

    /// An RTSP stream, for production use.
    Rtsp {
        url: &'a str,
        redacted_url: &'a str
    },
}

fn open(src: Source) -> Result<(), Error> {
    use ffmpeg::InputFormatContext;
    let (mut input, discard_first) = match src {
        // #[cfg(test)]
        Source::File(filename) => {
            let mut open_options = ffmpeg::Dictionary::new();

            // Work around https://github.com/scottlamb/moonfire-nvr/issues/10
            open_options.set(cstr!("advanced_editlist"), cstr!("false")).unwrap();
            let url = format!("file:{}", filename);
            let i = InputFormatContext::open(&CString::new(url.clone()).unwrap(),
                                             &mut open_options)?;
            if !open_options.empty() {
                warn!("While opening URL {}, some options were not understood: {}",
                      url, open_options);
            }
            (i, false)
        }
        Source::Rtsp{url, redacted_url} => {
            let mut open_options = ffmpeg::Dictionary::new();
            open_options.set(cstr!("rtsp_transport"), cstr!("tcp")).unwrap();
            open_options.set(cstr!("user-agent"), cstr!("moonfire-nvr")).unwrap();
            // 10-second socket timeout, in microseconds.
            open_options.set(cstr!("stimeout"), cstr!("10000000")).unwrap();

            // Moonfire NVR currently only supports video, so receiving audio is wasteful.
            // It also triggers <https://github.com/scottlamb/moonfire-nvr/issues/36>.
            open_options.set(cstr!("allowed_media_types"), cstr!("video")).unwrap();

            let i = InputFormatContext::open(&CString::new(url).unwrap(), &mut open_options)?;
            if !open_options.empty() {
                warn!("While opening URL {}, some options were not understood: {}",
                      redacted_url, open_options);
            }
            (i, true)
        },
    };

    input.find_stream_info()?;

    // Find the video stream.
    let mut video_i = None;
    {
        let s = input.streams();
        for i in 0 .. s.len() {
            if s.get(i).codecpar().codec_type().is_video() {
                debug!("Video stream index is {}", i);
                video_i = Some(i);
                break;
            }
        }
    }
    let video_i = match video_i {
        Some(i) => i,
        None => bail!("no video stream"),
    };

    print!("video_i:{}", video_i);

    // let mut stream = FfmpegStream{
    //     input,
    //     video_i,
    // };
    //
    // if discard_first {
    //     info!("Discarding the first packet to work around https://trac.ffmpeg.org/ticket/5018");
    //     stream.get_next()?;
    // }

    Ok(())
}

fn main() {
    println!("Hello, world!");

    open(Source::File("testdata/clip.mp4"));
}
