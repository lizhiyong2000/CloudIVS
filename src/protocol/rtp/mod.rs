pub mod rtcp;
pub mod rtp;
pub mod srtp;
pub mod mutex;


pub mod constants{
    pub const RTP_VERSION: u8 = 2;
}

mod traits{
    use crate::protocol::traits::PacketTrait;

    pub trait RtpPacketTrait: PacketTrait {}
    pub trait RtcpPacketTrait: PacketTrait {}


}