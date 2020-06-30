use std::io::{Read, Write};

use handy_async::sync_io::{ReadExt, WriteExt};

use crate::protocol::error::ErrorKind;
use crate::protocol::rtp::constants::RTP_VERSION;
use crate::protocol::rtp::traits::RtcpPacketTrait;
use crate::protocol::traits::{PacketTrait, ReadFrom, ReadPacket, Result, WritePacket, WriteTo};
use crate::protocol::types::{U13, U5, U6, U7};

use super::constants::*;
use super::feedback::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RtcpPayloadSpecificFeedback {
    Pli(PictureLossIndication),
    Sli(SliceLossIndication),
    Rpsi(ReferencePictureSelectionIndication),
    Afb(ApplicationLayerFeedback),
}
impl PacketTrait for RtcpPayloadSpecificFeedback {}
impl RtcpPacketTrait for RtcpPayloadSpecificFeedback {}
impl ReadFrom for RtcpPayloadSpecificFeedback {
    fn read_from<R: Read>(reader: &mut R) -> Result<Self> {
        let (fb_message_type, rest) = track_try!(read_common(reader, RTCP_PACKET_TYPE_PSFB));
        let reader = &mut &rest[..];
        match fb_message_type {
            PSFB_MESSAGE_TYPE_PLI => {
                track_err!(PictureLossIndication::read_from(reader).map(From::from))
            }
            PSFB_MESSAGE_TYPE_SLI => {
                track_err!(SliceLossIndication::read_from(reader).map(From::from))
            }
            PSFB_MESSAGE_TYPE_RPSI => {
                track_err!(ReferencePictureSelectionIndication::read_from(reader).map(From::from))
            }
            PSFB_MESSAGE_TYPE_AFB => {
                track_err!(ApplicationLayerFeedback::read_from(reader).map(From::from))
            }
            _ => track_panic!(
                ErrorKind::Unsupported,
                "Unknown feedback type: {}",
                fb_message_type
            ),
        }
    }
}
impl WriteTo for RtcpPayloadSpecificFeedback {
    fn write_to<W: Write>(&self, writer: &mut W) -> Result<()> {
        match *self {
            RtcpPayloadSpecificFeedback::Pli(ref f) => {
                let payload = track_try!(f.to_bytes());
                track_err!(write_common(
                    writer,
                    RTCP_PACKET_TYPE_PSFB,
                    PSFB_MESSAGE_TYPE_PLI,
                    &payload
                ))
            }
            RtcpPayloadSpecificFeedback::Sli(ref f) => {
                let payload = track_try!(f.to_bytes());
                track_err!(write_common(
                    writer,
                    RTCP_PACKET_TYPE_PSFB,
                    PSFB_MESSAGE_TYPE_SLI,
                    &payload
                ))
            }
            RtcpPayloadSpecificFeedback::Rpsi(ref f) => {
                let payload = track_try!(f.to_bytes());
                track_err!(write_common(
                    writer,
                    RTCP_PACKET_TYPE_PSFB,
                    PSFB_MESSAGE_TYPE_RPSI,
                    &payload
                ))
            }
            RtcpPayloadSpecificFeedback::Afb(ref f) => {
                let payload = track_try!(f.to_bytes());
                track_err!(write_common(
                    writer,
                    RTCP_PACKET_TYPE_PSFB,
                    PSFB_MESSAGE_TYPE_AFB,
                    &payload
                ))
            }
        }
    }
}
impl From<PictureLossIndication> for RtcpPayloadSpecificFeedback {
    fn from(f: PictureLossIndication) -> Self {
        RtcpPayloadSpecificFeedback::Pli(f)
    }
}
impl From<SliceLossIndication> for RtcpPayloadSpecificFeedback {
    fn from(f: SliceLossIndication) -> Self {
        RtcpPayloadSpecificFeedback::Sli(f)
    }
}
impl From<ReferencePictureSelectionIndication> for RtcpPayloadSpecificFeedback {
    fn from(f: ReferencePictureSelectionIndication) -> Self {
        RtcpPayloadSpecificFeedback::Rpsi(f)
    }
}
impl From<ApplicationLayerFeedback> for RtcpPayloadSpecificFeedback {
    fn from(f: ApplicationLayerFeedback) -> Self {
        RtcpPayloadSpecificFeedback::Afb(f)
    }
}



#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GenericNack {
    pub sender_ssrc: u32,
    pub media_ssrc: u32,
    pub packet_id: u16,
    pub lost_packets_bitmask: u16,
}
impl ReadFrom for GenericNack {
    fn read_from<R: Read>(reader: &mut R) -> Result<Self> {
        let sender_ssrc = track_try!(reader.read_u32be());
        let media_ssrc = track_try!(reader.read_u32be());
        let packet_id = track_try!(reader.read_u16be());
        let lost_packets_bitmask = track_try!(reader.read_u16be());
        Ok(GenericNack {
            sender_ssrc: sender_ssrc,
            media_ssrc: media_ssrc,
            packet_id: packet_id,
            lost_packets_bitmask: lost_packets_bitmask,
        })
    }
}
impl WriteTo for GenericNack {
    fn write_to<W: Write>(&self, writer: &mut W) -> Result<()> {
        track_try!(writer.write_u32be(self.sender_ssrc));
        track_try!(writer.write_u32be(self.media_ssrc));
        track_try!(writer.write_u16be(self.packet_id));
        track_try!(writer.write_u16be(self.lost_packets_bitmask));
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PictureLossIndication {
    pub sender_ssrc: u32,
    pub media_ssrc: u32,
}
impl ReadFrom for PictureLossIndication {
    fn read_from<R: Read>(reader: &mut R) -> Result<Self> {
        let sender_ssrc = track_try!(reader.read_u32be());
        let media_ssrc = track_try!(reader.read_u32be());
        Ok(PictureLossIndication {
            sender_ssrc: sender_ssrc,
            media_ssrc: media_ssrc,
        })
    }
}
impl WriteTo for PictureLossIndication {
    fn write_to<W: Write>(&self, writer: &mut W) -> Result<()> {
        track_try!(writer.write_u32be(self.sender_ssrc));
        track_try!(writer.write_u32be(self.media_ssrc));
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SliceLossIndication {
    pub sender_ssrc: u32,
    pub media_ssrc: u32,
    pub first: u16,
    pub number: U13,
    pub picture_id: U6,
}
impl ReadFrom for SliceLossIndication {
    fn read_from<R: Read>(reader: &mut R) -> Result<Self> {
        let sender_ssrc = track_try!(reader.read_u32be());
        let media_ssrc = track_try!(reader.read_u32be());
        let first = track_try!(reader.read_u16be());
        let num_and_pic = track_try!(reader.read_u16be());
        let number = num_and_pic >> 6;
        let picture_id = (num_and_pic as u8) & 0b0011_1111;
        Ok(SliceLossIndication {
            sender_ssrc: sender_ssrc,
            media_ssrc: media_ssrc,
            first: first,
            number: number,
            picture_id: picture_id,
        })
    }
}
impl WriteTo for SliceLossIndication {
    fn write_to<W: Write>(&self, writer: &mut W) -> Result<()> {
        track_try!(writer.write_u32be(self.sender_ssrc));
        track_try!(writer.write_u32be(self.media_ssrc));
        track_try!(writer.write_u16be(self.first));
        track_try!(writer.write_u16be((self.number << 6) + (self.picture_id as u16)));
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReferencePictureSelectionIndication {
    pub sender_ssrc: u32,
    pub media_ssrc: u32,
    pub rtp_payload_type: U7,
    pub information: Vec<u8>,
}
impl ReadFrom for ReferencePictureSelectionIndication {
    fn read_from<R: Read>(reader: &mut R) -> Result<Self> {
        let sender_ssrc = track_try!(reader.read_u32be());
        let media_ssrc = track_try!(reader.read_u32be());
        let padding = track_try!(reader.read_u8());
        let rtp_payload_type = track_try!(reader.read_u8());
        track_assert_eq!(rtp_payload_type & 0b1000_0000, 0, ErrorKind::Invalid);
        let info_len = track_try!(reader.read_u16be());
        let info = track_try!(reader.read_bytes(info_len as usize));
        let _ = track_try!(reader.read_bytes(padding as usize));
        Ok(ReferencePictureSelectionIndication {
            sender_ssrc: sender_ssrc,
            media_ssrc: media_ssrc,
            rtp_payload_type: rtp_payload_type,
            information: info,
        })
    }
}
impl WriteTo for ReferencePictureSelectionIndication {
    fn write_to<W: Write>(&self, writer: &mut W) -> Result<()> {
        track_try!(writer.write_u32be(self.sender_ssrc));
        track_try!(writer.write_u32be(self.media_ssrc));

        let len = 1 + 1 + 2 + self.information.len();
        let padding_len = (4 - len % 4) % 4;
        track_try!(writer.write_u8(padding_len as u8));

        track_assert_eq!(self.rtp_payload_type & 0b1000_0000, 0, ErrorKind::Invalid);
        track_try!(writer.write_u8(self.rtp_payload_type));

        track_assert!(self.information.len() <= 0xFFFF, ErrorKind::Invalid);
        track_try!(writer.write_u16be(self.information.len() as u16));
        track_try!(writer.write_all(&self.information));

        for _ in 0..padding_len {
            track_try!(writer.write_u8(0));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApplicationLayerFeedback {
    pub sender_ssrc: u32,
    pub media_ssrc: u32,
    pub data: Vec<u8>,
}
impl ReadFrom for ApplicationLayerFeedback {
    fn read_from<R: Read>(reader: &mut R) -> Result<Self> {
        let sender_ssrc = track_try!(reader.read_u32be());
        let media_ssrc = track_try!(reader.read_u32be());
        let data = track_try!(reader.read_all_bytes());
        Ok(ApplicationLayerFeedback {
            sender_ssrc: sender_ssrc,
            media_ssrc: media_ssrc,
            data: data,
        })
    }
}
impl WriteTo for ApplicationLayerFeedback {
    fn write_to<W: Write>(&self, writer: &mut W) -> Result<()> {
        track_try!(writer.write_u32be(self.sender_ssrc));
        track_try!(writer.write_u32be(self.media_ssrc));
        track_try!(writer.write_all(&self.data));
        Ok(())
    }
}