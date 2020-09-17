use futures::{Sink};
use crate::proto::rtsp::codec::{Message, ProtocolError};
use futures::channel::mpsc::{UnboundedReceiver, unbounded};
use futures::future::Fuse;
// use futures::channel::mpsc::unbounded;
// use tokio::sync::mpsc::UnboundedReceiver;
// use tokio::stream::StreamExt;
use futures::future::FutureExt;


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