use tokio::stream::Stream;
use crate::proto::rtsp::codec::{Message, ProtocolError, CodecEvent};
use tokio::net::TcpStream;
use tokio::time::Delay;
use futures::channel::mpsc::{UnboundedReceiver, Sender};
use std::time::Duration;
use crate::proto::rtsp::message::header::types::CSeq;
use crate::proto::rtsp::message::request::Request;
use bytes::BytesMut;
use futures::future::Fuse;
use futures::{FutureExt, Future};
use futures::task::Context;
use tokio::macros::support::{Pin, Poll};

pub struct MessageReceiver<TStream>
    where
        TStream: Stream<Item = Result<Message, ProtocolError>> + Send + 'static,
{

    /// The underlying connection stream from which messages are read and decoded from. This stream
    /// must represent an ordered, reliable protocol (e.g. TCP).
    stream: TStream,

    /// A stream of codec events used to reset the decoding timer.
    rx_codec_event: UnboundedReceiver<CodecEvent>,

    tx_incoming_request: Sender<(CSeq, Request<BytesMut>)>,
    /// How long should we wait before decoding is timed out and the connection is dropped.
    decode_timeout_duration: Duration,

    /// Are requests allowed to be accepted.
    requests_allowed: bool,


}

impl <TStream> MessageReceiver<TStream>
    where
        TStream: Stream<Item = Result<Message, ProtocolError>> + Send + 'static,
{

    /// Constructs a new receiver.
    pub fn new(
        stream: TStream,
        rx_codec_event: UnboundedReceiver<CodecEvent>,
        tx_incoming_request: Sender<(CSeq, Request<BytesMut>)>,
        decode_timeout_duration: Duration,
    ) -> Self {
        MessageReceiver {
            stream,
            rx_codec_event,
            tx_incoming_request,
            decode_timeout_duration,
            requests_allowed: true,
        }
    }
}

impl <TStream> Future for MessageReceiver<TStream>
    where
        TStream: Stream<Item = Result<Message, ProtocolError>> + Send + 'static,
{
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        println!("{}", "message receiver poll");
        unimplemented!()
    }
}