pub mod rtcp_packet;

pub mod report_packet;
pub mod source_description_packet;
pub mod bye_packet;
pub mod app_defined_packet;
pub mod transport_layer_feedback;
pub mod payload_specific_feedback;

mod feedback;

//
// pub use error::{Error, ErrorKind};
// use std::error::Error;

// pub use traits::*;
// pub use types::*;
// pub use error::*;
// pub use self::rtcp::{ReceptionReport, RtcpCompoundPacket, SdesChunk, SdesItem};
// pub use self::rtcp::{RtcpApplicationDefined, RtcpGoodbye, RtcpSourceDescription};
// pub use self::rtcp::{RtcpPacket, RtcpPacketReader, RtcpReceiverReport, RtcpSenderReport};
//
// pub use self::rtcp::{RTCP_PACKET_TYPE_APP, RTCP_PACKET_TYPE_BYE};
// pub use self::rtcp::{RTCP_PACKET_TYPE_RR, RTCP_PACKET_TYPE_SDES, RTCP_PACKET_TYPE_SR};
// pub use self::rtcp::{SDES_ITEM_TYPE_CNAME, SDES_ITEM_TYPE_END, SDES_ITEM_TYPE_NAME};
// pub use self::rtcp::{SDES_ITEM_TYPE_EMAIL, SDES_ITEM_TYPE_LOC, SDES_ITEM_TYPE_PHONE};
// pub use self::rtcp::{SDES_ITEM_TYPE_NOTE, SDES_ITEM_TYPE_PRIV, SDES_ITEM_TYPE_TOOL};

// mod error{
//     use std::io;
//     use trackable::error::{ErrorKind as TrackableErrorKind, ErrorKindExt};
//     use trackable::error::{IntoTrackableError, TrackableError};
//
//     pub type Error = TrackableError<ErrorKind>;
//
//     #[derive(Debug, Clone)]
//     pub enum ErrorKind {
//         Unsupported,
//         Invalid,
//         Other,
//     }
//     impl TrackableErrorKind for ErrorKind {}
//     impl IntoTrackableError<io::Error> for ErrorKind {
//         fn into_trackable_error(from: io::Error) -> Error {
//             ErrorKind::Other.cause(from)
//         }
//     }
// }

// pub use error::{Error, ErrorKind};


mod constants{



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


    pub const RTPFB_MESSAGE_TYPE_NACK: u8 = 1;

    pub const PSFB_MESSAGE_TYPE_PLI: u8 = 1;
    pub const PSFB_MESSAGE_TYPE_SLI: u8 = 2;
    pub const PSFB_MESSAGE_TYPE_RPSI: u8 = 3;
    pub const PSFB_MESSAGE_TYPE_AFB: u8 = 15;
}





