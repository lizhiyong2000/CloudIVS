use futures::{Future, Sink};
use futures::channel::mpsc::{unbounded, UnboundedReceiver};
// use futures::future::Fuse;
// use futures::channel::mpsc::unbounded;
// use tokio::sync::mpsc::UnboundedReceiver;
// use tokio::stream::StreamExt;
// use futures::future::FutureExt;
use futures::task::Context;
use tokio::macros::support::{Pin, Poll};

use crate::proto::rtsp::codec::{Message, ProtocolError};

pub struct MessageSender<TSink>
    where
        TSink: Sink<Message, Error = ProtocolError> + Send + 'static,
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
        TSink: Sink<Message, Error = ProtocolError> + Send + 'static,
{
    pub fn new(sink: TSink, rx_outgoing_message:UnboundedReceiver<Message>) -> Self {
        let (tx_outgoing_message, rx_outgoing_message) = unbounded();
        let sender = MessageSender {
            buffered_message: None,
            rx_outgoing_message: rx_outgoing_message,
            sink,
        };

        sender
    }
}


impl <TSink> Future for MessageSender<TSink>
    where
        TSink: Sink<Message, Error = ProtocolError> + Send + 'static,
{
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        println!("{}", "message sender poll");
        Poll::Pending
    }
}