
pub mod sync_io;

pub enum TransportProtocol{
    UDP,
    TCP
}

mod frame;
pub use self::frame::UdpFramed;

