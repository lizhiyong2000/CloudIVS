use std::fmt::{Display, Formatter, Debug};
use std::{fmt, mem};
use std::sync::{Arc, Mutex};
use std::sync::atomic::AtomicBool;
use std::time::Duration;

use log::{info, error};

use atomic::Ordering;
use bytes::BytesMut;
use futures::{Future, future, FutureExt, SinkExt, StreamExt, TryFutureExt};
use futures::channel::{mpsc, oneshot};
use futures::channel::mpsc::{unbounded, UnboundedSender};
use futures::future::{Either, Shared};
use futures::stream::SplitSink;
use futures::stream::SplitStream;
use futures::task::Context;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::macros::support::{Pin, Poll};
use tokio_util::codec::Framed;

use crate::proto::rtsp::codec::{Codec, Message};
use crate::proto::rtsp::connection::handler::MessageHandler;
// use crate::proto::rtsp::connection::OperationError::RequestTimedOut;
use crate::proto::rtsp::connection::pending::{RequestOptions, PendingRequestUpdate, SendRequest};
use crate::proto::rtsp::connection::receiver::MessageReceiver;
use crate::proto::rtsp::connection::sender::{MessageSender, SenderHandle};
use crate::proto::rtsp::message::request::Request;
use crate::proto::rtsp::message::response::Response;
use std::error::Error;
use crate::proto::rtsp::message::header::map::HeaderMapExtension;
use crate::proto::rtsp::message::header::types::CSeq;

mod shutdown;
mod handler;
mod sender;
mod receiver;
mod pending;

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

    request_max_timeout_default_duration: Option<Duration>,

    request_timeout_default_duration: Option<Duration>,

    /// The internal receiver responsible for processing all incoming messages.
    receiver: Option<MessageReceiver<SplitStream<Framed<TTransport, Codec>>>>,

    /// The internal sender responsible for sending all outgoing messages through the connection.
    sender: Option<MessageSender<SplitSink<Framed<TTransport, Codec>, Message>>>,

    // A shutdown event receiver for when the request handler has finished processing all requests.
    // rx_handler_shutdown_event: Option<Shared<oneshot::Receiver<()>>>,
    //
    // /// The shutdown handler that keeps watch for a shutdown signal.
    // handler: Option<MessageHandler>,
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
    ) -> (Self, ConnectionHandle)
    {
        Connection::with_config(transport, Config::default())
    }



    /// Polls the receiver if it is still running.
    fn poll_receiver(mut self: Pin<&mut Self>, cx: &mut Context<'_>) {


        if let Some(receiver) = self.receiver.as_mut() {
            match receiver.poll_unpin(cx) {
                Poll::Ready(_) => {
                    self.shutdown_receiver();
                }
                _ => (),
            }
        }
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
    fn poll_sender(mut self: Pin<&mut Self>, cx: &mut Context<'_>) {
        if let Some(sender) = self.sender.as_mut() {
            match sender.poll_unpin(cx) {
                Poll::Ready(_) => {
                    // self.shutdown_request_receiver();
                    self.shutdown_sender();
                }
                _ => (),
            }
        }
    }


    // fn poll_handler(mut self: Pin<&mut Self>, cx: &mut Context<'_>) {
    //     if let Some(handler) = self.handler.as_mut() {
    //         match handler.poll_unpin(cx) {
    //             Poll::Ready(_) => {
    //                 // self.shutdown_request_receiver();
    //                 self.shutdown_handler();
    //             }
    //             _ => (),
    //         }
    //     }
    // }


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
        self.sender = None;
    }

    /// Shuts down the sender.
    // fn shutdown_handler(&mut self) {
    //     self.handler = None;
    // }

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
    ) -> (Self, ConnectionHandle)
    {
        // Create all channels that the connection components will use to communicate with each
        // other.

        let (tx_codec_event, rx_codec_event) = mpsc::unbounded();
        let codec = Codec::with_events(tx_codec_event);

        // for receiver to delieve request message.
        let (tx_incoming_request, rx_incoming_request) =
            mpsc::channel(config.request_buffer_size());

        // let (tx_outgoing_message, rx_outgoing_message) = unbounded();

        let (tx_pending_request, rx_pending_request) = mpsc::unbounded();


        let (tx_initiate_shutdown, rx_initiate_shutdown) = oneshot::channel();
        let (tx_connection_shutdown_event, rx_connection_shutdown_event) = oneshot::channel();
        // let (tx_handler_shutdown_event, rx_handler_shutdown_event) = oneshot::channel();


        let (sink, stream) = Framed::new(transport, codec).split();

        // Create individual components. A request handler is only created if a service was given.

        let (sender, sender_handle) = MessageSender::new(sink);


        let handler = MessageHandler::new(
            rx_incoming_request,
            rx_pending_request,
            config.continue_wait_duration(), config.request_buffer_size(), sender_handle.clone());


        let receiver = MessageReceiver::new(
            stream,
            rx_codec_event,
            tx_incoming_request,
            Some(handler),
            config.decode_timeout_duration(),
        );



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
            // handler: Some(handler),
            request_max_timeout_default_duration: None,

            request_timeout_default_duration: None,
        };
        let connection_handle = ConnectionHandle::new(
            connection.allow_requests.clone(),
            config.request_max_timeout_default_duration(),
            config.request_timeout_default_duration(),

            tx_initiate_shutdown,
            rx_connection_shutdown_event.shared(),
            sender_handle,
            // connection.allow_requests.clone(),

            // rx_handler_shutdown_event,

            tx_pending_request,


            // config.graceful_shutdown_timeout_default_duration(),
            // config.request_max_timeout_default_duration(),
            // config.request_timeout_default_duration(),
        );

        (connection, connection_handle)
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

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        self.as_mut().poll_receiver(cx);
        self.as_mut().poll_sender(cx);
        // self.as_mut().poll_handler(cx);

        info!("connection poll");

        Poll::Pending

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


/// Specifies what kind of request timeout occurred.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum RequestTimeoutType {
    /// The timeout was for the entire duration of waiting for the final response. Specifically,
    /// Continue (100) responses are not final responses.
    Long,

    /// The timeout was for a duration for which no response was heard.
    Short,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum OperationError {

    RequestNotAllowed,

    /// An attempt was made to send a request when the write state no longer allows sending
    /// requests. This situation can occur if, for example, a graceful shutdown is happening or an
    /// error occurred while trying to send a message to the receiving agent.
    Closed,

    /// A pending request that neither timed out nor received a corresponding response was
    /// cancelled. This will only occur when the read state has been changed such that responses are
    /// no longer able to be read, thus any requests currently pending will be cancelled.
    RequestCancelled,

    /// A pending request timed out while waiting for its corresponding response. For any given
    /// request, there are two timeouts to be considered. The first is a timeout for a duration of
    /// time for which no response is heard. However, if a Continue (100) response is received for
    /// this request, the timer will be reset. The second timer is one for the entire duration for
    /// which we are waiting for the final response regardless of any received Continue (100)
    /// responses.
    RequestTimedOut(RequestTimeoutType),
}

impl Display for OperationError{
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        use self::OperationError::*;

        match self {
            RequestNotAllowed => write!(formatter, "RequestNotAllowed"),
            RequestTimedOut(error) => write!(formatter, "RequestTimedOut"),
            Closed => write!(formatter, "Closed"),
            RequestCancelled => write!(formatter, "RequestCancelled"),
        }
    }
}

// impl Debug for OperationError{
//     fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
//         write!(formatter, "{}", "test");
//     }
// }

impl Error for OperationError {}

/// A handle to an RTSP connection.
///
/// This can be used to send requests or shutdown the connection.
// #[derive(Clone, Debug)]
pub struct ConnectionHandle {
    allow_requests: Arc<AtomicBool>,

    request_max_timeout_default_duration: Option<Duration>,

    request_timeout_default_duration: Option<Duration>,

    rx_connection_shutdown_event:Shared<oneshot::Receiver<()>>,
    tx_initiate_shutdown:Option<oneshot::Sender<()>>,
    /// A handle to the sender, so that we can send requests.
    sender_handle: SenderHandle,

    // /// The next `"CSeq"` that will be used when sending a request.
    // sequence_number: Arc<Mutex<CSeq>>,
    //
    // /// A receiver which can be used to check when shutdown of the connection and request handler
    // /// has finished.
    // shutdown_receiver: ConnectionShutdownReceiver,
    //
    // /// A shared sender which allows us to shutdown the connection.
    // shutdown_sender: Arc<Mutex<ConnectionShutdownSender>>,
    //
    // /// A sender used to notify the response receiver that we want to add a new pending request.
    tx_pending_request: UnboundedSender<PendingRequestUpdate>,

    /// The next `"CSeq"` that will be used when sending a request.
    sequence_number: Arc<Mutex<CSeq>>,
}

impl ConnectionHandle {
    /// Constructs a new connection handle.
    #[allow(clippy::too_many_arguments)]
    pub(self) fn new(
        allow_requests: Arc<AtomicBool>,

        request_max_timeout_default_duration: Option<Duration>,

        request_timeout_default_duration: Option<Duration>,


        tx_initiate_shutdown: oneshot::Sender<()>,
        // allow_requests: Arc<AtomicBool>,
        rx_connection_shutdown_event: Shared<oneshot::Receiver<()>>,
        // rx_handler_shutdown_event: Option<Shared<oneshot::Receiver<()>>>,
        sender_handle: SenderHandle,
        tx_pending_request: UnboundedSender<PendingRequestUpdate>,

        // graceful_shutdown_timeout_default_duration: Duration,
        // request_max_timeout_default_duration: Option<Duration>,
        // request_timeout_default_duration: Option<Duration>,


    ) -> Self {
        // let shutdown_receiver = ConnectionShutdownReceiver::new(
        //     rx_connection_shutdown_event,
        //     rx_handler_shutdown_event,
        // );
        // let shutdown_sender = ConnectionShutdownSender::new(
        //     tx_initiate_shutdown,
        //     graceful_shutdown_timeout_default_duration,
        // );

        ConnectionHandle {
            allow_requests,
            request_max_timeout_default_duration,
            request_timeout_default_duration,
            sender_handle,
            rx_connection_shutdown_event,
            tx_initiate_shutdown: Some(tx_initiate_shutdown),
            tx_pending_request,
            sequence_number: Arc::new(Mutex::new(CSeq::random())),
        }
    }

    // /// Sends the given request with default options.
    // ///
    // /// See [`ConnectionHandle::send_request_with_options`] for more information.
    // pub fn send_request<TRequest, TBody>(
    //     &mut self,
    //     request: TRequest,
    // ) -> impl Future<Output = Result<Response<BytesMut>, OperationError>>
    //     where
    //         TRequest: Into<Request<TBody>>,
    //         TBody: AsRef<[u8]>,
    // {
    //
    //     self.sender_handle.try_send_message()
    // }


    /// Shuts down the connection if it is not already shutdown.
    ///
    /// See [`ShutdownType`] for the different ways in which the connection can be shutdown.
    ///
    /// Even if the connection is shutdown, the request handler may still be processing remaining
    /// requests.
    pub fn shutdown(&mut self) {
        if let Some(tx_initiate_shutdown) = self.tx_initiate_shutdown.take() {
            let _ = tx_initiate_shutdown.send(());
        }
    }

    /// Sends the given request with default options.
    ///
    /// See [`ConnectionHandle::send_request_with_options`] for more information.
    pub async fn send_request<TRequest, TBody>(
        &mut self,
        request: TRequest,
    ) -> Result<Response<BytesMut>, OperationError>
    // ) -> impl Future<Output = Result<Response<BytesMut>, OperationError>>
        where
            TRequest: Into<Request<TBody>>,
            TBody: AsRef<[u8]>,
    {

        info!("{}","send request");

        let options = RequestOptions::builder()
            .max_timeout_duration(self.request_max_timeout_default_duration)
            .timeout_duration(self.request_timeout_default_duration)
            .build();

        info!("{}", options);

        if !self.allow_requests.load(Ordering::SeqCst) {
            // future::Either::
            return Err(OperationError::RequestNotAllowed);
        }

        // return future::err(OperationError::Closed) ;

        let mut lock = self
            .sequence_number
            .lock()
            .expect("`ConnectionHandler.sequence_number` should not be poisoned");
        let sequence_number = *lock;

        // Convert the request into the expect transport format, including setting the `"CSeq"`.
        let mut request = request.into().map(|body| BytesMut::from(body.as_ref()));
        request.headers_mut().typed_insert(sequence_number);

        // Notify the response receiver that we are going to be expecting a response for this
        // request.
        let (tx_response, rx_response) = oneshot::channel();
        let update = PendingRequestUpdate::AddPendingRequest((sequence_number, tx_response));

        if self.tx_pending_request.unbounded_send(update).is_err() {
            info!("tx_pending_request send failed");
            return Err(OperationError::Closed);
        }

        if self
            .sender_handle
            .try_send_message(Message::Request(request))
            .is_err()
        {

            info!("sender handle send error");
            // The sender is shutdown, so we need to renotify the response receiver and remove the
            // pending request we just added. If this fails as well, then the response receiver has
            // been shutdown, but it does not matter.
            let _ = self
                .tx_pending_request
                .unbounded_send(PendingRequestUpdate::RemovePendingRequest(sequence_number));
            return Err(OperationError::Closed);
        }

        *lock = sequence_number.wrapping_increment();
        mem::drop(lock);

        // return future::err(OperationError::Closed);
        let mut sr = SendRequest::new(
            rx_response,
            self.tx_pending_request.clone(),
            sequence_number,
            options.timeout_duration(),
            options.max_timeout_duration(),
        );
        Pin::new(&mut sr).await

    }

}