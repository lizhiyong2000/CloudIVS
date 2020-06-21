use crate::h264;
use cstr::*;
use failure::{Error, bail};
use ffmpeg;
use lazy_static::lazy_static;
use log::{debug, info, warn};
use std::ffi::CString;
use std::result::Result;
use crate::camera;

static START: parking_lot::Once = parking_lot::Once::new();

lazy_static! {
    pub static ref FFMPEG: Ffmpeg = Ffmpeg::new();
}

pub enum Source<'a> {
    /// A filename, for testing.
    #[cfg(test)]
    File(&'a str),

    /// An RTSP stream, for production use.
    Rtsp {
        url: &'a str,
        redacted_url: &'a str
    },
}

pub trait Opener<S : Stream> : Sync {
    fn open(&self, src: Source) -> Result<S, Error>;
}

pub trait Stream {
    fn get_extra_data(&self) -> Result<h264::ExtraData, Error>;
    fn get_next<'p>(&'p mut self) -> Result<ffmpeg::Packet<'p>, ffmpeg::Error>;
}

pub struct Ffmpeg {}

impl Ffmpeg {
    fn new() -> Ffmpeg {
        START.call_once(|| {
            ffmpeg::Ffmpeg::new();
            //ffmpeg::init().unwrap();
            //ffmpeg::format::network::init();
        });
        Ffmpeg{}
    }
}

impl Opener<FfmpegStream> for Ffmpeg {
    fn open(&self, src: Source) -> Result<FfmpegStream, Error> {
        use ffmpeg::InputFormatContext;
        let (mut input, discard_first) = match src {
            #[cfg(test)]
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
                open_options.set(cstr!("user-agent"), cstr!("cloudivs-server")).unwrap();
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

        let mut stream = FfmpegStream{
            input,
            video_i,
        };

        if discard_first {
            info!("Discarding the first packet to work around https://trac.ffmpeg.org/ticket/5018");
            stream.get_next()?;
        }

        Ok(stream)
    }
}

pub struct FfmpegStream {
    input: ffmpeg::InputFormatContext,
    video_i: usize,
}

impl Stream for FfmpegStream {
    fn get_extra_data(&self) -> Result<h264::ExtraData, Error> {
        let video = self.input.streams().get(self.video_i);
        let tb = video.time_base();
        if tb.num != 1 || tb.den != 90000 {
            bail!("video stream has timebase {}/{}; expected 1/90000", tb.num, tb.den);
        }
        let codec = video.codecpar();
        let codec_id = codec.codec_id();
        if !codec_id.is_h264() {
            bail!("stream's video codec {:?} is not h264", codec_id);
        }
        h264::ExtraData::parse(codec.extradata(), codec.width() as u16, codec.height() as u16)
    }

    fn get_next<'i>(&'i mut self) -> Result<ffmpeg::Packet<'i>, ffmpeg::Error> {
        loop {
            let p = self.input.read_frame()?;
            if p.stream_index() == self.video_i {
                return Ok(p);
            }
        }
    }
}
