use std::io::{Read, Write};

use crate::common::sync_io::{ReadExt, WriteExt};
use crate::protocol::error::ErrorKind;
use crate::protocol::rtp::traits::RtcpPacketTrait;
use crate::protocol::traits::*;
use crate::protocol::traits::{PacketTrait, ReadFrom, ReadPacket, Result, WritePacket, WriteTo};
use crate::protocol::types::*;

use super::constants::*;
use super::rtcp_packet::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GoodbyePacket {
    pub ssrc_csrc_list: Vec<u32>,
    pub reason: Option<String>,
}
impl GoodbyePacket {
    pub fn new() -> Self {
        GoodbyePacket {
            ssrc_csrc_list: Vec::new(),
            reason: None,
        }
    }
}
impl PacketTrait for GoodbyePacket {}
impl RtcpPacketTrait for GoodbyePacket {}
impl ReadFrom for GoodbyePacket {
    fn read_from<R: Read>(reader: &mut R) -> Result<Self> {
        let (source_count, payload) = track_try!(read_sctp(reader, RTCP_PACKET_TYPE_BYE));
        let reader = &mut &payload[..];

        let list = track_try!((0..source_count).map(|_| reader.read_u32be()).collect());
        let mut reason = None;
        if let Ok(len) = reader.read_u8() {
            reason = Some(track_try!(reader.read_string(len as usize)));
        }
        Ok(GoodbyePacket {
            ssrc_csrc_list: list,
            reason: reason,
        })
    }
}
impl WriteTo for GoodbyePacket {
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


#[cfg(test)]
mod tests {
    use crate::protocol::rtcp::constants::SDES_ITEM_TYPE_CNAME;
    use crate::protocol::rtcp::report_packet::ReceptionReport;
    use crate::protocol::traits::{ReadFrom, WriteTo};

    use super::GoodbyePacket;

    struct Setup {
        data: Vec<u8>,
        // SR values.
        ssrc1:u32,
        ssrc2:u32,
        reason: String,
    }

    impl Setup {
        fn new() -> Self {
            Self {
                data: vec![
                    0x82, 0xcb, 0x00, 0x06, // Type: 203 (Bye), Count: 2, length: 2
                    0x62, 0x42, 0x76, 0xe0, // SSRC: 0x624276e0
                    0x26, 0x24, 0x67, 0x0e, // SSRC: 0x2624670e
                    0x0e, 0x48, 0x61, 0x73, // Length: 14, Text: "Hasta la vista"
                    0x74, 0x61, 0x20, 0x6c,
                    0x61, 0x20, 0x76, 0x69,
                    0x73, 0x74, 0x61, 0x00
                ],

                ssrc1:0x624276e0,
                ssrc2:0x2624670e,
                reason: String::from("Hasta la vista") ,

            }
        }
    }


    #[test]
    fn test_bye_parse() {
        let setup = Setup::new();

        let reader = &mut &setup.data[..];
        let bye = GoodbyePacket::read_from(reader).unwrap();

        assert_eq!(bye.ssrc_csrc_list.len(), 2);


        let reason = bye.reason.as_deref();
        assert_eq!(reason.unwrap(), &setup.reason[..]);


        let ssrc1 = bye.ssrc_csrc_list.get(0).unwrap();
        let ssrc2 = bye.ssrc_csrc_list.get(1).unwrap();



        assert_eq!(*ssrc1, setup.ssrc1);
        assert_eq!(*ssrc2, setup.ssrc2);


        let serialized = bye.to_bytes().unwrap();

        assert_eq!(serialized, setup.data);

    }


    #[test]
    fn test_bye_create() {
        let setup = Setup::new();

        let bye = GoodbyePacket{
            ssrc_csrc_list: vec![setup.ssrc1, setup.ssrc2],
            reason: Some(setup.reason)
        };


        let serialized = bye.to_bytes().unwrap();
        assert_eq!(serialized, setup.data);

    }

}



