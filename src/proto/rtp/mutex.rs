use std::io::{Read, Write};

use crate::proto::error::ErrorKind;
use crate::proto::error::Error;
use crate::proto::rtp::traits::{RtcpPacketTrait, RtpPacketTrait};
use crate::proto::traits::{PacketTrait, ReadPacket, Result, WritePacket};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MuxPacketReader<T, U>
    where
        T: ReadPacket,
        T::Packet: RtpPacketTrait,
        U: ReadPacket,
        U::Packet: RtcpPacketTrait,
{
    rtp_reader: T,
    rtcp_reader: U,
}
impl<T, U> MuxPacketReader<T, U>
    where
        T: ReadPacket,
        T::Packet: RtpPacketTrait,
        U: ReadPacket,
        U::Packet: RtcpPacketTrait,
{
    pub fn new(rtp_reader: T, rtcp_reader: U) -> Self {
        MuxPacketReader {
            rtp_reader: rtp_reader,
            rtcp_reader: rtcp_reader,
        }
    }
}
impl<T, U> ReadPacket for MuxPacketReader<T, U>
    where
        T: ReadPacket,
        T::Packet: RtpPacketTrait,
        U: ReadPacket,
        U::Packet: RtcpPacketTrait,
{
    type Packet = MuxedPacket<T::Packet, U::Packet>;
    fn read_packet<R: Read>(&mut self, reader: &mut R) -> Result<Self::Packet> {
        let mut buf = [0; 2];
        track!(reader.read_exact(&mut buf).map_err(Error::from));

        let ty = buf[1];
        if self.rtcp_reader.supports_type(ty) {
            let reader = &mut (&buf[..]).chain(reader);
            track_err!(self.rtcp_reader.read_packet(reader).map(MuxedPacket::Rtcp))
        } else {
            let reader = &mut (&buf[..]).chain(reader);
            track_err!(self.rtp_reader.read_packet(reader).map(MuxedPacket::Rtp))
        }
    }
    fn supports_type(&self, ty: u8) -> bool {
        self.rtp_reader.supports_type(ty) || self.rtcp_reader.supports_type(ty)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MuxPacketWriter<T, U>
    where
        T: WritePacket,
        T::Packet: RtpPacketTrait,
        U: WritePacket,
        U::Packet: RtcpPacketTrait,
{
    rtp_writer: T,
    rtcp_writer: U,
}
impl<T, U> MuxPacketWriter<T, U>
    where
        T: WritePacket,
        T::Packet: RtpPacketTrait,
        U: WritePacket,
        U::Packet: RtcpPacketTrait,
{
    pub fn new(rtp_writer: T, rtcp_writer: U) -> Self {
        MuxPacketWriter {
            rtp_writer: rtp_writer,
            rtcp_writer: rtcp_writer,
        }
    }
}
impl<T, U> WritePacket for MuxPacketWriter<T, U>
    where
        T: WritePacket,
        T::Packet: RtpPacketTrait,
        U: WritePacket,
        U::Packet: RtcpPacketTrait,
{
    type Packet = MuxedPacket<T::Packet, U::Packet>;
    fn write_packet<W: Write>(&mut self, writer: &mut W, packet: &Self::Packet) -> Result<()> {
        match *packet {
            MuxedPacket::Rtp(ref p) => self.rtp_writer.write_packet(writer, p),
            MuxedPacket::Rtcp(ref p) => self.rtcp_writer.write_packet(writer, p),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MuxedPacket<T, U> {
    Rtp(T),
    Rtcp(U),
}
impl<T, U> PacketTrait for MuxedPacket<T, U>
    where
        T: RtpPacketTrait,
        U: RtcpPacketTrait,
{
}