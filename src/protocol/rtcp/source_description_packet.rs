use std::io::{Read, Write};

use crate::common::sync_io::{ReadExt, WriteExt};
use crate::protocol::error::ErrorKind;
use crate::protocol::rtcp::constants::{*};
use crate::protocol::rtp::traits::RtcpPacketTrait;
use crate::protocol::traits::*;
use crate::protocol::traits::{PacketTrait, ReadFrom, ReadPacket, Result, WritePacket, WriteTo};
use crate::protocol::types::*;

use super::constants::*;
use super::rtcp_packet::*;

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceDescriptionPacket {
    pub chunks: Vec<SdesChunk>,
}
impl SourceDescriptionPacket {
    pub fn new() -> Self {
        SourceDescriptionPacket { chunks: Vec::new() }
    }
}
impl PacketTrait for SourceDescriptionPacket {}
impl RtcpPacketTrait for SourceDescriptionPacket {}
impl ReadFrom for SourceDescriptionPacket {
    fn read_from<R: Read>(reader: &mut R) -> Result<Self> {
        let (source_count, payload) = track_try!(read_sctp(reader, RTCP_PACKET_TYPE_SDES));
        let reader = &mut &payload[..];

        let chunks = track_try!(
            (0..source_count)
                .map(|_| SdesChunk::read_from(reader))
                .collect()
        );
        Ok(SourceDescriptionPacket { chunks: chunks })
    }
}
impl WriteTo for SourceDescriptionPacket {
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
    pub ssrc_or_csrc: u32,
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
            // track_try!(writer.write_u16be(text.len() as u16));
            track_try!(writer.write_u8(text.len() as u8));

            track_try!(writer.write_all(text.as_bytes()));
            // write_bytes += 2 + text.len();
            write_bytes += 1 + text.len();
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


#[cfg(test)]
mod tests {
    use crate::protocol::rtcp::constants::SDES_ITEM_TYPE_CNAME;
    // use super::SenderReportPacket;
    // use super::ReceiverReportPacket;
    use crate::protocol::rtcp::report_packet::ReceptionReport;
    use crate::protocol::rtcp::source_description_packet::{SdesChunk, SdesItem};
    use crate::protocol::traits::{ReadFrom, WriteTo};

    use super::SourceDescriptionPacket;

    struct Setup {
        data: Vec<u8>,
        // SR values.
        ssrc: u32 ,
        itemType: u8,
        itemLength: u8,
        value: String ,
    }

    impl Setup {
        fn new() -> Self {
            Self {
                data: vec![
                    0x81, 0xca, 0x00, 0x06, // Type: 202 (SDES), Count: 1, Length: 6
                    0x9f, 0x65, 0xe7, 0x42, // SSRC: 0x9f65e742
                    0x01, 0x10, 0x74, 0x37, // Item Type: 1 (CNAME), Length: 16, Value: t7mkYnCm46OcINy/
                    0x6d, 0x6b, 0x59, 0x6e,
                    0x43, 0x6d, 0x34, 0x36,
                    0x4f, 0x63, 0x49, 0x4e,
                    0x79, 0x2f, 0x00, 0x00
                ],

                ssrc:0x9f65e742,
                itemType: SDES_ITEM_TYPE_CNAME ,
                itemLength: 16 ,
                value: String::from("t7mkYnCm46OcINy/") ,

            }
        }
    }


    #[test]
    fn test_sdes_parse() {
        let setup = Setup::new();

        let reader = &mut &setup.data[..];
        let sdp = SourceDescriptionPacket::read_from(reader).unwrap();

        assert_eq!(sdp.chunks.len(), 1);

        let chunk = sdp.chunks.get(0).unwrap();

        assert_eq!(chunk.ssrc_or_csrc, setup.ssrc);
        assert_eq!(chunk.items.len(), 1);

        let item = chunk.items.get(0).unwrap();


        assert_eq!(item.item_type(), setup.itemType);
        assert_eq!(item.text().len() as u8, setup.itemLength);
        assert_eq!(item.text(), &setup.value[..]);


        let serialized = sdp.to_bytes().unwrap();

        assert_eq!(serialized, setup.data);

    }


    #[test]
    fn test_sender_report_create() {
        let setup = Setup::new();

        let item = SdesItem::Cname(setup.value.clone());

        assert_eq!(item.item_type(), setup.itemType);
        assert_eq!(item.text().len() as u8, setup.itemLength);
        assert_eq!(item.text(), &setup.value[..]);

        let chunk = SdesChunk{
            ssrc_or_csrc:setup.ssrc,
            items: vec![item],
        };

        assert_eq!(chunk.ssrc_or_csrc, setup.ssrc);

        let sdes = SourceDescriptionPacket {
            chunks: vec![chunk]
        };

        let serialized = sdes.to_bytes().unwrap();
        assert_eq!(serialized, setup.data);

    }

}