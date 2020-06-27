use std::io::{Read, Write};
use handy_async::sync_io::{ReadExt, WriteExt};

use super::constants::*;
use super::traits::*;
use super::types::*;
use super::rtcp_packet::*;



/*
        0                   1                   2                   3
        0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
       +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
header |V=2|P|    RC   |   PT=SR=200   |             length            |
       +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
       |                         SSRC of sender                        |
       +=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+
sender |              NTP timestamp, most significant word             |
info   +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
       |             NTP timestamp, least significant word             |
       +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
       |                         RTP timestamp                         |
       +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
       |                     sender's packet count                     |
       +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
       |                      sender's octet count                     |
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