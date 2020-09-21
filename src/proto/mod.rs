pub mod common;
pub mod rtsp;
pub mod sip;
pub mod sdp;
pub mod rtcp;
pub mod rtp;



pub mod types {


    pub type U2 = u8;
    pub type U4 = u8;
    pub type U5 = u8;
    pub type U6 = u8;
    pub type U7 = u8;
    pub type U13 = u16;
    pub type U24 = u32;
    pub type U48 = u64;
    // pub type RtpTimestamp = u32;
    // pub type NtpTimestamp = u64;
    // pub type NtpMiddleTimetamp = u32;
    // pub type Ssrc = u32;
    // pub type Csrc = u32;
    // pub type SsrcOrCsrc = u32;

    // pub type Result<T> = ::std::result::Result<T, Error>;
}

pub mod error{
    use std::io;

    use trackable::error::{ErrorKind as TrackableErrorKind, ErrorKindExt, Failure};
    use trackable::error::{TrackableError};

    #[derive(Debug, Clone, TrackableError)]
    pub struct Error(TrackableError<ErrorKind>);
    impl From<Failure> for Error {
        fn from(f: Failure) -> Self {
            ErrorKind::Other.takes_over(f).into()
        }
    }
    impl From<std::io::Error> for Error {
        fn from(f: std::io::Error) -> Self {
            ErrorKind::Other.cause(f).into()
        }
    }


    #[derive(Debug, Clone)]
    pub enum ErrorKind {
        Unsupported,
        Invalid,
        Other
    }
    impl TrackableErrorKind for ErrorKind {}

}

pub mod traits{
    use std::io::{Read, Write};

    use super::error::{Error, ErrorKind};

    pub type Result<T> = ::std::result::Result<T, Error>;


    pub trait PacketTrait {}


    // TODO: DecodePacket(?)
    pub trait ReadPacket {
        type Packet: PacketTrait;
        fn read_packet<R: Read>(&mut self, reader: &mut R) -> Result<Self::Packet>;
        fn supports_type(&self, packet_type: u8) -> bool;
    }

    pub trait WritePacket {
        type Packet: PacketTrait;
        fn write_packet<W: Write>(&mut self, writer: &mut W, packet: &Self::Packet) -> Result<()>;
    }

    pub trait PacketData {
        fn to_bytes(&self) -> Vec<u8>;
    }


    pub trait ReadFrom: Sized {
        fn read_from<R: Read>(reader: &mut R) -> Result<Self>;
    }

    pub trait WriteTo {
        fn write_to<W: Write>(&self, writer: &mut W) -> Result<()>;
        fn to_bytes(&self) -> Result<Vec<u8>> {
            let mut buf = Vec::new();
            track!(self.write_to(&mut buf).map_err(Error::from));

            Ok(buf)
        }
    }

}