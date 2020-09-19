use futures::{Future, Sink, StreamExt, SinkExt};
use futures::channel::mpsc::{unbounded, UnboundedReceiver, UnboundedSender};
// use futures::future::Fuse;
// use futures::channel::mpsc::unbounded;
// use tokio::sync::mpsc::UnboundedReceiver;
// use tokio::stream::StreamExt;
// use futures::future::FutureExt;
use futures::task::Context;
use tokio::macros::support::{Pin, Poll};

use log::info;

use crate::proto::rtsp::codec::{Message, ProtocolError};
use crate::proto::rtsp::message::header::map::HeaderMapExtension;
use crate::proto::rtsp::message::header::types::Date;

pub struct MessageSender<TSink>
    where
        TSink: Sink<Message, Error = ProtocolError> + Send + Unpin + 'static,
{
    /// The current message that we are trying to send but cannot yet because the sink is not ready.
    buffered_message: Option<Message>,

    /// The outgoing stream of messages that are to be sent through the sink.
    rx_outgoing_message: UnboundedReceiver<Message>,

    /// The sink representing a reliable transport mechanism (e.g. TCP).
    sink: TSink,
}

impl<TSink> MessageSender<TSink>
    where
        TSink: Sink<Message, Error = ProtocolError> + Send + Unpin + 'static,
{
    pub fn new(sink: TSink, rx_outgoing_message:UnboundedReceiver<Message>) -> (Self, SenderHandle) {
        let (tx_outgoing_message, rx_outgoing_message) = unbounded();
        let sender = MessageSender {
            buffered_message: None,
            rx_outgoing_message: rx_outgoing_message,
            sink,
        };

        let sender_handle = SenderHandle(tx_outgoing_message);

        (sender, sender_handle)
    }


    /// Tries to send the given message through the internal sink.
    ///
    /// If `Poll::Ready(Ok(()))` is returned, then the message was successfully sent through the
    /// sink. It may not have been flushed yet though, this will happen at a later point.
    ///
    /// If `Poll::Pending` is returned, then the sink is unable to accept the message at this
    /// time, probably because it is full. The message will be buffered temporarily until we can try
    /// to send it through the sink again.
    ///
    /// If `Err(`[`ProtocolError`]`)` is returned, there was an error trying to send the message
    /// through the sink.
    fn try_send_message(mut self: Pin<&mut Self>, cx: &mut Context<'_>, message: Message) -> Poll<Result<(), ProtocolError>> {
        debug_assert!(self.buffered_message.is_none());

        if let Poll::Pending = self.sink.poll_ready_unpin(cx){
            return Poll::Pending;
        }


        if let result = self.sink.start_send_unpin(message.clone())? {
            self.buffered_message = Some(message);
            // return Poll::Pending;
        }

        Poll::Ready(Ok(()))
    }
}


impl <TSink> Future for MessageSender<TSink>
    where
        TSink: Sink<Message, Error = ProtocolError> + Send + Unpin + 'static,
{
    type Output = Result<(), ProtocolError>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        info!("message sender poll");

        loop {
            match self
                .rx_outgoing_message
                .poll_next_unpin(cx)
            // .expect("`Sender.rx_outgoing_message` should not error")
            {
                Poll::Ready(Some(mut message)) => {
                    match message {
                        Message::Request(ref mut request) => {
                            request.headers_mut().typed_insert(Date::new());
                        }
                        Message::Response(ref mut response) => {
                            response.headers_mut().typed_insert(Date::new());
                        }
                    }

                    self.as_mut().try_send_message(cx, message);
                }
                Poll::Pending => {
                    // ready!(self.sink.poll_flush_unpin(cx));

                    let num = match self.sink.poll_flush_unpin(cx) {
                        Poll::Ready(t) => t,
                        Poll::Pending => return Poll::Pending,
                    };


                    return Poll::Pending;
                }
                Poll::Ready(None) => return Poll::Ready(Ok(())),
            }
        }


        Poll::Pending
    }
}


#[derive(Clone, Debug)]
pub struct SenderHandle(pub(crate) UnboundedSender<Message>);

impl SenderHandle {
    pub fn try_send_message(&self, message: Message) -> Result<(), ()> {

        info!("message sended");
        self.0.unbounded_send(message).map_err(|_| ())


    }
}