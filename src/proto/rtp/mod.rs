pub mod rtp;
pub mod srtp;
pub mod mutex;


pub mod constants{
    pub const RTP_VERSION: u8 = 2;
}

pub mod traits{
    use crate::proto::traits::PacketTrait;

    pub trait RtpPacketTrait: PacketTrait {}
    pub trait RtcpPacketTrait: PacketTrait {}


}