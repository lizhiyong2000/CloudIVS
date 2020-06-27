use std::io::{Read, Write};
use handy_async::sync_io::{ReadExt, WriteExt};

use super::constants::*;
use super::traits::*;
use super::types::*;
use super::rtcp_packet::*;


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