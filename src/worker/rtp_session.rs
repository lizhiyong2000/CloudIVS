use std::net::SocketAddr;
use futures::stream::SplitStream;
use futures::stream::SplitSink;
use tokio_util::codec::{Decoder, Encoder, Framed};
use crate::proto::rtp::rtp::RtpPacket;
use futures::{Future, StreamExt, Stream, SinkExt};
use futures::task::{Context, Poll};
use tokio::macros::support::Pin;
use tokio::net::{UdpSocket, TcpStream};
use std::io;
// use tokio_util::udp::UdpFramed;
use crate::proto::common::UdpFramed;
use crate::proto::common::TransportProtocol;
use crate::proto::rtp::codec::Codec;
use crate::proto::rtp::mutex::MuxedPacket;
use crate::proto::rtcp::rtcp_packet::{RtcpCompoundPacket, RtcpPacket};
use crate::proto::rtsp::codec::ProtocolError;

use log::{info, error};
use crate::proto::rtsp::message::status::StatusCode::PaymentRequired;
use std::sync::Arc;
use futures::io::Error;

pub struct RTPSession
{
    local_addr:Option<SocketAddr>,
    remote_addr:Option<SocketAddr>,

    protocol_type: TransportProtocol,

    // udp_socket: Option<Box<UdpSocket>>,
    // tcp_stream: Option<TcpStream>,

    stream:Option<SplitStream<UdpFramed<Codec>>>,
    sink: Option<SplitSink<UdpFramed<Codec>, (MuxedPacket<RtpPacket, RtcpCompoundPacket>, SocketAddr)>>,


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
                // udp_socket:None,
                // tcp_stream:None,
                stream:None,
                sink:None
            }
    }


    pub async fn connect(&mut self) -> io::Result<()>
    {
        //
        // if let Some(udp) = self.udp_socket.as_mut(){
        //     return Ok(());
        // }

        let udpsocket = UdpSocket::bind(self.local_addr.unwrap()).await;


        match udpsocket{
            Err(e)=> Err(e),
            Ok(udp) => {
                udp.connect(self.remote_addr.unwrap()).await;
                // self.udp_socket = Some(udp);
                let (sink, stream ) = UdpFramed::new(udp, Codec::new()).split();
                self.stream = Some(stream);
                self.sink = Some(sink);
                // let Some(socket) = self.udp_socket.as_mut();
                // socket.connect(self.remote_addr.unwrap()).await
                return Ok(());
            }
        }



    }


    pub fn poll_stream(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {

        let stream_result = self.stream.as_mut().unwrap().poll_next_unpin(cx);
        match stream_result {
            Poll::Ready(Some(result)) => {
                match result{
                    Ok(message) => {

                        // info!("RTP Message recieved:{:?} from {}", message.0, message.1);
                    },

                    Err(p) =>{
                        error!("poll_stream error:{}", p);
                        // return Poll::Ready(Err(ProtocolError::IO(Arc::new(Error("")))))
                    }

                }
            }
            Poll::Pending => {
                // self.stream = Some(stream);
                error!("poll_stream pending");
            }
            Poll::Ready(None) => {
                error!("poll_stream ready none");
            }
        }
        // match stream_result {
        //     Poll::Ready(Some(result)) => {
        //         match result{
        //             Ok(message) => {
        //
        //                 info!("Message recieved");
        //
        //                 if let Some(message_handler) = self.message_handler.as_mut() {
        //                     message_handler.handle_message(message);
        //                 }
        //             },
        //
        //             Err(p) =>{
        //                 error!("poll_stream error:{}", p);
        //                 return Poll::Ready(Err(p))
        //             }
        //
        //         }
        //     }
        //     Poll::Pending => {
        //         // self.stream = Some(stream);
        //         return Poll::Pending;
        //     }
        //     Poll::Ready(None) => return Poll::Ready(Ok(())),
        //     // Poll::Ready(Err(error)) => {
        //     //     self.handle_protocol_error(&error);
        //     //     return Poll::Ready(Err(error));
        //     // }
        // }



        Poll::Pending


    }

}

impl Future for RTPSession
{
    type Output = Result<(), ProtocolError>;



    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {

        info!("RTPSession poll");
        // self.stream.as_mut().unwrap().poll_next_unpin(cx);
        // self.stream.
        loop{
            match self.as_mut().poll_stream(cx) {
                // Poll::Ready(Ok(message))=> {
                //     // match message{
                //     //     Message::Request(request) =>{
                //     //         info!("request received");
                //     //     },
                //     //     Message::Response(request) =>{
                //     //         info!("response received");
                //     //     },
                //     //     _ => {}
                //     // }
                //     info!("message received ok");
                // },

                Poll::Ready(())=> {
                    info!("message received error");
                },

                // Ok(Async::Ready(_)) | Err(_) => {
                //     self.shutdown_receiving();
                // }
                _ => {
                    // info!("message received");
                    // ()
                },
            }
        }


        return Poll::Pending;
        //
        // // self.sink.as_mut().unwrap().poll_ready_unpin(cx);
        // let stream_result = self.stream.as_mut().unwrap().poll_next_unpin(cx);
        // match stream_result {
        //     Poll::Ready(Some(result)) => {
        //         match result{
        //             Ok(message) => {
        //
        //                 info!("RTP Message recieved:{:?} from {}", message.0, message.1);
        //
        //                 // if let Some(message_handler) = self.message_handler.as_mut() {
        //                 //     message_handler.handle_message(message);
        //                 // }
        //
        //                 // if let Message::Request(request) = message.clone(){
        //                 //     info!("Message recieved:{}", "request");
        //                 // }
        //                 // if let Message::Response(response) = message.clone(){
        //                 //     info!("Message recieved:{}", "response");
        //                 // }
        //
        //
        //
        //                 // if let Err(error) = self.as_mut().handle_message(message) {
        //                 //     // self.as_mut().handle_request_receiver_error(error);
        //                 // }
        //                 // return Poll::Ready(Ok(message.0))
        //                 return Poll::Pending
        //             },
        //
        //             Err(p) =>{
        //                 error!("poll_stream error:{}", p);
        //                 // return Poll::Ready(Err(ProtocolError::IO(Arc::new(Error("")))))
        //                 return Poll::Pending;
        //             }
        //
        //         }
        //     }
        //     Poll::Pending => {
        //         // self.stream = Some(stream);
        //         error!("poll_stream pending");
        //         return Poll::Pending;
        //     }
        //     Poll::Ready(None) => {
        //         error!("poll_stream ready none");
        //         return Poll::Pending
        //     }
        // }
    }
}