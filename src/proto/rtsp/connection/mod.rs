mod shutdown;
mod handler;

use tokio::io::{AsyncRead, AsyncWrite};
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use crate::proto::rtsp::codec::Codec;
use tokio_util::codec::Framed;
use futures::stream::SplitStream;
use futures::channel::mpsc::{Receiver, Sender};
use futures::future::Shared;
use futures::channel::{oneshot, mpsc};
use crate::proto::rtsp::message::request::Request;
use bytes::BytesMut;
use crate::proto::rtsp::message::response::Response;
use futures::{FutureExt, StreamExt};

/// Represents an RTSP connection between two RTSP agents.
///
/// RTSP servers and clients are both capable of sending and receiving requests and responses. As a
/// result, they share the same underlying connection logic.
#[must_use = "futures do nothing unless polled"]
pub struct Connection<TTransport>
    where
        TTransport: AsyncRead + AsyncWrite + Send + 'static,
{
    /// A shared atomic that determines whether we are allowed to send requests through this
    /// connection.
    allow_requests: Arc<AtomicBool>,

    /// The internal receiver responsible for processing all incoming messages.
    receiver: Option<Receiver<SplitStream<Framed<TTransport, Codec>>>>,

    /// The internal sender responsible for sending all outgoing messages through the connection.
    sender: Option<Sender<SplitSink<Framed<TTransport, Codec>>>>,

    /// A shutdown event receiver for when the request handler has finished processing all requests.
    rx_handler_shutdown_event: Option<Shared<oneshot::Receiver<()>>>,

    /// The shutdown handler that keeps watch for a shutdown signal.
    shutdown: ShutdownHandler,
}


impl<TTransport> Connection<TTransport>
    where
        TTransport: AsyncRead + AsyncWrite + Send + 'static,
{
    /// Returns whether the receiver is shutdown.
    fn is_receiver_shutdown(&self) -> bool {
        self.receiver.is_none()
    }

    /// Returns whether the sender is shutdown.
    fn is_sender_shutdown(&self) -> bool {
        self.sender.is_none()
    }

    /// Returns whether both the receiver and sender are shutdown.
    fn is_shutdown(&self) -> bool {
        self.is_receiver_shutdown() && self.is_sender_shutdown()
    }

    /// Constructs a new connection using the default configuration.
    ///
    /// See [`Connection::with_config`] for more information.
    pub fn new<TService>(
        transport: TTransport,
        service: Option<TService>,
    ) -> (Self, Option<RequestHandler<TService>>, ConnectionHandle)
        where
            TService: Service<Request<BytesMut>> + Send + 'static,
            TService::Future: Send + 'static,
            TService::Response: Into<Response<BytesMut>>,
    {
        Connection::with_config(transport, service, Config::default())
    }

    /// Polls the receiver if it is still running.
    fn poll_receiver(&mut self) {
        if let Some(receiver) = self.receiver.as_mut() {
            match receiver.poll() {
                Ok(Async::Ready(_)) | Err(_) => {
                    self.shutdown_receiver();
                }
                _ => (),
            }
        }
    }

    /// Polls the request handler shutdown event receiver to see if it has been shutdown.
    ///
    /// This is a no-op if the receiver is not shutdown. Otherwise, if the request handler is also
    /// shutdown, this means the sender needs to be shutdown as well, so the connection can be
    /// closed.
    fn poll_request_handler_shutdown(&mut self) {
        if self.is_receiver_shutdown() {
            if let Some(rx_handler_shutdown_event) = self.rx_handler_shutdown_event.as_mut() {
                match rx_handler_shutdown_event.poll() {
                    Ok(Async::Ready(_)) | Err(_) => {
                        self.shutdown_sender();
                    }
                    Ok(Async::NotReady) => (),
                }
            }
        }
    }

    /// Polls the sender if it is still running.
    ///
    /// If the sender finishes, then no more messages can be sent. Since no more messages can be
    /// sent, we shutdown request receiving since we would not be able to send responses.
    fn poll_sender(&mut self) {
        if let Some(sender) = self.sender.as_mut() {
            match sender.poll() {
                Ok(Async::Ready(_)) | Err(_) => {
                    self.shutdown_request_receiver();
                    self.shutdown_sender();
                }
                _ => (),
            }
        }
    }

    /// Shuts down the receiver.
    fn shutdown_receiver(&mut self) {
        self.receiver = None;
    }

    /// Shuts down the request receiver.
    ///
    /// If the request receiver was the only remaining active component of the receiver, then the
    /// entire receiver is shutdown.
    fn shutdown_request_receiver(&mut self) {
        if let Some(receiver) = self.receiver.as_mut() {
            if receiver.shutdown_request_receiver() {
                self.receiver = None;
            }
        }
    }

    /// Shuts down the sender.
    fn shutdown_sender(&mut self) {
        self.sender = None;
    }

    /// Constructs a new connection with the given configuration.
    ///
    /// Three different parts are returned: the connection itself, the request handler, and a handle
    /// to the connection.
    ///
    /// The connection should be run as a task until completion. It is responsible for all reading,
    /// writing, and shutdown management.
    ///
    /// The request handler should also be run as a task until completion. It is responsible for
    /// the processing and mapping of incoming requests into responses. The given service will be
    /// used as the mapping function.
    ///
    /// The connection handle is used to send requests and to force a shutdown of the connection if
    /// desired.
    pub fn with_config<TService>(
        transport: TTransport,
        service: Option<TService>,
        config: Config,
    ) -> (Self, Option<RequestHandler<TService>>, ConnectionHandle)
        where
            TService: Service<Request<BytesMut>> + Send + 'static,
            TService::Future: Send + 'static,
            TService::Response: Into<Response<BytesMut>>,
    {
        // Create all channels that the connection components will use to communicate with each
        // other.

        let (tx_codec_event, rx_codec_event) = mpsc::unbounded();
        let (tx_incoming_request, rx_incoming_request) =
            mpsc::channel(config.request_buffer_size());
        let (tx_pending_request, rx_pending_request) = mpsc::unbounded();
        let (tx_initiate_shutdown, rx_initiate_shutdown) = oneshot::channel();
        let (tx_connection_shutdown_event, rx_connection_shutdown_event) = oneshot::channel();
        let (tx_handler_shutdown_event, rx_handler_shutdown_event) = oneshot::channel();
        let codec = Codec::with_events(tx_codec_event);
        let (sink, stream) = Framed::new(transport, codec).split();

        // Create individual components. A request handler is only created if a service was given.

        let (sender, sender_handle) = Sender::new(sink);
        let receiver = Receiver::new(
            stream,
            rx_pending_request,
            rx_codec_event,
            tx_incoming_request,
            sender_handle.clone(),
            config.decode_timeout_duration(),
            config.request_buffer_size(),
        );
        let handler = if let Some(service) = service {
            Some(RequestHandler::new(
                service,
                rx_incoming_request,
                sender_handle.clone(),
                tx_handler_shutdown_event,
                config.continue_wait_duration(),
            ))
        } else {
            None
        };
        let rx_handler_shutdown_event = if handler.is_some() {
            Some(rx_handler_shutdown_event.shared())
        } else {
            None
        };

        // Create the connection and the connection handle.

        let connection = Connection {
            allow_requests: Arc::new(AtomicBool::new(true)),
            receiver: Some(receiver),
            rx_handler_shutdown_event: rx_handler_shutdown_event.clone(),
            sender: Some(sender),
            shutdown: ShutdownHandler::new(rx_initiate_shutdown, tx_connection_shutdown_event),
        };
        let connection_handle = ConnectionHandle::new(
            connection.allow_requests.clone(),
            rx_connection_shutdown_event.shared(),
            rx_handler_shutdown_event,
            sender_handle,
            tx_pending_request,
            tx_initiate_shutdown,
            config.graceful_shutdown_timeout_default_duration(),
            config.request_max_timeout_default_duration(),
            config.request_timeout_default_duration(),
        );

        (connection, handler, connection_handle)
    }
}