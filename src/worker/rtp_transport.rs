use tokio::net::UdpSocket;
use tokio::net::TcpStream;
// use tokio_util::udp::UdpFramed;
use crate::proto::common::UdpFramed;
use tokio_util::codec::Framed;
use std::net::SocketAddr;
use crate::proto::rtsp::codec::Codec;
use futures::StreamExt;

struct RTPTransport{



}

impl RTPTransport{
    pub async fn new(){
        let listening_addr = "127.0.0.1:8080".parse::<SocketAddr>().unwrap();
        let socket = UdpSocket::bind(&listening_addr).await;
        match socket{
            Ok(s)=> {
                let (sink, stream) = UdpFramed::new(s, Codec::new()).split();

            }
            Err(e) => {

            }
        }
    }

}

