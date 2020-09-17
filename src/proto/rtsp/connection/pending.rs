use std::time::Duration;

use crate::proto::rtsp::connection::{REQUEST_MAX_TIMEOUT_DEFAULT_DURATION, REQUEST_TIMEOUT_DEFAULT_DURATION};

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
