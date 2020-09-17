mod shutdown;
mod handler;
mod sender;
mod receiver;

use tokio::io::{AsyncRead, AsyncWrite};
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use crate::proto::rtsp::codec::{Codec, Message};
use tokio_util::codec::Framed;
use futures::future::Shared;
use futures::channel::{oneshot, mpsc};
use crate::proto::rtsp::message::request::Request;
use bytes::BytesMut;
use crate::proto::rtsp::message::response::Response;
use futures::{FutureExt, StreamExt, SinkExt, Future};
use futures::stream::SplitSink;
use futures::stream::SplitStream;

use crate::proto::rtsp::connection::receiver::MessageReceiver;
use crate::proto::rtsp::connection::sender::MessageSender;
use crate::proto::rtsp::connection::handler::MessageHandler;
use std::time::Duration;
use futures::channel::mpsc::unbounded;
use futures::task::Context;
use tokio::macros::support::{Pin, Poll};

pub const DEFAULT_CONTINUE_WAIT_DURATION: Duration = Duration::from_secs(5);
pub const DEFAULT_DECODE_TIMEOUT_DURATION: Duration = Duration::from_secs(10);
pub const DEFAULT_GRACEFUL_SHUTDOWN_TIMEOUT_DURATION: Duration = Duration::from_secs(10);
pub const DEFAULT_REQUEST_BUFFER_SIZE: usize = 10;

/// The default timeout for the maximum amount of time that we will wait for a request.
pub const REQUEST_MAX_TIMEOUT_DEFAULT_DURATION: Duration = Duration::from_secs(20);

/// The default timeout for the amount of time that we will wait for a request in between responses.
pub const REQUEST_TIMEOUT_DEFAULT_DURATION: Duration = Duration::from_secs(10);

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
    receiver: Option<MessageReceiver<SplitStream<Framed<TTransport, Codec>>>>,

    /// The internal sender responsible for sending all outgoing messages through the connection.
    sender: Option<MessageSender<SplitSink<Framed<TTransport, Codec>, Message>>>,

    /// A shutdown event receiver for when the request handler has finished processing all requests.
    // rx_handler_shutdown_event: Option<Shared<oneshot::Receiver<()>>>,

    /// The shutdown handler that keeps watch for a shutdown signal.
    handler: MessageHandler,
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
    pub fn new(
        transport: TTransport,
        // service: Option<TService>,
    ) -> Self
        // where
        //     TService: Service<Request<BytesMut>> + Send + 'static,
        //     TService::Future: Send + 'static,
        //     TService::Response: Into<Response<BytesMut>>,
    {
        Connection::with_config(transport, Config::default())
    }

    /// Polls the receiver if it is still running.
    fn poll_receiver(&mut self) {
        // if let Some(receiver) = self.receiver.as_mut() {
        //     match receiver.poll() {
        //         Ok(Async::Ready(_)) | Err(_) => {
        //             self.shutdown_receiver();
        //         }
        //         _ => (),
        //     }
        // }
    }

    // /// Polls the request handler shutdown event receiver to see if it has been shutdown.
    // ///
    // /// This is a no-op if the receiver is not shutdown. Otherwise, if the request handler is also
    // /// shutdown, this means the sender needs to be shutdown as well, so the connection can be
    // /// closed.
    // fn poll_request_handler_shutdown(&mut self) {
    //     // if self.is_receiver_shutdown() {
    //     //     if let Some(rx_handler_shutdown_event) = self.rx_handler_shutdown_event.as_mut() {
    //     //         match rx_handler_shutdown_event.poll() {
    //     //             Ok(Async::Ready(_)) | Err(_) => {
    //     //                 self.shutdown_sender();
    //     //             }
    //     //             Ok(Async::NotReady) => (),
    //     //         }
    //     //     }
    //     // }
    // }

    /// Polls the sender if it is still running.
    ///
    /// If the sender finishes, then no more messages can be sent. Since no more messages can be
    /// sent, we shutdown request receiving since we would not be able to send responses.
    fn poll_sender(&mut self) {
        // if let Some(sender) = self.sender.as_mut() {
        //     match sender.poll() {
        //         Ok(Async::Ready(_)) | Err(_) => {
        //             self.shutdown_request_receiver();
        //             self.shutdown_sender();
        //         }
        //         _ => (),
        //     }
        // }
    }

    /// Shuts down the receiver.
    fn shutdown_receiver(&mut self) {
        self.receiver = None;
    }

    // /// Shuts down the request receiver.
    // ///
    // /// If the request receiver was the only remaining active component of the receiver, then the
    // /// entire receiver is shutdown.
    // fn shutdown_request_receiver(&mut self) {
    //     // if let Some(receiver) = self.receiver.as_mut() {
    //     //     if receiver.shutdown_request_receiver() {
    //     //         self.receiver = None;
    //     //     }
    //     // }
    // }

    /// Shuts down the sender.
    fn shutdown_sender(&mut self) {
        // self.sender = None;
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
    pub fn with_config(
        transport: TTransport,
        config: Config,
    ) -> Self
        // where
        //     TService: Service<Request<BytesMut>> + Send + 'static,
        //     TService::Future: Send + 'static,
        //     TService::Response: Into<Response<BytesMut>>,
    {
        // Create all channels that the connection components will use to communicate with each
        // other.

        let (tx_codec_event, rx_codec_event) = mpsc::unbounded();
        let codec = Codec::with_events(tx_codec_event);

        // for receiver to delieve request message.
        let (tx_incoming_request, rx_incoming_request) =
            mpsc::channel(config.request_buffer_size());

        let (tx_outgoing_message, rx_outgoing_message) = unbounded();

        // let (tx_pending_request, rx_pending_request) = mpsc::unbounded();


        // let (tx_initiate_shutdown, rx_initiate_shutdown) = oneshot::channel();
        // let (tx_connection_shutdown_event, rx_connection_shutdown_event) = oneshot::channel();
        // let (tx_handler_shutdown_event, rx_handler_shutdown_event) = oneshot::channel();


        let (sink, stream) = Framed::new(transport, codec).split();

        // Create individual components. A request handler is only created if a service was given.

        let sender = MessageSender::new(sink,  rx_outgoing_message);
        let receiver = MessageReceiver::new(
            stream,
            rx_codec_event,
            tx_incoming_request,
            config.decode_timeout_duration(),
        );

        let handler = MessageHandler::new(
            rx_incoming_request,
            tx_outgoing_message,
            config.continue_wait_duration());

        // let handler = if let Some(service) = service {
        //     Some(RequestHandler::new(
        //         service,
        //         rx_incoming_request,
        //         sender_handle.clone(),
        //         tx_handler_shutdown_event,
        //         config.continue_wait_duration(),
        //     ))
        // } else {
        //     None
        // };



        // let rx_handler_shutdown_event = if handler.is_some() {
        //     Some(rx_handler_shutdown_event.shared())
        // } else {
        //     None
        // };

        // Create the connection and the connection handle.

        let connection = Connection {
            allow_requests: Arc::new(AtomicBool::new(true)),
            receiver: Some(receiver),
            // rx_handler_shutdown_event: rx_handler_shutdown_event.clone(),
            sender: Some(sender),
            handler: handler,
        };
        // let connection_handle = ConnectionHandle::new(
        //     connection.allow_requests.clone(),
        //     rx_connection_shutdown_event.shared(),
        //     rx_handler_shutdown_event,
        //     sender_handle,
        //     tx_pending_request,
        //     tx_initiate_shutdown,
        //     config.graceful_shutdown_timeout_default_duration(),
        //     config.request_max_timeout_default_duration(),
        //     config.request_timeout_default_duration(),
        // );

        connection
    }
}
// pub struct Connection<TTransport>
// where
// TTransport: AsyncRead + AsyncWrite + Send + 'static,

impl <TTransport> Future for Connection<TTransport>
    where
        TTransport: AsyncRead + AsyncWrite + Send + 'static,
{
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        unimplemented!()
    }
}


/// A configuration option for controlling the behavior of an RTSP connection.
pub struct Config {
    continue_wait_duration: Option<Duration>,
    decode_timeout_duration: Duration,
    graceful_shutdown_timeout_default_duration: Duration,
    request_buffer_size: usize,
    request_max_timeout_default_duration: Option<Duration>,
    request_timeout_default_duration: Option<Duration>,
}

impl Config {
    pub fn builder() -> ConfigBuilder {
        ConfigBuilder::new()
    }

    pub fn new() -> Self {
        Config::default()
    }

    /// Returns how long the server should wait to send Continue (100) responses, while a request is
    /// being processed.
    pub fn continue_wait_duration(&self) -> Option<Duration> {
        self.continue_wait_duration
    }

    /// Returns how long the server will wait on a decoding step before considering the connection
    /// dead.
    pub fn decode_timeout_duration(&self) -> Duration {
        self.decode_timeout_duration
    }

    /// Returns the default timeout duration for how long a graceful shutdown should take.
    pub fn graceful_shutdown_timeout_default_duration(&self) -> Duration {
        self.graceful_shutdown_timeout_default_duration
    }

    /// Returns how many requests are allow to be buffered on the connection.
    pub fn request_buffer_size(&self) -> usize {
        self.request_buffer_size
    }

    /// Returns the default timeout duration for how long we should wait for a request until it is
    /// considered timed out. This is not refreshed by Continue (100) responses.
    pub fn request_max_timeout_default_duration(&self) -> Option<Duration> {
        self.request_max_timeout_default_duration
    }

    /// Returns the default timeout duration for how long we should wait for a request until it is
    /// considered timed out. This is refreshed by Continue (100) responses.
    pub fn request_timeout_default_duration(&self) -> Option<Duration> {
        self.request_timeout_default_duration
    }
}

impl Default for Config {
    fn default() -> Self {
        Config::builder().build()
    }
}

/// A builder type for constructing a connection configuration instance.
pub struct ConfigBuilder {
    continue_wait_duration: Option<Duration>,
    decode_timeout_duration: Duration,
    graceful_shutdown_timeout_default_duration: Duration,
    request_buffer_size: usize,
    request_max_timeout_default_duration: Option<Duration>,
    request_timeout_default_duration: Option<Duration>,
}

impl ConfigBuilder {
    /// Consumes the builder and constructs the [`Config`].
    pub fn build(self) -> Config {
        Config {
            continue_wait_duration: self.continue_wait_duration,
            decode_timeout_duration: self.decode_timeout_duration,
            graceful_shutdown_timeout_default_duration: self
                .graceful_shutdown_timeout_default_duration,
            request_buffer_size: self.request_buffer_size,
            request_max_timeout_default_duration: self.request_max_timeout_default_duration,
            request_timeout_default_duration: self.request_timeout_default_duration,
        }
    }

    /// Sets how long the server should wait to send Continue (100) responses, while a request is
    /// being processed.
    pub fn continue_wait_duration(&mut self, duration: Option<Duration>) -> &mut Self {
        self.continue_wait_duration = duration;
        self
    }

    /// Sets how long the server will wait on a decoding step before considering the connection
    /// dead.
    pub fn decode_timeout_duration(&mut self, duration: Duration) -> &mut Self {
        self.decode_timeout_duration = duration;
        self
    }

    /// Sets the default timeout duration for how long a graceful shutdown should take.
    pub fn graceful_shutdown_timeout_default_duration(&mut self, duration: Duration) -> &mut Self {
        self.graceful_shutdown_timeout_default_duration = duration;
        self
    }

    /// Constructs a new config builder.
    pub fn new() -> Self {
        ConfigBuilder {
            continue_wait_duration: Some(DEFAULT_CONTINUE_WAIT_DURATION),
            decode_timeout_duration: DEFAULT_DECODE_TIMEOUT_DURATION,
            graceful_shutdown_timeout_default_duration: DEFAULT_GRACEFUL_SHUTDOWN_TIMEOUT_DURATION,
            request_buffer_size: DEFAULT_REQUEST_BUFFER_SIZE,
            request_max_timeout_default_duration: Some(REQUEST_MAX_TIMEOUT_DEFAULT_DURATION),
            request_timeout_default_duration: Some(REQUEST_TIMEOUT_DEFAULT_DURATION),
        }
    }

    /// Sets how many requests are allow to be buffered on the connection.
    pub fn request_buffer_size(&mut self, size: usize) -> &mut Self {
        self.request_buffer_size = size;
        self
    }

    /// Sets the default timeout duration for how long we should wait for a request until it is
    /// considered timed out. This is not refreshed by Continue (100) responses.
    pub fn request_max_timeout_default_duration(
        &mut self,
        duration: Option<Duration>,
    ) -> &mut Self {
        self.request_max_timeout_default_duration = duration;
        self
    }

    /// Sets the default timeout duration for how long we should wait for a request until it is
    /// considered timed out. This is refreshed by Continue (100) responses.
    pub fn request_timeout_default_duration(&mut self, duration: Option<Duration>) -> &mut Self {
        self.request_timeout_default_duration = duration;
        self
    }

    /// Consumes the builder and sets how long the server should wait to send Continue (100)
    /// responses, while a request is being processed.
    pub fn with_continue_wait_duration(mut self, duration: Option<Duration>) -> Self {
        self.continue_wait_duration(duration);
        self
    }

    /// Consumes the builder and sets how long the server will wait on a decoding step before
    /// considering the connection dead.
    pub fn with_decode_timeout_duration(mut self, duration: Duration) -> Self {
        self.decode_timeout_duration(duration);
        self
    }

    /// Consumes the builder and sets the default timeout duration for how long a graceful shutdown
    /// should take.
    pub fn with_graceful_shutdown_timeout_default_duration(mut self, duration: Duration) -> Self {
        self.graceful_shutdown_timeout_default_duration(duration);
        self
    }

    /// Consumes the builder and sets how many requests are allow to be buffered on the connection.
    pub fn with_request_buffer_size(mut self, size: usize) -> Self {
        self.request_buffer_size(size);
        self
    }

    /// Consumes the builder and sets the default timeout duration for how long we should wait for a
    /// request until it is  considered timed out. This is not refreshed by Continue (100)
    /// responses.
    pub fn with_request_max_timeout_default_duration(mut self, duration: Option<Duration>) -> Self {
        self.request_max_timeout_default_duration(duration);
        self
    }

    /// Consumes the builder and sets the default timeout duration for how long we should wait for a
    /// request until it is considered timed out. This is refreshed by Continue (100) responses.
    pub fn with_request_timeout_default_duration(mut self, duration: Option<Duration>) -> Self {
        self.request_timeout_default_duration(duration);
        self
    }
}

impl Default for ConfigBuilder {
    fn default() -> Self {
        ConfigBuilder::new()
    }
}