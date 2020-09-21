use std::io::{Read, Write};

use crate::proto::common::sync_io::{ReadExt, WriteExt};
use crate::proto::error::ErrorKind;
use crate::proto::error::Error;
use crate::proto::rtp::traits::RtcpPacketTrait;
use crate::proto::traits::*;
use crate::proto::traits::{PacketTrait, ReadFrom, ReadPacket, Result, WritePacket, WriteTo};
use crate::proto::types::*;
use crate::proto::types::U5;

use super::constants::*;
use super::rtcp_packet::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApplicationDefinedPacket {
    pub subtype: U5,
    pub ssrc_or_csrc: u32,
    pub name: [u8; 4],
    pub data: Vec<u8>,
}
impl PacketTrait for ApplicationDefinedPacket {}
impl RtcpPacketTrait for ApplicationDefinedPacket {}
impl ReadFrom for ApplicationDefinedPacket {
    fn read_from<R: Read>(reader: &mut R) -> Result<Self> {
        let (subtype, payload) = track_try_unwrap!(read_sctp(reader, RTCP_PACKET_TYPE_APP).map_err(Error::from));
        let reader = &mut &payload[..];

        let ssrc_or_csrc = track_try_unwrap!(reader.read_u32be().map_err(Error::from));
        let mut name = [0; 4];
        track!(reader.read_exact(&mut name).map_err(Error::from));
        let data = track_try_unwrap!(reader.read_all_bytes().map_err(Error::from));
        Ok(ApplicationDefinedPacket {
            subtype: subtype,
            ssrc_or_csrc: ssrc_or_csrc,
            name: name,
            data: data,
        })
    }
}
impl WriteTo for ApplicationDefinedPacket {
    fn write_to<W: Write>(&self, writer: &mut W) -> Result<()> {
        let mut payload = Vec::new();
        track!((&mut payload).write_u32be(self.ssrc_or_csrc).map_err(Error::from));
        payload.extend(&self.name);
        payload.extend(&self.data);

        track_assert!(self.subtype <= 0b0001_1111, ErrorKind::Invalid);
        track!(write_sctp(
            writer,
            RTCP_PACKET_TYPE_APP,
            self.subtype,
            &payload
        ).map_err(Error::from));
        Ok(())
    }
}