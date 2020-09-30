use std::net::SocketAddr;
use futures::stream::SplitStream;
use futures::stream::SplitSink;
use tokio_util::codec::{Decoder, Encoder, Framed};
use crate::proto::rtp::rtp::RtpPacket;
use futures::{Future, StreamExt};
use futures::task::{Context, Poll};
use tokio::macros::support::Pin;
use tokio::net::{UdpSocket, TcpStream};
use std::io;
use tokio_util::udp::UdpFramed;
use crate::proto::common::TransportProtocol;
use crate::proto::rtp::codec::Codec;
use crate::proto::rtp::mutex::MuxedPacket;
use crate::proto::rtcp::rtcp_packet::{RtcpCompoundPacket, RtcpPacket};

pub struct RTPSession
{
    local_addr:Option<SocketAddr>,
    remote_addr:Option<SocketAddr>,

    protocol_type: TransportProtocol,

    udp_socket: Option<UdpSocket>,
    // tcp_stream: Option<TcpStream>,

    stream:Option<SplitStream<UdpFramed<Codec>>>,
    sink: Option<SplitSink<UdpFramed<Codec>, (MuxedPacket<RtpPacket, RtcpCompoundPacket<RtcpPacket>>, SocketAddr)>>,


}

impl RTPSession
{
    // pub fn newTcpSession(remote_addr:SocketAddr) -> Self{
    //     RTPSession{
    //         local_addr:None,
    //         remote_addr:Some(remote_addr),
    //         protocol_type:TransportProtocol::TCP,
    //         udp_socket:None,
    //         // tcp_stream:None,
    //         stream:None,
    //         sink:None
    //     }
    //
    // }
    pub fn newSession(local_addr:SocketAddr, remote_addr:Option<SocketAddr>) -> Self{
            RTPSession{
                local_addr:Some(local_addr),
                remote_addr:remote_addr,
                protocol_type:TransportProtocol::UDP,
                udp_socket:None,
                // tcp_stream:None,
                stream:None,
                sink:None
            }
    }


    pub async fn connect(&mut self) -> io::Result<()>
    {

        if let Some(udp) = self.udp_socket.as_mut(){
            return Ok(());
        }

        let udpsocket = UdpSocket::bind(self.local_addr.unwrap()).await;
        if let Err(e) = udpsocket {
            return Err(e);
        }

        let Ok(udp) = udpsocket;
        self.udp_socket = Some(udp);
        let (sink, stream ) = UdpFramed::new(self.udp_socket.unwrap(), Codec::new()).split();
        self.stream = Some(stream);
        self.sink = Some(sink);
        let Some(socket) = self.udp_socket.as_mut();
        socket.connect(self.remote_addr.unwrap()).await

    }

}

impl Future for RTPSession
{
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        unimplemented!()
    }
}