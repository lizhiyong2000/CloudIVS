use std::time::{Duration, Instant};

use bytes::BytesMut;
use futures::{Future, StreamExt};
use futures::channel::mpsc::{Sender, UnboundedReceiver};
use futures::task::Context;
use tokio::macros::support::{Pin, Poll};
use tokio::stream::Stream;

use log::{info, error};

use crate::proto::rtsp::codec::{CodecEvent, Message, ProtocolError};
use crate::proto::rtsp::message::header::types::CSeq;
use crate::proto::rtsp::message::request::Request;
use crate::proto::rtsp::connection::pending::PendingRequestUpdate;
use crate::proto::rtsp::connection::handler::MessageHandler;
use std::fmt::{Display, Formatter};
use std::fmt;
use std::error::Error;
// use crate::proto::rtsp::codec::Message::{Request as MessageReque};

pub struct MessageReceiver<TStream>
    where
        TStream: Stream<Item = Result<Message, ProtocolError>> + Send + Unpin + 'static,
{

    /// The underlying connection stream from which messages are read and decoded from. This stream
    /// must represent an ordered, reliable protocol (e.g. TCP).
    stream: TStream,

    /// A stream of codec events used to reset the decoding timer.
    rx_codec_event: UnboundedReceiver<CodecEvent>,


    tx_incoming_request: Sender<(CSeq, Request<BytesMut>)>,

    message_handler: Option<MessageHandler>,
    /// How long should we wait before decoding is timed out and the connection is dropped.
    decode_timeout_duration: Duration,
    //
    // /// Are requests allowed to be accepted.
    // requests_allowed: bool,


}

impl <TStream> MessageReceiver<TStream>
    where
        TStream: Stream<Item = Result<Message, ProtocolError>> + Send + Unpin + 'static,
{

    /// Constructs a new receiver.
    pub fn new(
        stream: TStream,
        rx_codec_event: UnboundedReceiver<CodecEvent>,
        tx_incoming_request: Sender<(CSeq, Request<BytesMut>)>,
        message_handler: Option<MessageHandler>,
        decode_timeout_duration: Duration,
    ) -> Self {
        MessageReceiver {
            stream,
            rx_codec_event,
            tx_incoming_request,
            message_handler,
            decode_timeout_duration,
            // requests_allowed: true,
        }
    }

    fn handle_codec_event(&mut self, event: CodecEvent) {
        use self::CodecEvent::*;

        match event {
            DecodingStarted => {
                let expire_time = Instant::now() + self.decode_timeout_duration;
                // self.decoding_timer = Some(Delay::new(expire_time));
            }
            DecodingEnded => {
                // self.decoding_timer = None;
            }
            _ => {}
        }
    }
    /// Checks for new codec events.
    ///
    /// If `Ok(Async::Ready(()))` is never returned.
    ///
    /// If `Ok(Async::NotReady)` is returned, then there are no more codec events to be processed.
    ///
    /// If `Err(())` is never returned.
    // pub fn poll_codec_events(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), ()>> {
    //     loop {
    //         match self.as_mut()
    //             .rx_codec_event.poll_next_unpin(cx)
    //             // .expect("`Receiver.rx_codec_event` should not error")
    //         {
    //
    //             Poll::Ready(Some(event)) =>{
    //                 self.handle_codec_event(event)
    //             },
    //             Poll::Pending =>{
    //                 return Poll::Pending
    //             },
    //             _ =>{
    //                 return Poll::Pending
    //             }
    //             // Async::Ready(Some(event)) => self.handle_codec_event(event),
    //             // Async::NotReady => return Ok(Async::NotReady),
    //             // Async::Ready(None) => panic!("`Receiver.rx_codec_event` should not end"),
    //         }
    //     }
    // }


    /// Checks if there are any messages to be processed from the internal connection stream.
///
/// If `Ok(Async::Ready(()))` is returned, then the stream has been closed and no more messages
/// will be received.
///
/// If `Ok(Async::NotReady)` is returned, then either there are no more messages to be processed
/// from the stream currently, or no messages can currently be accepted.
///
/// If `Err(`[`ProtocolError`]`)` is returned, then there was a protocol error while trying to
/// poll the stream.
    pub fn poll_stream(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), ProtocolError>> {

        if let Some(message_handler) = self.message_handler.as_ref() {
            // If the forwarding receiver is full, then any incoming requests cannot be
            // handled. This also blocks any incoming responses, since we have to process
            // messages as they come.
            if message_handler.is_full() {
                // self.stream = Some(stream);
                return Poll::Pending;
            }
        }


        let stream_result = self.stream.poll_next_unpin(cx);
        match stream_result {
            Poll::Ready(Some(result)) => {
                match result{
                    Ok(message) => {

                        info!("Message recieved");

                        if let Some(message_handler) = self.message_handler.as_mut() {
                            message_handler.handle_message(message);
                        }

                        // if let Message::Request(request) = message.clone(){
                        //     info!("Message recieved:{}", "request");
                        // }
                        // if let Message::Response(response) = message.clone(){
                        //     info!("Message recieved:{}", "response");
                        // }



                        // if let Err(error) = self.as_mut().handle_message(message) {
                        //     // self.as_mut().handle_request_receiver_error(error);
                        // }
                    },

                    Err(p) =>{
                        error!("poll_stream error:{}", p);
                        return Poll::Ready(Err(p))
                    }

                }
            }
            Poll::Pending => {
                // self.stream = Some(stream);
                return Poll::Pending;
            }
            Poll::Ready(None) => return Poll::Ready(Ok(())),
            // Poll::Ready(Err(error)) => {
            //     self.handle_protocol_error(&error);
            //     return Poll::Ready(Err(error));
            // }
        }



        Poll::Pending
        // let s = self.stream.take(1);
        // match s
        // {
        //
        //     // Ok(Async::Ready(Some(message))) => {
        //     //     if let Err(error) = self.handle_message(message) {
        //     //         self.handle_request_receiver_error(error);
        //     //     }
        //     // }
        //     // Ok(Async::NotReady) => {
        //     //     self.stream = Some(stream);
        //     //     return Ok(Async::NotReady);
        //     // }
        //     // Ok(Async::Ready(None)) => return Ok(Async::Ready(())),
        //     // Err(error) => {
        //     //     self.handle_protocol_error(&error);
        //     //     return Err(error);
        //     // }
        // }

    }
}

impl <TStream> Future for MessageReceiver<TStream>
    where
        TStream: Stream<Item = Result<Message, ProtocolError>> + Send + Unpin + 'static,
{
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        info!("message receiver poll");
        match self.poll_stream(cx) {
            Poll::Ready(Ok(message))=> {
                // match message{
                //     Message::Request(request) =>{
                //         info!("request received");
                //     },
                //     Message::Response(request) =>{
                //         info!("response received");
                //     },
                //     _ => {}
                // }
                info!("message received ok");
            },

            Poll::Ready(Err(err))=> {
                info!("message received error");
            },

            // Ok(Async::Ready(_)) | Err(_) => {
            //     self.shutdown_receiving();
            // }
            _ => (),
        }

        // self.poll_codec_events(cx);

        Poll::Pending
    }
}



/// Error that may be returned when processing incoming requests.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum RequestReceiverError {
    /// Some part of the request is invalid and cannot be processed.
    BadRequest,

    /// The difference in the next expected `"CSeq"` and the request's `"CSeq"` was too large to
    /// internally buffer.
    CSeqDifferenceTooLarge,
}

impl Display for RequestReceiverError {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        use self::RequestReceiverError::*;

        match self {
            BadRequest => write!(formatter, "bad request"),
            CSeqDifferenceTooLarge => write!(formatter, "CSeq difference too large"),
        }
    }
}

impl Error for RequestReceiverError {}