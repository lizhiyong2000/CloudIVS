mod packet;

mod send_report_packet;
//
// pub use error::{Error, ErrorKind};
// use std::error::Error;

// pub use traits::*;
// pub use types::*;
// pub use error::*;


mod error{
    use std::io;
    use trackable::error::{ErrorKind as TrackableErrorKind, ErrorKindExt};
    use trackable::error::{IntoTrackableError, TrackableError};

    pub type Error = TrackableError<ErrorKind>;

    #[derive(Debug, Clone)]
    pub enum ErrorKind {
        Unsupported,
        Invalid,
        Other,
    }
    impl TrackableErrorKind for ErrorKind {}
    impl IntoTrackableError<io::Error> for ErrorKind {
        fn into_trackable_error(from: io::Error) -> Error {
            ErrorKind::Other.cause(from)
        }
    }
}

// pub use error::{Error, ErrorKind};


mod constants{

    pub const RTP_VERSION: u8 = 2;

    pub const RTCP_PACKET_TYPE_SR: u8 = 200;
    pub const RTCP_PACKET_TYPE_RR: u8 = 201;
    pub const RTCP_PACKET_TYPE_SDES: u8 = 202;
    pub const RTCP_PACKET_TYPE_BYE: u8 = 203;
    pub const RTCP_PACKET_TYPE_APP: u8 = 204;
    pub const RTCP_PACKET_TYPE_RTPFB: u8 = 205;
    pub const RTCP_PACKET_TYPE_PSFB: u8 = 206;
    pub const RTCP_PACKET_TYPE_XR: u8 = 207;

    pub const SDES_ITEM_TYPE_END: u8 = 0;
    pub const SDES_ITEM_TYPE_CNAME: u8 = 1;
    pub const SDES_ITEM_TYPE_NAME: u8 = 2;
    pub const SDES_ITEM_TYPE_EMAIL: u8 = 3;
    pub const SDES_ITEM_TYPE_PHONE: u8 = 4;
    pub const SDES_ITEM_TYPE_LOC: u8 = 5;
    pub const SDES_ITEM_TYPE_TOOL: u8 = 6;
    pub const SDES_ITEM_TYPE_NOTE: u8 = 7;
    pub const SDES_ITEM_TYPE_PRIV: u8 = 8;
}


mod types {

    pub use crate::protocol::rtcp::error::{Error, ErrorKind};

    pub type U2 = u8;
    pub type U4 = u8;
    pub type U5 = u8;
    pub type U6 = u8;
    pub type U7 = u8;
    pub type U13 = u16;
    pub type U24 = u32;
    pub type U48 = u64;
    pub type RtpTimestamp = u32;
    pub type NtpTimestamp = u64;
    pub type NtpMiddleTimetamp = u32;
    pub type Ssrc = u32;
    pub type Csrc = u32;
    pub type SsrcOrCsrc = u32;

    pub type Result<T> = ::std::result::Result<T, Error>;
}


mod traits{
    use std::io::{Read, Write};

    use super::types::Result;

    pub trait PacketTrait {}

    pub trait RtpPacketTrait: PacketTrait {}
    pub trait RtcpPacketTrait: PacketTrait {}

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
            track_try!(self.write_to(&mut buf));
            Ok(buf)
        }
    }
}