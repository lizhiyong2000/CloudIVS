

#[macro_use]
pub mod sip;

pub mod sdp;

pub mod rtp;

pub mod rtsp;




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
            track_try!(self.write_to(&mut buf));
            Ok(buf)
        }
    }

}


// pub use self::rtcp::{ReceptionReport, RtcpCompoundPacket, SdesChunk, SdesItem};
// pub use self::rtcp::{RtcpApplicationDefined, RtcpGoodbye, RtcpSourceDescription};
// pub use self::rtcp::{RtcpPacket, RtcpPacketReader, RtcpReceiverReport, RtcpSenderReport};
//
// pub use self::rtcp::{RTCP_PACKET_TYPE_APP, RTCP_PACKET_TYPE_BYE};
// pub use self::rtcp::{RTCP_PACKET_TYPE_RR, RTCP_PACKET_TYPE_SDES, RTCP_PACKET_TYPE_SR};
// pub use self::rtcp::{SDES_ITEM_TYPE_CNAME, SDES_ITEM_TYPE_END, SDES_ITEM_TYPE_NAME};
// pub use self::rtcp::{SDES_ITEM_TYPE_EMAIL, SDES_ITEM_TYPE_LOC, SDES_ITEM_TYPE_PHONE};
// pub use self::rtcp::{SDES_ITEM_TYPE_NOTE, SDES_ITEM_TYPE_PRIV, SDES_ITEM_TYPE_TOOL};

// pub use self::rtp::{RtpFixedHeader, RtpHeaderExtension, RtpPacket, RtpPacketReader};

