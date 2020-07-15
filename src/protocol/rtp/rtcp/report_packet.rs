use std::io::{Read, Write};


use crate::common::sync_io::{ReadExt, WriteExt};

use crate::protocol::error::ErrorKind;
use crate::protocol::rtp::traits::RtcpPacketTrait;
use crate::protocol::traits::*;
use crate::protocol::traits::{PacketTrait, ReadFrom, ReadPacket, Result, WritePacket, WriteTo};
use crate::protocol::types::*;
use crate::protocol::types::U24;

use super::constants::*;
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



#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReceptionReport {
    pub ssrc: u32,
    pub fraction_lost: u8,
    pub packets_lost: U24,
    pub seq_num_ext: u32,
    pub jitter: u32,
    pub last_sr_timestamp: u32,
    pub delay_since_last_sr: u32,
}
impl ReceptionReport {
    pub fn new(ssrc: u32) -> Self {
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
pub struct SenderReportPacket {
    pub ssrc: u32,
    pub ntp_sec: u32,
    pub ntp_frac: u32,
    pub rtp_timestamp: u32,
    pub sent_packets: u32,
    pub sent_octets: u32,
    pub reception_reports: Vec<ReceptionReport>,
    pub extensions: Vec<u8>,
}
impl SenderReportPacket {
    pub fn new(ssrc: u32) -> Self {
        SenderReportPacket {
            ssrc: ssrc,
            ntp_sec: 0,
            ntp_frac:0,
            rtp_timestamp: 0,
            sent_packets: 0,
            sent_octets: 0,
            reception_reports: Vec::new(),
            extensions: Vec::new(),
        }
    }
}
impl PacketTrait for SenderReportPacket {}
impl RtcpPacketTrait for SenderReportPacket {}
impl ReadFrom for SenderReportPacket {
    fn read_from<R: Read>(reader: &mut R) -> Result<Self> {
        let (reception_report_count, payload) = track_try!(read_sctp(reader, RTCP_PACKET_TYPE_SR));
        let reader = &mut &payload[..];

        let ssrc = track_try!(reader.read_u32be());

        let ntp_sec = track_try!(reader.read_u32be());
        let ntp_frac  = track_try!(reader.read_u32be());
        let rtp_timestamp = track_try!(reader.read_u32be());
        let sent_packets = track_try!(reader.read_u32be());
        let sent_octets = track_try!(reader.read_u32be());

        let mut reception_reports = Vec::new();
        for _ in 0..reception_report_count {
            let report = track_try!(ReceptionReport::read_from(reader));
            reception_reports.push(report);
        }
        let extensions = track_try!(reader.read_all_bytes());

        Ok(SenderReportPacket {
            ssrc: ssrc,
            ntp_sec: ntp_sec,
            ntp_frac: ntp_frac,
            rtp_timestamp: rtp_timestamp,
            sent_packets: sent_packets,
            sent_octets: sent_octets,
            reception_reports: reception_reports,
            extensions: extensions,
        })
    }
}
impl WriteTo for SenderReportPacket {
    fn write_to<W: Write>(&self, writer: &mut W) -> Result<()> {
        let mut payload = Vec::new();
        track_try!((&mut payload).write_u32be(self.ssrc));
        track_try!((&mut payload).write_u32be(self.ntp_sec));
        track_try!((&mut payload).write_u32be(self.ntp_frac));
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
pub struct ReceiverReportPacket {
    pub ssrc: u32,
    pub reception_reports: Vec<ReceptionReport>,
    pub extensions: Vec<u8>,
}
impl ReceiverReportPacket {
    pub fn new(ssrc: u32) -> Self {
        ReceiverReportPacket {
            ssrc: ssrc,
            reception_reports: Vec::new(),
            extensions: Vec::new(),
        }
    }
}
impl PacketTrait for ReceiverReportPacket {}
impl RtcpPacketTrait for ReceiverReportPacket {}
impl ReadFrom for ReceiverReportPacket {
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

        Ok(ReceiverReportPacket {
            ssrc: ssrc,
            reception_reports: reception_reports,
            extensions: extensions,
        })
    }
}
impl WriteTo for ReceiverReportPacket {
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



#[cfg(test)]
mod tests {
    use crate::protocol::rtp::rtcp::report_packet::ReceptionReport;
    use crate::protocol::traits::{ReadFrom, WriteTo};

    use super::ReceiverReportPacket;
    use super::SenderReportPacket;

    struct SetupSR {
        data: Vec<u8>,
        // SR values.
        ssrc: u32 ,
        ntpSec: u32 ,
        ntpFrac: u32 ,
        rtpTs: u32 ,
        packetCount: u32 ,
        octetCount: u32 ,
    }

    impl SetupSR {
        fn new() -> Self {
            Self {
                data: vec![
                    0x80, 0xc8, 0x00, 0x06, // Type: 200 (Sender Report), Count: 0, Length: 6
                    0x5d, 0x93, 0x15, 0x34, // SSRC: 0x5d931534
                    0xdd, 0x3a, 0xc1, 0xb4, // NTP Sec: 3711615412
                    0x76, 0x54, 0x71, 0x71, // NTP Frac: 1985245553
                    0x00, 0x08, 0xcf, 0x00, // RTP timestamp: 577280
                    0x00, 0x00, 0x0e, 0x18, // Packet count: 3608
                    0x00, 0x08, 0xcf, 0x00  // Octet count: 577280
                ],

                ssrc:0x5d931534,
                ntpSec: 3711615412 ,
                ntpFrac: 1985245553 ,
                rtpTs: 577280 ,
                packetCount: 3608 ,
                octetCount:577280 ,
            }
        }
    }


    struct SetupRR {
        data: Vec<u8>,
        sender_ssrc: u32,
        // SR values.
        ssrc: u32 ,
        fractionLost: u8 ,
        totalLost: u32 ,
        lastSeq: u32 ,
        jitter: u32 ,
        lastSenderReport:u32 ,
        delaySinceLastSenderReport: u32
    }

    impl SetupRR {
        fn new() -> Self {
            Self {
                data: vec![
                    0x81, 0xc9, 0x00, 0x07, // Type: 201 (Receiver Report), Count: 1, Length: 7
                    0x5d, 0x93, 0x15, 0x34, // Sender SSRC: 0x5d931534
                    // Receiver Report
                    0x01, 0x93, 0x2d, 0xb4, // SSRC. 0x01932db4
                    0x00, 0x00, 0x00, 0x01, // Fraction lost: 0, Total lost: 1
                    0x00, 0x00, 0x00, 0x00, // Extended highest sequence number: 0
                    0x00, 0x00, 0x00, 0x00, // Jitter: 0
                    0x00, 0x00, 0x00, 0x00, // Last SR: 0
                    0x00, 0x00, 0x00, 0x05  // DLSR: 0
                ],
                sender_ssrc: 0x5d931534,
                ssrc:0x01932db4,
                fractionLost: 0 ,
                totalLost: 1 ,
                lastSeq: 0 ,
                jitter: 0 ,
                lastSenderReport:0 ,
                delaySinceLastSenderReport:5

            }
        }
    }

    #[test]
    fn test_sender_report_parse() {
        let setup = SetupSR::new();

        let reader = &mut &setup.data[..];
        let sr = SenderReportPacket::read_from(reader).unwrap();

        assert_eq!(sr.ssrc, setup.ssrc);
        assert_eq!(sr.ntp_sec, setup.ntpSec);
        assert_eq!(sr.ntp_frac, setup.ntpFrac);
        assert_eq!(sr.rtp_timestamp, setup.rtpTs);
        assert_eq!(sr.sent_packets, setup.packetCount);
        assert_eq!(sr.sent_octets, setup.octetCount);

        let serialized = sr.to_bytes().unwrap();

        assert_eq!(serialized, setup.data);

    }


    #[test]
    fn test_sender_report_create() {
        let setup = SetupSR::new();

        let reader = &mut &setup.data[..];
        let sr = SenderReportPacket {
            ssrc: setup.ssrc,
            ntp_sec: setup.ntpSec,
            ntp_frac: setup.ntpFrac,
            rtp_timestamp: setup.rtpTs,
            sent_packets: setup.packetCount,
            sent_octets: setup.octetCount,
            reception_reports: Vec::new(),
            extensions: Vec::new(),
        };

        assert_eq!(sr.ssrc, setup.ssrc);
        assert_eq!(sr.ntp_sec, setup.ntpSec);
        assert_eq!(sr.ntp_frac, setup.ntpFrac);
        assert_eq!(sr.rtp_timestamp, setup.rtpTs);
        assert_eq!(sr.sent_packets, setup.packetCount);
        assert_eq!(sr.sent_octets, setup.octetCount);

        let serialized = sr.to_bytes().unwrap();

        assert_eq!(serialized, setup.data);

    }



    #[test]
    fn test_receiver_report_parse() {
        let setup = SetupRR::new();

        let reader = &mut &setup.data[..];
        let rrp = ReceiverReportPacket::read_from(reader).unwrap();

        assert_eq!(rrp.ssrc, setup.sender_ssrc);

        assert_eq!(rrp.reception_reports.len(), 1);

        let rr: &ReceptionReport = rrp.reception_reports.get(0).unwrap();

        assert_eq!(rr.ssrc, setup.ssrc);
        assert_eq!(rr.fraction_lost, setup.fractionLost);
        assert_eq!(rr.packets_lost, setup.totalLost);
        assert_eq!(rr.seq_num_ext, setup.lastSeq);
        assert_eq!(rr.jitter, setup.jitter);
        assert_eq!(rr.last_sr_timestamp, setup.lastSenderReport);
        assert_eq!(rr.delay_since_last_sr, setup.delaySinceLastSenderReport);

        let serialized = rrp.to_bytes().unwrap();

        assert_eq!(serialized, setup.data);

    }


    #[test]
    fn test_receiver_report_create() {
        let setup = SetupRR::new();

        let reader = &mut &setup.data[..];

        let rr = ReceptionReport{
            ssrc: setup.ssrc,
            fraction_lost: setup.fractionLost,
            packets_lost: setup.totalLost,
            seq_num_ext: setup.lastSeq,
            jitter: setup.jitter,
            last_sr_timestamp: setup.lastSenderReport,
            delay_since_last_sr: setup.delaySinceLastSenderReport,
        };

        assert_eq!(rr.ssrc, setup.ssrc);
        assert_eq!(rr.fraction_lost, setup.fractionLost);
        assert_eq!(rr.packets_lost, setup.totalLost);
        assert_eq!(rr.seq_num_ext, setup.lastSeq);
        assert_eq!(rr.jitter, setup.jitter);
        assert_eq!(rr.last_sr_timestamp, setup.lastSenderReport);
        assert_eq!(rr.delay_since_last_sr, setup.delaySinceLastSenderReport);

        let reports = vec![rr];

        let rrp = ReceiverReportPacket {
            ssrc:setup.sender_ssrc,
            reception_reports: reports,
            extensions: Vec::new()
        };

        assert_eq!(rrp.ssrc, setup.sender_ssrc);
        assert_eq!(rrp.reception_reports.len(), 1);

        let serialized = rrp.to_bytes().unwrap();
        assert_eq!(serialized, setup.data);

    }


}
