use std::collections::HashMap;
use std::str::FromStr;

use strum;
use strum_macros::EnumString;
use pnet_macros_support::packet::PrimitiveValues;
use handy_async::sync_io::{ReadExt, WriteExt};
use std::io::{Read, Write};

use super::constants::*;
use super::traits::*;
use super::types::*;

use super::report_packet::*;
use super::source_description_packet::*;
use super::bye_packet::*;
use super::app_defined_packet::*;


/// RTCP message types.
///
/// See the [IANA page] on the matter for an up-to-date-list.
///
/// [IANA page]: https://www.iana.org/assignments/rtp-parameters/rtp-parameters.xhtml#rtp-parameters-4
///
/// RFC links:
/// + [https://tools.ietf.org/html/rfc3550#section-12.1](https://tools.ietf.org/html/rfc3550#section-12.1)
///
/// | Abbrev.       | Name       |         Value          |
/// | ------------- | ------------ |:---------------------:|
/// |SR       |sender report        |  200
/// |RR       |receiver report      |  201
/// |SDES     |source description   |  202
/// |BYE      |goodbye              |  203
/// |APP      |application-defined  | 204
#[derive(Copy, Clone, Debug, Eq, PartialEq, Display)]
pub enum RtcpPacketType {
    /// SMPTE time-code mapping.
    ///
    /// See [RFC 5484](https://tools.ietf.org/html/rfc5484).
    ///
    /// Code 194.
    SmpteMap,

    /// Extended inter-arrival jitter report.
    ///
    /// See [RFC 5450](https://tools.ietf.org/html/rfc5450).
    ///
    /// Code 195.
    JitterReport,

    /// Sender report, containing jitter, reception, timing and volume information.
    ///
    /// See the relevant [RTP RFC section](https://tools.ietf.org/html/rfc3550#section-6.4.1).
    /// Unlike the `ReceiverReport`, this includes a `SenderInfo` block.
    ///
    /// Code 200.
    SenderReport,

    /// Sender report, containing jitter and reception information.
    ///
    /// See the relevant [RTP RFC section](https://tools.ietf.org/html/rfc3550#section-6.4.2).
    ///
    /// Code 201.
    ReceiverReport,

    /// Source description, mapping SSRC/CCRC values to information about each host.
    ///
    /// See the relevant [RTP RFC section](https://tools.ietf.org/html/rfc3550#section-6.5).
    ///
    /// Code 202.
    SourceDescription,

    /// Source exit message, denoting SSRC/CCRC of exiting hosts and an optional reason string.
    ///
    /// See the relevant [RTP RFC section](https://tools.ietf.org/html/rfc3550#section-6.6).
    ///
    /// Code 203.
    Goodbye,

    /// Application-defined RTCP message, containing a name and arbitrary data.
    ///
    /// See the relevant [RTP RFC section](https://tools.ietf.org/html/rfc3550#section-6.7).
    ///
    /// Code 204.
    ApplicationDefined,

    /// RTPFB, feedback on the RTP transport layer.
    ///
    /// See [RFC 4585](https://tools.ietf.org/html/rfc4585)
    ///
    /// Code 205.
    TransportFeedback,

    /// PSFB, feedback on the payload.
    ///
    /// See [RFC 4585](https://tools.ietf.org/html/rfc4585)
    ///
    /// Code 206.
    PayloadFeedback,

    /// Extended Report message, used for additional/mixed report blocks.
    ///
    /// See [RTCP XR](https://tools.ietf.org/html/rfc3611).
    ///
    /// Code 207.
    ExtendedReport,

    /// AVB RTCP packet.
    ///
    /// See [IEEE P1733](https://ieeexplore.ieee.org/document/5154142).
    ///
    /// Code 208.
    Avb,

    /// Receiver Summary information.
    ///
    /// See [RFC 5760](https://tools.ietf.org/html/rfc5760).
    ///
    /// Code 209.
    ReceiverSummary,

    /// Port mapping.
    ///
    /// See [RFC 6284](https://tools.ietf.org/html/rfc6284).
    ///
    /// Code 210.
    PortMapping,

    /// IDMS settings.
    ///
    /// See [RFC 7272](https://tools.ietf.org/html/rfc7272).
    ///
    /// Code 211.
    Idms,

    /// Reporting group reporting sources.
    ///
    /// See the [draft RFC](https://datatracker.ietf.org/doc/draft-ietf-avtcore-rtp-multi-stream-optimisation/).
    ///
    /// Code 212.
    ReportingGroupSources,

    /// Splicing notification message.
    ///
    /// See [RFC 8286](https://tools.ietf.org/html/rfc8286).
    ///
    /// Code 213.
    SplicingNotification,

    /// Explicitly reserved code point.
    Reserved(u8),

    /// Unknown message type.
    Unassigned(u8),
}

impl<'a> RtcpPacketType {
    pub fn new(val: u8) -> Self {
        use RtcpPacketType::*;
        match val {
            194 => SmpteMap,
            195 => JitterReport,
            200 => SenderReport,
            201 => ReceiverReport,
            202 => SourceDescription,
            203 => Goodbye,
            204 => ApplicationDefined,
            205 => TransportFeedback,
            206 => PayloadFeedback,
            207 => ExtendedReport,
            208 => Avb,
            209 => ReceiverSummary,
            210 => PortMapping,
            211 => Idms,
            212 => ReportingGroupSources,
            213 => SplicingNotification,
            0 | 192 | 193 | 255 => Reserved(val),
            _ => Unassigned(val),
        }
    }

}

impl PrimitiveValues for RtcpPacketType {
    type T = u8;

    fn to_primitive_values(&self) -> Self::T {
        use RtcpPacketType::*;
        match self {
            SmpteMap => 194,
            JitterReport =>195,
            SenderReport => 200,
            ReceiverReport => 201,
            SourceDescription => 202,
            Goodbye => 203,
            ApplicationDefined => 204,
            TransportFeedback => 205,
            PayloadFeedback => 206,
            ExtendedReport => 207,
            Avb => 208,
            ReceiverSummary => 209,
            PortMapping => 210,
            Idms => 211,
            ReportingGroupSources => 212,
            SplicingNotification => 213,

            Reserved(val) => *val,
            Unassigned(val) => *val,
        }
    }
}


const BUFFER_SIZE:i32 = 65536;
// extern uint8_t Buffer[BufferSize];

// Maximum interval for regular RTCP mode.
const MAX_AUDIO_INTERVAL_MS:i32 = 5000;
const MAX_VIDEO_INTERVAL_MS:i32 = 1000;

pub struct CommonHeader {
    version: u8,
    padding: u8,
    count: u8,
    packetType: RtcpPacketType,
    length: u16,
}

impl PacketData for CommonHeader{
    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes: Vec<u8> = vec![];
        let start = format!("{:02b}", self.version) + format!("{:01b}", self.padding).as_ref() + format!("{:05b}", self.count).as_ref();
        bytes.push(u8::from_str_radix(&start[..], 2).unwrap());

        bytes.push(self.packetType.to_primitive_values());

        let length = format!("{:016b}", self.length);
        bytes.push(u8::from_str_radix(&length[0..8], 2).unwrap());
        bytes.push(u8::from_str_radix(&length[8..16], 2).unwrap());

        bytes
    }
}

// /// RTCP Packet Format ( https://tools.ietf.org/html/rfc3550#section-6.1 )
// ///
// /// These define the packet format used for both the header and payload.
// /// + SR: Sender Report RTCP Packet       : https://tools.ietf.org/html/rfc3550#section-6.4.1
// /// + RR: Receiver Report RTCP Packet     : https://tools.ietf.org/html/rfc3550#section-6.4.2
// /// + SDES: Source Description RTCP Packet: https://tools.ietf.org/html/rfc3550#section-6.5
// /// + BYE: Goodbye RTCP Packet            : https://tools.ietf.org/html/rfc3550#section-6.6
// /// + APP: Application-Defined RTCP Packet: https://tools.ietf.org/html/rfc3550#section-6.7
// pub enum RtcpPacket {
//     SMPTETC(),
//     SR(SendReportPacket),     // Sender Report RTCP Packet
//     RR(ReceiverReportPacket),     // Receiver Report RTCP Packet
//     SDES(SourceDescriptionPacket), // Source Description RTCP Packet
//     BYE(GoodbyePacket),   // Goodbye RTCP Packet
//     APP(ApplicationDefinedPacket),    // Application-Defined RTCP Packet
//     RTPFB(),
//     PSFB(),
//     XR(),
//     AVB(),
//     RSI(),
//     TOKEN(),
//     IDMS(),
//     RGRS(),
//     SNM(),
//
// }




#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RtcpPacketReader;
impl ReadPacket for RtcpPacketReader {
    type Packet = RtcpCompoundPacket<RtcpPacket>;
    fn read_packet<R: Read>(&mut self, reader: &mut R) -> Result<Self::Packet> {
        // TODO: optimize
        let buf = track_try!(reader.read_all_bytes());
        let mut packets = Vec::new();
        let reader = &mut &buf[..];
        while !reader.is_empty() {
            let packet = track_try!(RtcpPacket::read_from(reader));
            packets.push(packet);
        }
        Ok(RtcpCompoundPacket::new(packets))
    }
    fn supports_type(&self, ty: u8) -> bool {
        match ty {
            RTCP_PACKET_TYPE_SR
            | RTCP_PACKET_TYPE_RR
            | RTCP_PACKET_TYPE_SDES
            | RTCP_PACKET_TYPE_BYE
            | RTCP_PACKET_TYPE_APP => true,
            _ => false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RtcpPacketWriter;
impl WritePacket for RtcpPacketWriter {
    type Packet = RtcpCompoundPacket<RtcpPacket>;
    fn write_packet<W: Write>(&mut self, writer: &mut W, packet: &Self::Packet) -> Result<()> {
        for p in packet.packets.iter() {
            track_try!(p.write_to(writer));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RtcpCompoundPacket<T> {
    pub packets: Vec<T>,
}
impl<T> RtcpCompoundPacket<T> {
    pub fn new(packets: Vec<T>) -> Self {
        RtcpCompoundPacket { packets: packets }
    }
}
impl<T: PacketTrait> PacketTrait for RtcpCompoundPacket<T> {}
impl<T: RtcpPacketTrait> RtcpPacketTrait for RtcpCompoundPacket<T> {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RtcpPacket {
    Sr(RtcpSenderReport),
    Rr(RtcpReceiverReport),
    Sdes(RtcpSourceDescription),
    Bye(RtcpGoodbye),
    App(RtcpApplicationDefined),
}
impl PacketTrait for RtcpPacket {}
impl RtcpPacketTrait for RtcpPacket {}
impl ReadFrom for RtcpPacket {
    fn read_from<R: Read>(reader: &mut R) -> Result<Self> {
        let mut buf = [0; 2];
        track_try!(reader.read_exact(&mut buf));

        let reader = &mut (&buf[..]).chain(reader);
        let packet_type = buf[1];
        match packet_type {
            RTCP_PACKET_TYPE_SR => track_err!(RtcpSenderReport::read_from(reader).map(From::from)),
            RTCP_PACKET_TYPE_RR => {
                track_err!(RtcpReceiverReport::read_from(reader).map(From::from))
            }
            RTCP_PACKET_TYPE_SDES => {
                track_err!(RtcpSourceDescription::read_from(reader).map(From::from))
            }
            RTCP_PACKET_TYPE_BYE => track_err!(RtcpGoodbye::read_from(reader).map(From::from)),
            RTCP_PACKET_TYPE_APP => {
                track_err!(RtcpApplicationDefined::read_from(reader).map(From::from))
            }
            _ => track_panic!(
                ErrorKind::Unsupported,
                "Unknown packet type: {}",
                packet_type
            ),
        }
    }
}
impl WriteTo for RtcpPacket {
    fn write_to<W: Write>(&self, writer: &mut W) -> Result<()> {
        match *self {
            RtcpPacket::Sr(ref p) => track_err!(p.write_to(writer)),
            RtcpPacket::Rr(ref p) => track_err!(p.write_to(writer)),
            RtcpPacket::Sdes(ref p) => track_err!(p.write_to(writer)),
            RtcpPacket::Bye(ref p) => track_err!(p.write_to(writer)),
            RtcpPacket::App(ref p) => track_err!(p.write_to(writer)),
        }
    }
}
impl From<RtcpSenderReport> for RtcpPacket {
    fn from(f: RtcpSenderReport) -> Self {
        RtcpPacket::Sr(f)
    }
}
impl From<RtcpReceiverReport> for RtcpPacket {
    fn from(f: RtcpReceiverReport) -> Self {
        RtcpPacket::Rr(f)
    }
}
impl From<RtcpSourceDescription> for RtcpPacket {
    fn from(f: RtcpSourceDescription) -> Self {
        RtcpPacket::Sdes(f)
    }
}
impl From<RtcpGoodbye> for RtcpPacket {
    fn from(f: RtcpGoodbye) -> Self {
        RtcpPacket::Bye(f)
    }
}
impl From<RtcpApplicationDefined> for RtcpPacket {
    fn from(f: RtcpApplicationDefined) -> Self {
        RtcpPacket::App(f)
    }
}

pub fn read_sctp<R: Read>(reader: &mut R, expected_type: u8) -> Result<(U5, Vec<u8>)> {
    let b = track_try!(reader.read_u8());
    track_assert_eq!(
        b >> 6,
        RTP_VERSION,
        ErrorKind::Unsupported,
        "Unsupported RTP version: {}",
        b >> 6
    );
    let padding = (b & 0b0010_0000) != 0;
    let packet_specific = b & 0b0001_1111;

    let packet_type = track_try!(reader.read_u8());
    track_assert_eq!(
        packet_type,
        expected_type,
        ErrorKind::Invalid,
        "Unexpected SCTP packet type: actual={}, expected={}",
        packet_type,
        expected_type
    );

    let word_count = track_try!(reader.read_u16be()) as usize;
    let mut payload = track_try!(reader.read_bytes(word_count * 4));

    if padding {
        let payload_len = payload.len();
        track_assert_ne!(payload_len, 0, ErrorKind::Invalid);

        let padding_len = payload[payload_len - 1] as usize;
        track_assert!(padding_len <= payload.len(), ErrorKind::Invalid);

        payload.truncate(payload_len - padding_len);
    }
    track_assert_eq!(payload.len() % 4, 0, ErrorKind::Invalid);

    Ok((packet_specific, payload))
}

pub fn write_sctp<W: Write>(
    writer: &mut W,
    packet_type: u8,
    packet_specific: U5,
    payload: &[u8],
) -> Result<()> {
    track_assert_eq!(payload.len() % 4, 0, ErrorKind::Invalid);

    track_try!(writer.write_u8(RTP_VERSION << 6 | packet_specific));
    track_try!(writer.write_u8(packet_type));

    let word_count = payload.len() / 4;
    track_assert!(word_count < 0x10000, ErrorKind::Invalid);

    track_try!(writer.write_u16be(word_count as u16));
    track_try!(writer.write_all(payload));

    Ok(())
}






/*
        0                   1                   2                   3
        0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
       +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
header |V=2|P|    RC   |   PT=RR=201   |             length            |
       +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
       |                     SSRC of packet sender                     |
       +=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+
report |                 SSRC_1 (SSRC of first source)                 |
block  +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
  1    | fraction lost |       cumulative number of packets lost       |
       +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
       |           extended highest sequence number received           |
       +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
       |                      interarrival jitter                      |
       +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
       |                         last SR (LSR)                         |
       +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
       |                   delay since last SR (DLSR)                  |
       +=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+
report |                 SSRC_2 (SSRC of second source)                |
block  +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
  2    :                               ...                             :
       +=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+
       |                  profile-specific extensions                  |
       +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
*/
struct ReceiverReportPacket{

}

/*

        0                   1                   2                   3
        0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
       +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
header |V=2|P|    SC   |  PT=SDES=202  |             length            |
       +=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+
chunk  |                          SSRC/CSRC_1                          |
  1    +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
       |                           SDES items                          |
       |                              ...                              |
       +=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+
chunk  |                          SSRC/CSRC_2                          |
  2    +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
       |                           SDES items                          |
       |                              ...                              |
       +=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+

RTP SDES item types:
    https://tools.ietf.org/html/rfc3550#section-12.2
    http://www.iana.org/assignments/rtp-parameters/rtp-parameters.xhtml#rtp-parameters-5

    abbrev.  name                            value
    END      end of SDES list                    0
    CNAME    canonical name                      1
    NAME     user name                           2
    EMAIL    user's electronic mail address      3
    PHONE    user's phone number                 4
    LOC      geographic user location            5
    TOOL     name of application or tool         6
    NOTE     notice about the source             7
    PRIV     private extensions                  8

    0                   1                   2                   3
    0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
   +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
   |    CNAME=1    |     length    | user and domain name        ...
   +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
*/
struct SourceDescriptionPacket{

}


#[derive(Debug)]
pub enum SDES_ITEM {
    // http://www.iana.org/assignments/rtp-parameters/rtp-parameters.xhtml#rtp-parameters-5
    END  { value: u8, length: u8, content: String },      // 0
    CNAME{ value: u8, length: u8, content: String },      // 1
    NAME { value: u8, length: u8, content: String },      // 2
    EMAIL{ value: u8, length: u8, content: String },      // 3
    PHONE{ value: u8, length: u8, content: String },      // 4
    LOC  { value: u8, length: u8, content: String },      // 5
    TOOL { value: u8, length: u8, content: String },      // 6
    NOTE { value: u8, length: u8, content: String },      // 7
    PRIV { value: u8, length: u8, content: String },      // 8
    H323_CADDR { value: u8, length: u8, content: String },// 9
    APSI       { value: u8, length: u8, content: String },// 10
    RGRP       { value: u8, length: u8, content: String },// 11
    UNASSIGNED { value: u8, length: u8, content: String },// 12 - 255, Unassigned
}

struct GoodbyePacket{

}

struct ApplicationDefinedPacket{

}



#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::RtcpPacketType;
    // use super::CommonHeader;
    use pnet_macros_support::packet::PrimitiveValues;
    use crate::protocol::rtcp::rtcp_packet::CommonHeader;
    use crate::protocol::rtcp::traits::PacketData;

    #[test]
    fn test_packet_type() {
        let sr = RtcpPacketType::SenderReport.to_primitive_values();
        assert_eq!(sr, 200);
    }


    #[test]
    fn test_common_header_serialize() {

        let data = vec![0x80, 0xc8, 0x00, 0x00];

        let header = CommonHeader {
            version: 2,
            padding: 0,
            count: 0,
            packetType: RtcpPacketType::SenderReport,
            length: 0,

        };
        let vec = header.to_bytes();
        assert_eq!(data, vec);
    }
}