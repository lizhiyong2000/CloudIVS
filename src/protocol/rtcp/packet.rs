use std::collections::HashMap;
use std::str::FromStr;

use strum;
use strum_macros::EnumString;

use super::constants::*;

use super::send_report_packet::SendReportPacket;

use pnet_macros_support::packet::PrimitiveValues;

use handy_async::sync_io::{ReadExt, WriteExt};
use std::io::{Read, Write};

use crate::protocol::rtcp::constants::*;
use crate::protocol::rtcp::traits::*;
use crate::protocol::rtcp::types::*;


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

fn read_sctp<R: Read>(reader: &mut R, expected_type: u8) -> Result<(U5, Vec<u8>)> {
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

fn write_sctp<W: Write>(
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RtcpSenderReport {
    pub ssrc: Ssrc,
    pub ntp_timestamp: NtpTimestamp,
    pub rtp_timestamp: RtpTimestamp,
    pub sent_packets: u32,
    pub sent_octets: u32,
    pub reception_reports: Vec<ReceptionReport>,
    pub extensions: Vec<u8>,
}
impl RtcpSenderReport {
    pub fn new(ssrc: Ssrc) -> Self {
        RtcpSenderReport {
            ssrc: ssrc,
            ntp_timestamp: 0,
            rtp_timestamp: 0,
            sent_packets: 0,
            sent_octets: 0,
            reception_reports: Vec::new(),
            extensions: Vec::new(),
        }
    }
}
impl PacketTrait for RtcpSenderReport {}
impl RtcpPacketTrait for RtcpSenderReport {}
impl ReadFrom for RtcpSenderReport {
    fn read_from<R: Read>(reader: &mut R) -> Result<Self> {
        let (reception_report_count, payload) = track_try!(read_sctp(reader, RTCP_PACKET_TYPE_SR));
        let reader = &mut &payload[..];

        let ssrc = track_try!(reader.read_u32be());

        let ntp_timestamp = track_try!(reader.read_u64be());
        let rtp_timestamp = track_try!(reader.read_u32be());
        let sent_packets = track_try!(reader.read_u32be());
        let sent_octets = track_try!(reader.read_u32be());

        let mut reception_reports = Vec::new();
        for _ in 0..reception_report_count {
            let report = track_try!(ReceptionReport::read_from(reader));
            reception_reports.push(report);
        }
        let extensions = track_try!(reader.read_all_bytes());

        Ok(RtcpSenderReport {
            ssrc: ssrc,
            ntp_timestamp: ntp_timestamp,
            rtp_timestamp: rtp_timestamp,
            sent_packets: sent_packets,
            sent_octets: sent_octets,
            reception_reports: reception_reports,
            extensions: extensions,
        })
    }
}
impl WriteTo for RtcpSenderReport {
    fn write_to<W: Write>(&self, writer: &mut W) -> Result<()> {
        let mut payload = Vec::new();
        track_try!((&mut payload).write_u32be(self.ssrc));
        track_try!((&mut payload).write_u64be(self.ntp_timestamp));
        track_try!((&mut payload).write_u32be(self.rtp_timestamp));
        track_try!((&mut payload).write_u32be(self.sent_packets));
        track_try!((&mut payload).write_u32be(self.sent_octets));
        for report in self.reception_reports.iter() {
            track_try!(report.write_to(&mut payload));
        }
        payload.extend(&self.extensions);

        track_assert!(
            self.reception_reports.len() <= 0x0001_1111,
            ErrorKind::Invalid
        );
        track_try!(write_sctp(
            writer,
            RTCP_PACKET_TYPE_SR,
            self.reception_reports.len() as u8,
            &payload
        ));
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReceptionReport {
    pub ssrc: Ssrc,
    pub fraction_lost: u8,
    pub packets_lost: U24,
    pub seq_num_ext: u32,
    pub jitter: u32,
    pub last_sr_timestamp: NtpMiddleTimetamp,
    pub delay_since_last_sr: u32,
}
impl ReceptionReport {
    pub fn new(ssrc: Ssrc) -> Self {
        ReceptionReport {
            ssrc: ssrc,
            fraction_lost: 0,
            packets_lost: 0,
            seq_num_ext: 0,
            jitter: 0,
            last_sr_timestamp: 0,
            delay_since_last_sr: 0,
        }
    }
}
impl ReadFrom for ReceptionReport {
    fn read_from<R: Read>(reader: &mut R) -> Result<Self> {
        let ssrc = track_try!(reader.read_u32be());
        let fraction_lost = track_try!(reader.read_u8());
        let packets_lost = track_try!(reader.read_u24be());
        let seq_num_ext = track_try!(reader.read_u32be());
        let jitter = track_try!(reader.read_u32be());
        let last_sr_timestamp = track_try!(reader.read_u32be());
        let delay_since_last_sr = track_try!(reader.read_u32be());

        Ok(ReceptionReport {
            ssrc: ssrc,
            fraction_lost: fraction_lost,
            packets_lost: packets_lost,
            seq_num_ext: seq_num_ext,
            jitter: jitter,
            last_sr_timestamp: last_sr_timestamp,
            delay_since_last_sr: delay_since_last_sr,
        })
    }
}
impl WriteTo for ReceptionReport {
    fn write_to<W: Write>(&self, writer: &mut W) -> Result<()> {
        track_assert!(self.packets_lost <= 0x00FF_FFFF, ErrorKind::Invalid);

        track_try!(writer.write_u32be(self.ssrc));
        track_try!(writer.write_u8(self.fraction_lost));
        track_try!(writer.write_u24be(self.packets_lost));
        track_try!(writer.write_u32be(self.seq_num_ext));
        track_try!(writer.write_u32be(self.jitter));
        track_try!(writer.write_u32be(self.last_sr_timestamp));
        track_try!(writer.write_u32be(self.delay_since_last_sr));

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RtcpReceiverReport {
    pub ssrc: Ssrc,
    pub reception_reports: Vec<ReceptionReport>,
    pub extensions: Vec<u8>,
}
impl RtcpReceiverReport {
    pub fn new(ssrc: Ssrc) -> Self {
        RtcpReceiverReport {
            ssrc: ssrc,
            reception_reports: Vec::new(),
            extensions: Vec::new(),
        }
    }
}
impl PacketTrait for RtcpReceiverReport {}
impl RtcpPacketTrait for RtcpReceiverReport {}
impl ReadFrom for RtcpReceiverReport {
    fn read_from<R: Read>(reader: &mut R) -> Result<Self> {
        let (reception_report_count, payload) = track_try!(read_sctp(reader, RTCP_PACKET_TYPE_RR));
        let reader = &mut &payload[..];

        let ssrc = track_try!(reader.read_u32be());

        let mut reception_reports = Vec::new();
        for _ in 0..reception_report_count {
            let report = track_try!(ReceptionReport::read_from(reader));
            reception_reports.push(report);
        }
        let extensions = track_try!(reader.read_all_bytes());

        Ok(RtcpReceiverReport {
            ssrc: ssrc,
            reception_reports: reception_reports,
            extensions: extensions,
        })
    }
}
impl WriteTo for RtcpReceiverReport {
    fn write_to<W: Write>(&self, writer: &mut W) -> Result<()> {
        let mut payload = Vec::new();
        track_try!((&mut payload).write_u32be(self.ssrc));
        for report in self.reception_reports.iter() {
            track_try!(report.write_to(&mut payload));
        }
        payload.extend(&self.extensions);

        track_assert!(
            self.reception_reports.len() <= 0b0001_1111,
            ErrorKind::Invalid
        );
        track_try!(write_sctp(
            writer,
            RTCP_PACKET_TYPE_RR,
            self.reception_reports.len() as u8,
            &payload
        ));
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RtcpSourceDescription {
    pub chunks: Vec<SdesChunk>,
}
impl RtcpSourceDescription {
    pub fn new() -> Self {
        RtcpSourceDescription { chunks: Vec::new() }
    }
}
impl PacketTrait for RtcpSourceDescription {}
impl RtcpPacketTrait for RtcpSourceDescription {}
impl ReadFrom for RtcpSourceDescription {
    fn read_from<R: Read>(reader: &mut R) -> Result<Self> {
        let (source_count, payload) = track_try!(read_sctp(reader, RTCP_PACKET_TYPE_SDES));
        let reader = &mut &payload[..];

        let chunks = track_try!(
            (0..source_count)
                .map(|_| SdesChunk::read_from(reader))
                .collect()
        );
        Ok(RtcpSourceDescription { chunks: chunks })
    }
}
impl WriteTo for RtcpSourceDescription {
    fn write_to<W: Write>(&self, writer: &mut W) -> Result<()> {
        let mut payload = Vec::new();
        for chunk in self.chunks.iter() {
            track_try!(chunk.write_to(&mut payload));
        }

        track_assert!(self.chunks.len() <= 0b0001_1111, ErrorKind::Invalid);
        track_try!(write_sctp(
            writer,
            RTCP_PACKET_TYPE_SDES,
            self.chunks.len() as u8,
            &payload
        ));
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SdesChunk {
    pub ssrc_or_csrc: SsrcOrCsrc,
    pub items: Vec<SdesItem>,
}
impl ReadFrom for SdesChunk {
    fn read_from<R: Read>(reader: &mut R) -> Result<Self> {
        let mut read_bytes = 0;

        let ssrc_or_csrc = track_try!(reader.read_u32be());
        read_bytes += 4;

        let mut items = Vec::new();
        loop {
            let ty = track_try!(reader.read_u8());
            read_bytes += 1;

            if ty == SDES_ITEM_TYPE_END {
                break;
            }
            let len = track_try!(reader.read_u8()) as usize;
            let text = track_try!(reader.read_string(len));

            read_bytes += 1 + len;
            let item = match ty {
                SDES_ITEM_TYPE_CNAME => SdesItem::Cname(text),
                SDES_ITEM_TYPE_NAME => SdesItem::Name(text),
                SDES_ITEM_TYPE_EMAIL => SdesItem::Email(text),
                SDES_ITEM_TYPE_PHONE => SdesItem::Phone(text),
                SDES_ITEM_TYPE_LOC => SdesItem::Loc(text),
                SDES_ITEM_TYPE_TOOL => SdesItem::Tool(text),
                SDES_ITEM_TYPE_NOTE => SdesItem::Note(text),
                SDES_ITEM_TYPE_PRIV => SdesItem::Priv(text),
                _ => track_panic!(ErrorKind::Unsupported, "Unknown SDES item type: {}", ty),
            };
            items.push(item);
        }
        let padding_len = (4 - read_bytes % 4) % 4;
        track_try!(reader.read_bytes(padding_len as usize)); // discard

        Ok(SdesChunk {
            ssrc_or_csrc: ssrc_or_csrc,
            items: items,
        })
    }
}
impl WriteTo for SdesChunk {
    fn write_to<W: Write>(&self, writer: &mut W) -> Result<()> {
        let mut write_bytes = 0;

        track_try!(writer.write_u32be(self.ssrc_or_csrc));
        write_bytes += 4;

        for item in self.items.iter() {
            track_try!(writer.write_u8(item.item_type()));
            write_bytes += 1;

            let text = item.text();
            track_assert!(text.len() <= 0xFFFF, ErrorKind::Invalid);
            track_try!(writer.write_u16be(text.len() as u16));
            track_try!(writer.write_all(text.as_bytes()));
            write_bytes += 2 + text.len();
        }
        track_try!(writer.write_u8(SDES_ITEM_TYPE_END));
        write_bytes += 1;

        let padding_len = (4 - write_bytes % 4) % 4;
        for _ in 0..padding_len {
            track_try!(writer.write_u8(0));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SdesItem {
    Cname(String),
    Name(String),
    Email(String),
    Phone(String),
    Loc(String),
    Tool(String),
    Note(String),
    Priv(String),
}
impl SdesItem {
    pub fn item_type(&self) -> u8 {
        match *self {
            SdesItem::Cname(_) => SDES_ITEM_TYPE_CNAME,
            SdesItem::Name(_) => SDES_ITEM_TYPE_NAME,
            SdesItem::Email(_) => SDES_ITEM_TYPE_EMAIL,
            SdesItem::Phone(_) => SDES_ITEM_TYPE_PHONE,
            SdesItem::Loc(_) => SDES_ITEM_TYPE_LOC,
            SdesItem::Tool(_) => SDES_ITEM_TYPE_TOOL,
            SdesItem::Note(_) => SDES_ITEM_TYPE_NOTE,
            SdesItem::Priv(_) => SDES_ITEM_TYPE_PRIV,
        }
    }
    pub fn text(&self) -> &str {
        match *self {
            SdesItem::Cname(ref t) => t,
            SdesItem::Name(ref t) => t,
            SdesItem::Email(ref t) => t,
            SdesItem::Phone(ref t) => t,
            SdesItem::Loc(ref t) => t,
            SdesItem::Tool(ref t) => t,
            SdesItem::Note(ref t) => t,
            SdesItem::Priv(ref t) => t,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RtcpGoodbye {
    pub ssrc_csrc_list: Vec<SsrcOrCsrc>,
    pub reason: Option<String>,
}
impl RtcpGoodbye {
    pub fn new() -> Self {
        RtcpGoodbye {
            ssrc_csrc_list: Vec::new(),
            reason: None,
        }
    }
}
impl PacketTrait for RtcpGoodbye {}
impl RtcpPacketTrait for RtcpGoodbye {}
impl ReadFrom for RtcpGoodbye {
    fn read_from<R: Read>(reader: &mut R) -> Result<Self> {
        let (source_count, payload) = track_try!(read_sctp(reader, RTCP_PACKET_TYPE_BYE));
        let reader = &mut &payload[..];

        let list = track_try!((0..source_count).map(|_| reader.read_u32be()).collect());
        let mut reason = None;
        if let Ok(len) = reader.read_u8() {
            reason = Some(track_try!(reader.read_string(len as usize)));
        }
        Ok(RtcpGoodbye {
            ssrc_csrc_list: list,
            reason: reason,
        })
    }
}
impl WriteTo for RtcpGoodbye {
    fn write_to<W: Write>(&self, writer: &mut W) -> Result<()> {
        let mut payload = Vec::new();
        for x in self.ssrc_csrc_list.iter() {
            track_try!((&mut payload).write_u32be(*x));
        }
        if let Some(ref reason) = self.reason {
            track_assert!(reason.len() <= 0xFF, ErrorKind::Invalid);
            track_try!((&mut payload).write_u8(reason.len() as u8));
            track_try!((&mut payload).write_all(reason.as_bytes()));

            let padding_len = (4 - (reason.len() + 1) % 4) % 4;
            for _ in 0..padding_len {
                track_try!((&mut payload).write_u8(0));
            }
        }

        track_assert!(self.ssrc_csrc_list.len() <= 0b0001_1111, ErrorKind::Invalid);
        track_try!(write_sctp(
            writer,
            RTCP_PACKET_TYPE_BYE,
            self.ssrc_csrc_list.len() as u8,
            &payload
        ));
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RtcpApplicationDefined {
    pub subtype: U5,
    pub ssrc_or_csrc: SsrcOrCsrc,
    pub name: [u8; 4],
    pub data: Vec<u8>,
}
impl PacketTrait for RtcpApplicationDefined {}
impl RtcpPacketTrait for RtcpApplicationDefined {}
impl ReadFrom for RtcpApplicationDefined {
    fn read_from<R: Read>(reader: &mut R) -> Result<Self> {
        let (subtype, payload) = track_try!(read_sctp(reader, RTCP_PACKET_TYPE_APP));
        let reader = &mut &payload[..];

        let ssrc_or_csrc = track_try!(reader.read_u32be());
        let mut name = [0; 4];
        track_try!(reader.read_exact(&mut name));
        let data = track_try!(reader.read_all_bytes());
        Ok(RtcpApplicationDefined {
            subtype: subtype,
            ssrc_or_csrc: ssrc_or_csrc,
            name: name,
            data: data,
        })
    }
}
impl WriteTo for RtcpApplicationDefined {
    fn write_to<W: Write>(&self, writer: &mut W) -> Result<()> {
        let mut payload = Vec::new();
        track_try!((&mut payload).write_u32be(self.ssrc_or_csrc));
        payload.extend(&self.name);
        payload.extend(&self.data);

        track_assert!(self.subtype <= 0b0001_1111, ErrorKind::Invalid);
        track_try!(write_sctp(
            writer,
            RTCP_PACKET_TYPE_APP,
            self.subtype,
            &payload
        ));
        Ok(())
    }
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
    use crate::protocol::rtcp::packet::CommonHeader;
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