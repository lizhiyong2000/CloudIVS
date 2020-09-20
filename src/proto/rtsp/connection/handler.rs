use std::time::Duration;
use std::str;

use bytes::{BytesMut, Buf};
use futures::channel::mpsc::{Receiver, UnboundedSender, UnboundedReceiver};
use futures::{Future, StreamExt};
// use futures::future::Fuse;
use futures::future::FutureExt;
use futures::task::Context;
use tokio::macros::support::{Pin, Poll};
// use tokio::sync::mpsc::Receiver;
// use tokio::stream::StreamExt;

use log::{info, error};

use fnv::FnvBuildHasher;

use crate::proto::rtsp::codec::{Message, Codec, ProtocolError};
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
use crate::proto::rtsp::message::uri::Scheme;
// use crate::proto::rtsp::message::uri::Scheme;
use crate::proto::rtsp::message::response::{NOT_IMPLEMENTED_RESPONSE, BAD_REQUEST_RESPONSE};
use crate::proto::rtsp::connection::sender::SenderHandle;
use crate::proto::rtsp::message::header::types::authenticate::WWWAuthenticate;

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

    sender_handle: Option<SenderHandle>,

    www_authenticate: Option<WWWAuthenticate>,


}


impl MessageHandler{

    pub fn new(
        rx_incoming_request: Receiver<(CSeq, Request<BytesMut>)>,
        rx_pending_request: UnboundedReceiver<PendingRequestUpdate>,
        continue_wait_duration: Option<Duration>,
        request_buffer_size: usize,
        sender_handle: SenderHandle,
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
            requests_allowed: true,
            sender_handle: Some(sender_handle),
            www_authenticate:None,
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
            }
            else if let Some(pending_request) = self.pending_requests.remove(&cseq) {

                info!("send PendingRequestResponse::Response: {} ", cseq);
                let _ = pending_request.send(PendingRequestResponse::Response(response.clone()));


            }
        }
    }


    fn send_response(&mut self, cseq: CSeq, mut response: Response<BytesMut>) {
        response.headers_mut().typed_insert(cseq);

        if let Some(sender_handle) = self.sender_handle.as_mut() {
            if sender_handle
                .try_send_message(Message::Response(response))
                .is_err()
            {
                // The receive has been dropped implying no more responses can be sent. We'll still
                // process all incoming requests, but since no more responses can be sent, no more
                // requests should be handled other than the one already queued.
                self.sender_handle = None;
            }
        }
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


    /// Handles a pending request updates.
    ///
    /// If the update is the addition of a pending request, the request, along with its `"CSeq"`
    /// value, will be stored awaiting the corresponding response.
    ///
    /// If the update is the removal of a pending request, the request with the given `"CSeq"` is
    /// removed and no response will be matched even if it does come at a later time.
    fn handle_pending_request_update(&mut self, update: PendingRequestUpdate) {
        use self::PendingRequestUpdate::*;

        match update {
            AddPendingRequest((cseq, tx_pending_request)) => {
                info!("to AddPendingRequest :{}", cseq);
                debug_assert!(!self.pending_requests.contains_key(&cseq));
                self.pending_requests.insert(cseq, tx_pending_request);
                info!("AddPendingRequest success:{}", cseq);
                println!("{:?}", self.pending_requests);
                info!("pending_requests size :{}", self.pending_requests.len());
            }
            RemovePendingRequest(cseq) => {
                info!("to RemovePendingRequest :{}", cseq);
                println!("{:?}", self.pending_requests);
                info!("pending_requests size :{}", self.pending_requests.len());
                debug_assert!(self.pending_requests.contains_key(&cseq));
                self.pending_requests.remove(&cseq);

                info!("RemovePendingRequest success:{}", cseq);
            }
        }
    }

    /// Handles incoming pending request updates.
    ///
    /// A pending request update is either the addition of a pending request or the removal of a
    /// pending request (probably due a timeout).
    ///
    /// If `Poll::Ready(Ok(()))` is returned, then the pending request update stream has ended and
    /// the response receiver is shutdown.
    ///
    /// If `Poll::Pending` is returned, then there are no pending request updates to be
    /// processed currently.
    ///
    /// The error `Err(())` will never be returned.
    fn poll_pending_request(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), ProtocolError>>{
        loop {
            match self
                .rx_pending_request
                .poll_next_unpin(cx)
            // .expect("`ResponseReceiver.rx_pending_request` should not error")
            {
                Poll::Ready(Some(update)) => self.as_mut().handle_pending_request_update(update),
                Poll::Pending => return Poll::Pending,
                Poll::Ready(None) => {
                    // If the pending request stream has ended, this means there should be no
                    // pending requests. If there were pending requests, they could never expire
                    // because the stream used to remove them has ended. So, we assume that it
                    // cannot happen.
                    debug_assert!(self.pending_requests.is_empty());
                    return Poll::Ready(Ok(()));
                }
            }
        }
    }


    //
    // /// Tries to forward any ready requests to the request handler.
    // ///
    // /// If `Poll::Ready(Ok(()))` is returned, then all requests that could have been forwarded have
    // /// been forwarded.
    // ///
    // /// If `Poll::Pending` is returned, then channel between the forwarding receiver and the
    // /// request handler is full, and forwarding will have to be tried again later.
    // ///
    // /// If `Err(())` is returned, then the request handler's receiver has been dropped meaning the
    // /// forwarding receiver can be shutdown.
    //
    // fn poll_incoming_request(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), ProtocolError>>{
    //     if let Some(mut incoming_sequence_number) = self.incoming_sequence_number {
    //         // let test = self.tx_incoming_request;
    //         // test.
    //
    //         while let Some(request) = self.buffered_requests.remove(&incoming_sequence_number) {
    //             match self
    //                 .tx_incoming_request
    //                 .try_send((incoming_sequence_number, request.clone()))
    //             // .map_err(|_| ())?
    //             {
    //                 Ok(_) => {
    //                     incoming_sequence_number = incoming_sequence_number.wrapping_increment()
    //                 }
    //                 Err(_) => {
    //                     self.buffered_requests
    //                         .insert(incoming_sequence_number, request);
    //                     self.incoming_sequence_number = Some(incoming_sequence_number);
    //                     return Poll::Pending;
    //                 }
    //                 _ => {}
    //             }
    //         }
    //
    //         self.incoming_sequence_number = Some(incoming_sequence_number);
    //     }
    //
    //     Poll::Ready(Ok(()))
    // }
}


impl Future for MessageHandler
    // where
    //     TStream: Stream<Item = Result<Message, ProtocolError>> + Send + 'static,
{
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        info!("message handler poll");

        self.as_mut().poll_pending_request(cx);



        if let Some(mut incoming_sequence_number) = self.incoming_sequence_number {
            // let test = self.tx_incoming_request;
            // test.

            while let Some(request) = self.buffered_requests.remove(&incoming_sequence_number) {

                if request.uri().scheme() == Some(Scheme::RTSPU) {
                    self.send_response(incoming_sequence_number, NOT_IMPLEMENTED_RESPONSE.clone());
                    // return;
                }

                match request.headers().typed_get::<ContentLength>() {
                    Some(content_length)
                    if *content_length > 0
                        && !request.headers().contains_key(&HeaderName::ContentType) =>
                        {
                            self.send_response(incoming_sequence_number, BAD_REQUEST_RESPONSE.clone());
                        }
                    _ => {
                        // self.reset_continue_timer();
                        // self.serviced_request = Some((incoming_sequence_number, self.service.call(request)));
                    }
                }


                // match self
                //     .tx_incoming_request
                //     .try_send((incoming_sequence_number, request.clone()))
                // // .map_err(|_| ())?
                // {
                //     Ok(_) => {
                //         incoming_sequence_number = incoming_sequence_number.wrapping_increment()
                //     }
                //     Err(_) => {
                //         self.buffered_requests
                //             .insert(incoming_sequence_number, request);
                //         self.incoming_sequence_number = Some(incoming_sequence_number);
                //         return Poll::Pending;
                //     }
                //     _ => {}
                // }
            }

            self.incoming_sequence_number = Some(incoming_sequence_number);
        }

        // Poll::Ready(Ok(()))


        Poll::Pending
    }


}