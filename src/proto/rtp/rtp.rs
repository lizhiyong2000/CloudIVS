use std::io::{Read, Write};

use crate::proto::common::sync_io::{ReadExt, WriteExt};
use crate::proto::error::ErrorKind;
use crate::proto::error::Error;
use crate::proto::rtp::traits::RtpPacketTrait;
use crate::proto::traits::{PacketTrait, ReadFrom, ReadPacket, Result, WritePacket, WriteTo};
use crate::proto::types::U7;

use super::constants::RTP_VERSION;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RtpPacketReader;
impl ReadPacket for RtpPacketReader {
    type Packet = RtpPacket;
    fn read_packet<R: Read>(&mut self, reader: &mut R) -> Result<Self::Packet> {
        RtpPacket::read_from(reader)
    }
    fn supports_type(&self, _ty: u8) -> bool {
        true
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RtpPacketWriter;
impl WritePacket for RtpPacketWriter {
    type Packet = RtpPacket;
    fn write_packet<W: Write>(&mut self, writer: &mut W, packet: &Self::Packet) -> Result<()> {
        packet.write_to(writer)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RtpPacket {
    pub header: RtpFixedHeader,
    pub payload: Vec<u8>,
    pub padding: Vec<u8>,
}
impl PacketTrait for RtpPacket {}
impl RtpPacketTrait for RtpPacket {}
impl ReadFrom for RtpPacket {
    fn read_from<R: Read>(reader: &mut R) -> Result<Self> {
        let header = track_try_unwrap!(RtpFixedHeader::read_from(reader).map_err(Error::from));
        let mut payload = track_try_unwrap!(reader.read_all_bytes().map_err(Error::from));
        let mut padding = Vec::new();
        if header.padding {
            let payload_len = payload.len();
            track_assert_ne!(payload_len, 0, ErrorKind::Invalid);

            let padding_len = *payload.last().unwrap() as usize;
            track_assert!(padding_len <= payload_len, ErrorKind::Invalid);

            padding = payload.drain(payload_len - padding_len..).collect();
        }
        Ok(RtpPacket {
            header: header,
            payload: payload,
            padding: padding,
        })
    }
}
impl WriteTo for RtpPacket {
    fn write_to<W: Write>(&self, writer: &mut W) -> Result<()> {
        track!(self.header.write_to(writer).map_err(Error::from));
        track!(writer.write_all(&self.payload).map_err(Error::from));

        track_assert_ne!(
            self.header.padding,
            self.padding.is_empty(),
            ErrorKind::Invalid
        );
        if !self.padding.is_empty() {
            track_assert_eq!(
                *self.padding.last().unwrap() as usize,
                self.padding.len(),
                ErrorKind::Invalid
            );
            track!(writer.write_all(&self.padding).map_err(Error::from));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RtpFixedHeader {
    pub padding: bool,
    pub marker: bool,
    pub payload_type: U7,
    pub seq_num: u16,
    pub timestamp: u32,
    pub ssrc: u32,
    pub csrc_list: Vec<u32>,
    pub extension: Option<RtpHeaderExtension>,
}
impl ReadFrom for RtpFixedHeader {
    fn read_from<R: Read>(reader: &mut R) -> Result<Self> {
        let b = track_try_unwrap!(reader.read_u8().map_err(Error::from));
        track_assert_eq!(
            b >> 6,
            RTP_VERSION,
            ErrorKind::Unsupported,
            "Unsupported RTP version: {}",
            b >> 6
        );
        let padding = (b & 0b0010_0000) != 0;
        let extension = (b & 0b0001_0000) != 0;
        let csrc_count = b & 0b0000_1111;

        let b = track_try_unwrap!(reader.read_u8().map_err(Error::from));
        let marker = (b & 0b1000_0000) != 0;
        let payload_type = b & 0b0111_1111;

        let seq_num = track_try_unwrap!(reader.read_u16be().map_err(Error::from));

        //TODO

        let timestamp = track_try_unwrap!(reader.read_u32be().map_err(Error::from));
        let ssrc = track_try_unwrap!(reader.read_u32be().map_err(Error::from));
        let csrc_list = (0..csrc_count).map(|_| track_try_unwrap!(reader.read_u32be().map_err(Error::from))).collect();
        let extension = if extension {
            let e = track_try_unwrap!(RtpHeaderExtension::read_from(reader).map_err(Error::from));
            Some(e)
        } else {
            None
        };
        Ok(RtpFixedHeader {
            padding: padding,
            extension: extension,
            marker: marker,
            payload_type: payload_type,
            seq_num: seq_num,
            timestamp: timestamp,
            ssrc: ssrc,
            csrc_list: csrc_list,
        })
    }
}
impl WriteTo for RtpFixedHeader {
    fn write_to<W: Write>(&self, writer: &mut W) -> Result<()> {
        let mut b = RTP_VERSION << 6;
        if self.padding {
            b |= 0b0010_0000;
        }
        if self.extension.is_some() {
            b |= 0b0001_0000;
        }
        track_assert!(self.csrc_list.len() <= 0b0000_1111, ErrorKind::Invalid);
        b |= self.csrc_list.len() as u8;
        track!(writer.write_u8(b).map_err(Error::from));

        let mut b = 0;
        if self.marker {
            b |= 0b1000_0000;
        }
        b |= self.payload_type;
        track!(writer.write_u8(b).map_err(Error::from));

        track!(writer.write_u16be(self.seq_num).map_err(Error::from));
        track!(writer.write_u32be(self.timestamp).map_err(Error::from));
        track!(writer.write_u32be(self.ssrc).map_err(Error::from));
        for csrc in self.csrc_list.iter() {
            track!(writer.write_u32be(*csrc).map_err(Error::from));
        }
        if let Some(ref extension) = self.extension {
            track!(extension.write_to(writer).map_err(Error::from));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RtpHeaderExtension {
    pub profile_specific: u16,
    pub extension: Vec<u8>,
}
impl ReadFrom for RtpHeaderExtension {
    fn read_from<R: Read>(reader: &mut R) -> Result<Self> {
        let profile_specific = track_try_unwrap!(reader.read_u16be().map_err(Error::from));
        let word_count = track_try_unwrap!(reader.read_u16be().map_err(Error::from));
        let extension = track_try_unwrap!(reader.read_bytes(word_count as usize * 4).map_err(Error::from));
        Ok(RtpHeaderExtension {
            profile_specific: profile_specific,
            extension: extension,
        })
    }
}
impl WriteTo for RtpHeaderExtension {
    fn write_to<W: Write>(&self, writer: &mut W) -> Result<()> {
        track_assert_eq!(self.extension.len() % 4, 0, ErrorKind::Invalid);
        track_assert!(self.extension.len() / 4 < 0x10000, ErrorKind::Invalid);

        track!(writer.write_u16be(self.profile_specific).map_err(Error::from));
        track!(writer.write_u16be((self.extension.len() / 4) as u16).map_err(Error::from));
        track!(writer.write_all(&self.extension).map_err(Error::from));
        Ok(())
    }
}