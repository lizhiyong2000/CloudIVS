use bytes::BytesMut;
use futures::channel::mpsc::UnboundedSender;
// use futures::channel::oneshot::{self, Canceled};
use futures::{Future, FutureExt};
use std::time::{Duration, Instant};
// use tokio::time::Delay;

use crate::protocol::rtsp::header::types::CSeq;
use crate::protocol::rtsp::connection::{OperationError, RequestTimeoutType};
use crate::protocol::rtsp::response::Response;
// use tokio::sync::oneshot;
// use tokio::sync::mpsc::UnboundedSender;
use tokio::time::{Delay, delay_for};
use std::task::{Poll, Context};
use std::pin::Pin;
use futures::channel::oneshot;

/// The default timeout for the maximum amount of time that we will wait for a request.
pub const REQUEST_MAX_TIMEOUT_DEFAULT_DURATION: Duration = Duration::from_secs(20);

/// The default timeout for the amount of time that we will wait for a request in between responses.
pub const REQUEST_TIMEOUT_DEFAULT_DURATION: Duration = Duration::from_secs(10);

/// A future returned when sending a request through a connection handle.
///
/// It evaluates to the corresponding response or possibly to an error that occurred during the
/// process.
#[derive(Debug)]
#[must_use = "futures do nothing unless polled"]
pub struct SendRequest {
    /// The timer representing the maximum amount of time that we will wait before considering this
    /// request as timed out. This is not refreshed by Continue (100) responses.
    max_timer: Option<Delay>,

    /// A receiver which the response receiver will use to send us the matched response or
    /// potentially other information (e.g. Continue (100) or cancellation notice).
    rx_response: oneshot::Receiver<PendingRequestResponse>,

    /// The `"CSeq"` value that the request had. We need this as it serves as a key for the response
    /// receiver.
    sequence_number: CSeq,

    /// The amount of time allowed between successive responses, whether those be Continue (100)
    /// responses or the actual response.
    timeout_duration: Option<Duration>,

    /// The timer representing the amount of time we will wait before considering this request
    /// timed out. This is refreshed by Continue (100) responses.
    timer: Option<Delay>,

    /// A channel connected to the response receiver which allows us to notify it that we no longer
    /// want to wait for the request in the case of a timeout.
    tx_pending_request: UnboundedSender<PendingRequestUpdate>,
}

impl SendRequest {
    /// Cancels the request by removing the pending request from the response receiver.
    fn cancel_request(&mut self) {
        let _ = self
            .tx_pending_request
            .unbounded_send(PendingRequestUpdate::RemovePendingRequest(
                self.sequence_number,
            ));
        self.rx_response.close();
    }

    /// Constructs a new pending request.
    pub(crate) fn new(
        rx_response: oneshot::Receiver<PendingRequestResponse>,
        tx_pending_request: UnboundedSender<PendingRequestUpdate>,
        sequence_number: CSeq,
        timeout_duration: Option<Duration>,
        max_timeout_duration: Option<Duration>,
    ) -> Self {
        // let max_timer = max_timeout_duration.map(|duration| Delay::new(Instant::now() + duration));
        // let timer = timeout_duration.map(|duration| Delay::new(Instant::now() + duration));

        let max_timer = max_timeout_duration.map(|duration| delay_for(duration));
        let timer = timeout_duration.map(|duration| delay_for(duration));

        SendRequest {
            max_timer,
            rx_response,
            sequence_number,
            timer,
            timeout_duration,
            tx_pending_request,
        }
    }

    /// Returns whether the request has already been cancelled.
    fn poll_is_cancelled(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> bool {
        if let Poll::Ready(Err(Canceled)) = self.rx_response.poll_unpin(cx) {
            true
        } else {
            false
        }
    }

    /// Polls the pending request to see if a response has been matched.
    ///
    /// There are two other possibilities, a response was matched, but it was a Continue (100)
    /// response. This will refresh the timer, and we will continue waiting for the actual response.
    ///
    /// The other possibility is that the connection state has changed such that we will not be
    /// receiving any more responses. This effectively cancels the request, but does not necessarily
    /// mean it was not processed by the agent. It only means we will not be receiving a response.
    fn poll_request(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<Response<BytesMut>, OperationError>> {

        // self.rx_response.poll()

        if let Poll::Ready(response) = self
            .rx_response
            .poll_unpin(cx)
            // .expect("`SendRequest.rx_response` should not error")
        {
            match response {

                Ok(res) =>
                {
                    match res {
                        PendingRequestResponse::Continue(rx_response) => {
                            self.rx_response = rx_response;
                            self.timer = self
                                .timeout_duration
                                .map(|duration| delay_for(duration));
                            // .map(|duration| Delay::new(Instant::now() + duration));
                        }
                        PendingRequestResponse::None => return Poll::Ready(Err(OperationError::RequestCancelled)),
                        PendingRequestResponse::Response(response) => return Poll::Ready(Ok(response)),
                    }

                },

                Err(_) =>{
                    return Poll::Ready(Err(OperationError::RequestCancelled))
                }


            }
        }

        Poll::Pending
    }

    /// Polls the max timer to see if it has expired, and if it has, a long timeout error will be
    /// returned.
    fn poll_max_timer(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), OperationError>> {
        if let Some(timer) = self.max_timer.as_mut() {
            match timer.poll_unpin(cx) {
                Poll::Ready(_) => {
                    self.as_mut().cancel_request();
                    return Poll::Ready(Err(OperationError::RequestTimedOut(RequestTimeoutType::Long)));
                }
                Poll::Pending => (),
                // Err(ref error) if error.is_at_capacity() => {
                //     self.cancel_request();
                //     return Poll::Ready(Err(OperationError::RequestCancelled));
                // }
                _ => panic!("max timer should not be shutdown"),
            }
        }

        Poll::Pending
    }

    /// Polls the timer to see if it has expired, and if it has, a long timeout error will be
    /// returned.
    fn poll_timer(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), OperationError>> {
        if let Some(timer) = self.timer.as_mut() {
            match timer.poll_unpin(cx) {
                Poll::Ready(_) => {
                    self.as_mut().cancel_request();
                    return Poll::Ready(Err(OperationError::RequestTimedOut(RequestTimeoutType::Short)));
                }
                Poll::Pending => (),
                // Poll::Ready(Err(ref error)) if error.is_at_capacity() => {
                //     self.cancel_request();
                //     return Poll::Ready(Err(OperationError::RequestCancelled));
                // }
                _ => panic!("timer should not be shutdown"),
            }
        }

        Poll::Pending
    }
}

impl Drop for SendRequest {
    fn drop(&mut self) {
        // if !self.poll_is_cancelled() {
            self.cancel_request();
        // }
    }
}

impl Future for SendRequest {
    // type Item = Response<BytesMut>;
    // type Error = OperationError;

    type Output = Result<Response<BytesMut>, OperationError>;

    /// Checks if a response has been returned for this request.
    ///
    /// If `Ok(Poll::Ready(`[`Response`]`)` is returned, then we have received a response.
    ///
    /// If `Poll::Pending` is returned, then we are still waiting for a response to be
    /// received.
    ///
    /// If `Err(`[`OperationError`]`)` is returned, then either the request has timed out or has
    /// been cancelled.
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output>{
        match self.as_mut().poll_request(cx) {
            Poll::Ready(Ok(response)) => return Poll::Ready(Ok(response)),
            Poll::Ready(Err(error)) => return Poll::Ready(Err(error)),
            _ => (),
        }

        if let Poll::Ready(Err(error)) = self.as_mut().poll_max_timer(cx) {
            return Poll::Ready(Err(error))
        }

        if let Poll::Ready(Err(error)) = self.as_mut().poll_timer(cx) {
            return Poll::Ready(Err(error))
        }

        Poll::Pending
    }
}

/// Options used to modify the behavior of a request.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct RequestOptions {
    /// How long we are willing to wait before the request is timed out. This is not refreshed by
    /// Continue (100) responses.
    max_timeout_duration: Option<Duration>,

    /// How long are we willing to wait before the request is timed out. This is refreshed by
    /// Continue (100) responses.
    timeout_duration: Option<Duration>,
}

impl RequestOptions {
    /// Constructs a new request options builder.
    pub fn builder() -> RequestOptionsBuilder {
        RequestOptionsBuilder::new()
    }

    /// Sets how long we are willing to wait before the request is timed out. This is not refreshed
    /// by Continue (100) responses.
    pub fn max_timeout_duration(&self) -> Option<Duration> {
        self.max_timeout_duration
    }

    // Constructs new request options with default values.
    pub fn new() -> Self {
        RequestOptions::builder().build()
    }

    /// Sets how long are we willing to wait before the request is timed out. This is refreshed by
    /// Continue (100) responses.
    pub fn timeout_duration(&self) -> Option<Duration> {
        self.timeout_duration
    }
}

impl Default for RequestOptions {
    fn default() -> Self {
        RequestOptions::new()
    }
}

/// Options builder used to modify the behavior of a request.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct RequestOptionsBuilder {
    /// How long we are willing to wait before the request is timed out. This is not refreshed by
    /// Continue (100) responses.
    max_timeout_duration: Option<Duration>,

    /// How long are we willing to wait before the request is timed out. This is refreshed by
    /// Continue (100) responses.
    timeout_duration: Option<Duration>,
}

impl RequestOptionsBuilder {
    // Constructs new request options with the set values.
    pub fn build(self) -> RequestOptions {
        RequestOptions {
            max_timeout_duration: self.max_timeout_duration,
            timeout_duration: self.timeout_duration,
        }
    }

    /// Sets how long we are willing to wait before the request is timed out. This is not refreshed
    /// by Continue (100) responses.
    pub fn max_timeout_duration(&mut self, duration: Option<Duration>) -> &mut Self {
        self.max_timeout_duration = duration;
        self
    }

    /// Constructs a new request options builder.
    pub fn new() -> Self {
        RequestOptionsBuilder {
            max_timeout_duration: Some(REQUEST_MAX_TIMEOUT_DEFAULT_DURATION),
            timeout_duration: Some(REQUEST_TIMEOUT_DEFAULT_DURATION),
        }
    }

    /// Sets how long are we willing to wait before the request is timed out. This is refreshed by
    /// Continue (100) responses.
    pub fn timeout_duration(&mut self, duration: Option<Duration>) -> &mut Self {
        self.timeout_duration = duration;
        self
    }

    /// Consumes the builder and sets how long we are willing to wait before the request is timed
    /// out. This is not refreshed by Continue (100) responses.
    pub fn with_max_timeout_duration(mut self, duration: Option<Duration>) -> Self {
        self.max_timeout_duration(duration);
        self
    }

    /// Consumes the builder and sets how long are we willing to wait before the request is timed
    /// out. This is refreshed by Continue (100) responses.
    pub fn with_timeout_duration(mut self, duration: Option<Duration>) -> Self {
        self.timeout_duration(duration);
        self
    }
}

impl Default for RequestOptionsBuilder {
    fn default() -> Self {
        RequestOptionsBuilder::new()
    }
}

/// Possible updates that the response receiver make send the pending request future.
#[derive(Debug)]
#[allow(clippy::large_enum_variant)]
pub enum PendingRequestResponse {
    /// The connection has received a Continue (100) response for this request.
    ///
    /// The given receiver will be used for further updates.
    Continue(oneshot::Receiver<PendingRequestResponse>),

    /// The response receiver is being shutdown, a matching response will not be arriving.
    None,

    /// The connection received a response for this request that was not a Continue (100) response.
    Response(Response<BytesMut>),
}

/// An update used to notify the response receiver of either a new pending request or that we want
/// to remove an existing pending request.
#[derive(Debug)]
pub enum PendingRequestUpdate {
    /// The response receiver should watch for a response with the given `"CSeq"`.
    ///
    /// Any updates the response receiver has shouild go through the given sender.
    AddPendingRequest((CSeq, oneshot::Sender<PendingRequestResponse>)),

    /// The request with the given `"CSeq"` should no longer be watched by the response receiver.
    RemovePendingRequest(CSeq),
}
