use std::io::{Read, Write};
use handy_async::sync_io::{ReadExt, WriteExt};

use super::constants::*;
use super::traits::*;
use super::types::*;
use super::rtcp_packet::*;




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