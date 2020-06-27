use std::io::{Read, Write};
use handy_async::sync_io::{ReadExt, WriteExt};

use super::constants::*;
use super::traits::*;
use super::types::*;
use super::rtcp_packet::*;

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