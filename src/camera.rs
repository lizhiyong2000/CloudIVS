use std::fmt::Write;
use uuid::Uuid;

/// In-memory state about a camera.
#[derive(Debug)]
pub struct Camera {
    pub id: i32,
    pub uuid: Uuid,
    pub short_name: String,
    pub description: String,
    pub onvif_host: String,
    pub username: String,
    pub password: String,
    pub streams: [Option<i32>; 2],
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum StreamType { MAIN, SUB }

impl StreamType {
    pub fn from_index(i: usize) -> Option<Self> {
        match i {
            0 => Some(StreamType::MAIN),
            1 => Some(StreamType::SUB),
            _ => None,
        }
    }

    pub fn index(self) -> usize {
        match self {
            StreamType::MAIN => 0,
            StreamType::SUB => 1,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            StreamType::MAIN => "main",
            StreamType::SUB => "sub",
        }
    }

    pub fn parse(type_: &str) -> Option<Self> {
        match type_ {
            "main" => Some(StreamType::MAIN),
            "sub" => Some(StreamType::SUB),
            _ => None,
        }
    }
}

impl ::std::fmt::Display for StreamType {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> Result<(), ::std::fmt::Error> {
        f.write_str(self.as_str())
    }
}

pub const ALL_STREAM_TYPES: [StreamType; 2] = [StreamType::MAIN, StreamType::SUB];

pub struct Stream {
    pub id: i32,
    pub camera_id: i32,
    pub sample_file_dir_id: Option<i32>,
    pub type_: StreamType,
    pub rtsp_url: String,
    pub retain_bytes: i64,
    pub flush_if_sec: i64,

    pub record: bool,

}