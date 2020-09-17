use crate::proto::rtsp::message::header::name::HeaderName;
use crate::proto::rtsp::message::header::types::{ContentLength, CSeq};
use crate::proto::rtsp::message::uri::Scheme;
use crate::proto::rtsp::message::request::Request;
use bytes::BytesMut;
use crate::proto::rtsp::message::header::map::HeaderMapExtension;
use crate::proto::rtsp::message::response::Response;
use crate::proto::rtsp::codec::Message;
use std::time::Duration;
// use tokio::sync::mpsc::Receiver;
use tokio::stream::StreamExt;
use futures::channel::mpsc::{UnboundedSender, Receiver};
use futures::future::Fuse;
use futures::future::FutureExt;
use futures::Future;
use futures::task::Context;
use tokio::macros::support::{Pin, Poll};

pub(crate) struct MessageHandler{
    rx_incoming_request: Receiver<(CSeq, Request<BytesMut>)>,
    tx_outgoing_message: UnboundedSender<Message>,
    continue_wait_duration: Option<Duration>,
}


impl MessageHandler{

    pub fn new(
        rx_incoming_request: Receiver<(CSeq, Request<BytesMut>)>,
        tx_outgoing_message: UnboundedSender<Message>,
        continue_wait_duration: Option<Duration>,
    ) -> Self {
        MessageHandler {
            rx_incoming_request: rx_incoming_request,
            tx_outgoing_message,
            continue_wait_duration
        }
    }

    fn process_request(&mut self, cseq: CSeq, request: Request<BytesMut>) {
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
        // debug_assert!(!self.should_shutdown());

        // Ignore any responses without a `"CSeq"` header or without a corresponding pending
        // request.
        //
        // It is possible the oneshot receiver has been dropped before we can send a
        // response. If the pending request future was being polled on a separate thread
        // then the response receiver, the pending request future may have closed the
        // receiver and sent a new pending request update to cancel the request.

        // if let Some(cseq) = response.headers().typed_get::<CSeq>() {
        //     if response.status_code() == StatusCode::Continue {
        //         if let Some(pending_request) = self.pending_requests.get_mut(&cseq) {
        //             let (tx_pending_request, rx_pending_request) = oneshot::channel();
        //
        //             if mem::replace(pending_request, tx_pending_request)
        //                 .send(PendingRequestResponse::Continue(rx_pending_request))
        //                 .is_err()
        //             {
        //                 self.pending_requests.remove(&cseq);
        //             }
        //         }
        //     } else if let Some(pending_request) = self.pending_requests.remove(&cseq) {
        //         let _ = pending_request.send(PendingRequestResponse::Response(response));
        //     }
        // }
    }


    fn send_response(&mut self, cseq: CSeq, mut response: Response<BytesMut>) {
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

}


impl Future for MessageHandler
    // where
    //     TStream: Stream<Item = Result<Message, ProtocolError>> + Send + 'static,
{
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        println!("{}", "message handler poll");
        unimplemented!()
    }
}