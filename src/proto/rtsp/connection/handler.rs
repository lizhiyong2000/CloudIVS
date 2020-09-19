use std::time::Duration;
use std::str;

use bytes::{BytesMut, Buf};
use futures::channel::mpsc::{Receiver, UnboundedSender, UnboundedReceiver};
use futures::Future;
// use futures::future::Fuse;
use futures::future::FutureExt;
use futures::task::Context;
use tokio::macros::support::{Pin, Poll};
// use tokio::sync::mpsc::Receiver;
use tokio::stream::StreamExt;

use log::{info, error};

use fnv::FnvBuildHasher;

use crate::proto::rtsp::codec::{Message, Codec};
use crate::proto::rtsp::message::header::map::HeaderMapExtension;
use crate::proto::rtsp::message::header::name::HeaderName;
use crate::proto::rtsp::message::header::types::{ContentLength, CSeq};
use crate::proto::rtsp::message::request::Request;
use crate::proto::rtsp::message::response::Response;
use crate::proto::rtsp::connection::pending::{PendingRequestUpdate, PendingRequestResponse};
use crate::proto::rtsp::message::status::StatusCode;
use futures::channel::oneshot;
use std::mem;
use std::collections::HashMap;
use std::collections::hash_map::Entry;
use crate::proto::rtsp::connection::receiver::RequestReceiverError;
use tokio_util::codec::Encoder;
// use crate::proto::rtsp::message::uri::Scheme;

pub struct MessageHandler{
    rx_incoming_request: Receiver<(CSeq, Request<BytesMut>)>,
    rx_pending_request: UnboundedReceiver<PendingRequestUpdate>,
    continue_wait_duration: Option<Duration>,

    buffered_requests: HashMap<CSeq, Request<BytesMut>, FnvBuildHasher>,

    /// The expected sequence number for the next incoming request. This will be [`Option::None`] in
    /// the case where we have yet to receive a request, since it is the client that determines the
    /// initial `"CSeq"`.
    incoming_sequence_number: Option<CSeq>,

    /// The capacity of the buffer map.
    request_buffer_size: usize,

    /// A map of pending requests sent by this agent that are awaiting responses from the end-agent.
    pending_requests: HashMap<CSeq, oneshot::Sender<PendingRequestResponse>, FnvBuildHasher>,

    /// Are requests allowed to be accepted.
    requests_allowed: bool,


}


impl MessageHandler{

    pub fn new(
        rx_incoming_request: Receiver<(CSeq, Request<BytesMut>)>,
        rx_pending_request: UnboundedReceiver<PendingRequestUpdate>,
        continue_wait_duration: Option<Duration>,
        request_buffer_size: usize,
    ) -> Self {
        MessageHandler {
            rx_incoming_request,
            rx_pending_request,
            continue_wait_duration,
            buffered_requests: HashMap::with_capacity_and_hasher(
                request_buffer_size,
                FnvBuildHasher::default(),
            ),
            incoming_sequence_number: None,
            request_buffer_size,
            pending_requests: HashMap::with_hasher(FnvBuildHasher::default()),

            /// Are requests allowed to be accepted.
            requests_allowed: true
        }
    }


    /// Processes the given message.
     ///
     /// If it is a request, it will be buffered internally until it is ready to be forwarded to the
     /// request handler.
     ///
     /// If it is a response, it will be matched against a pending request with the same `"CSeq"` (if
     /// it exists).
    pub(crate) fn handle_message(&mut self, message: Message) -> Result<(), RequestReceiverError> {
        match message {
            Message::Request(request) => {
                if self.requests_allowed {
                    self.process_request(request)?;
                }
            }
            Message::Response(response) => {

                    self.process_response(response);
            }
        }

        Ok(())
    }

    fn process_request(&mut self, request: Request<BytesMut>) -> Result<(), RequestReceiverError>{

        let mut codec = Codec::new();
        let mut buffer = BytesMut::new();
        codec.encode(Message::Request(request.clone()), &mut buffer).unwrap();
        info!("process_request: {} ", str::from_utf8(&buffer).unwrap());

        match request.headers().typed_get::<CSeq>() {
            Some(cseq) => {
                let incoming_sequence_number = self.incoming_sequence_number_or_default(cseq);

                if *(cseq - incoming_sequence_number) > self.request_buffer_size as u32 {
                    Err(RequestReceiverError::CSeqDifferenceTooLarge)
                } else {
                    debug_assert!(self.buffered_requests.len() < self.request_buffer_size);

                    match self.buffered_requests.entry(cseq) {
                        Entry::Occupied(_) => Err(RequestReceiverError::BadRequest),
                        Entry::Vacant(entry) => {
                            entry.insert(request);
                            Ok(())
                        }
                    }
                }
            }
            None => Err(RequestReceiverError::BadRequest),
        }
        // if request.uri().scheme() == Some(Scheme::RTSPU) {
        //     self.send_response(cseq, NOT_IMPLEMENTED_RESPONSE.clone());
        //     return;
        // }
        //
        // match request.headers().typed_get::<ContentLength>() {
        //     Some(content_length)
        //     if *content_length > 0
        //         && !request.headers().contains_key(&HeaderName::ContentType) =>
        //         {
        //             self.send_response(cseq, BAD_REQUEST_RESPONSE.clone());
        //         }
        //     _ => {
        //         self.reset_continue_timer();
        //         self.serviced_request = Some((cseq, self.service.call(request)));
        //     }
        // }
    }


    pub fn process_response(&mut self, response: Response<BytesMut>) {

        let mut codec = Codec::new();
        let mut buffer = BytesMut::new();
        codec.encode(Message::Response(response.clone()), &mut buffer).unwrap();
        info!("process_response: {} ", str::from_utf8(&buffer).unwrap());

        // debug_assert!(!self.should_shutdown());

        // Ignore any responses without a `"CSeq"` header or without a corresponding pending
        // request.
        //
        // It is possible the oneshot receiver has been dropped before we can send a
        // response. If the pending request future was being polled on a separate thread
        // then the response receiver, the pending request future may have closed the
        // receiver and sent a new pending request update to cancel the request.

        if let Some(cseq) = response.headers().typed_get::<CSeq>() {
            // info!("process_response, cseq:{} status code:{} ", , response.status_code());


            if response.status_code() == StatusCode::Continue {
                if let Some(pending_request) = self.pending_requests.get_mut(&cseq) {
                    let (tx_pending_request, rx_pending_request) = oneshot::channel();

                    if mem::replace(pending_request, tx_pending_request)
                        .send(PendingRequestResponse::Continue(rx_pending_request))
                        .is_err()
                    {
                        self.pending_requests.remove(&cseq);
                    }
                }
            } else if let Some(pending_request) = self.pending_requests.remove(&cseq) {
                let _ = pending_request.send(PendingRequestResponse::Response(response));
            }
        }
    }


    fn send_response(&mut self, cseq: CSeq, response: Response<BytesMut>) {
        // response.headers_mut().typed_insert(cseq);

        // if let Some(sender_handle) = self.sender_handle.as_mut() {
        //     if sender_handle
        //         .try_send_message(Message::Response(response))
        //         .is_err()
        //     {
        //         // The receive has been dropped implying no more responses can be sent. We'll still
        //         // process all incoming requests, but since no more responses can be sent, no more
        //         // requests should be handled other than the one already queued.
        //         self.sender_handle = None;
        //     }
        // }
    }


    /// Returns the current `"CSeq"` used for incoming requests or defaults to the given one.
    ///
    /// Before the first request has been received on a connection, we do not know what `"CSeq"` we
    /// should be looking at, so we default to whatever the first request has if we do not yet know.
    /// In this case, the internal `"CSeq"` will be set to this default.
    pub fn incoming_sequence_number_or_default(&mut self, cseq: CSeq) -> CSeq {
        match self.incoming_sequence_number {
            Some(cseq) => cseq,
            None => {
                self.incoming_sequence_number = Some(cseq);
                cseq
            }
        }
    }

    /// Returns whether the internal request buffer is full.
    pub fn is_full(&self) -> bool {
        self.buffered_requests.len() >= self.request_buffer_size
    }


}


impl Future for MessageHandler
    // where
    //     TStream: Stream<Item = Result<Message, ProtocolError>> + Send + 'static,
{
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        info!("message handler poll");
        Poll::Pending
    }
}