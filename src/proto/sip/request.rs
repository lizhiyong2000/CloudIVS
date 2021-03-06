use std::io::{
    Error as IoError,
    ErrorKind as IoErrorKind,
    Result as IoResult
};

use super::*;

/// Sip Request Generator. When build is called the struct
/// is consumed and produces a SipMessage::Request variant.
/// Calling the `method` & `uri` methods before the `build`
/// method is required.
#[derive(Default)]
pub struct RequestBuilder {
    method: Option<Method>,
    uri: Option<Uri>,
    version: Version,
    headers: Headers,
    body: Vec<u8>,
}

impl RequestBuilder {
    /// Create a new instance.
    pub fn new() -> RequestBuilder {
        RequestBuilder {
            method: None,
            uri: None,
            version: Version::default(),
            headers: Headers::new(),
            body: vec![],
        }
    }

    /// Set the sip request method.
    pub fn method(mut self, method: Method) -> RequestBuilder {
        self.method = Some(method);
        self
    }

    /// Set the sip request uri.
    pub fn uri(mut self, uri: Uri) -> RequestBuilder {
        self.uri = Some(uri);
        self
    }

    /// Add multiple headers to the request header list.
    /// This use's Vec::extend so that the current items
    /// in the header list are kept.
    pub fn headers(mut self, headers: Vec<Header>) -> RequestBuilder {
        self.headers.extend(headers);
        self
    }

    /// Add a single header to the request header list.
    pub fn header(mut self, header: Header) -> RequestBuilder {
        self.headers.push(header);
        self
    }

    /// Get a reference to the header list.
    pub fn header_ref(&self) -> &Headers {
        &self.headers
    }

    /// Get a mutable reference to the header list.
    pub fn headers_ref_mut(&mut self) -> &mut Headers {
        &mut self.headers
    }

    /// Set the sip request body. This completely replaces
    /// the current request body.
    pub fn body(mut self, body: Vec<u8>) -> RequestBuilder {
        self.body = body;
        self
    }

    /// Build the sip request.
    pub fn build(self) -> IoResult<SipMessage> {
        if let Some(method) = self.method {
            if let Some(uri) = self.uri {
                Ok(SipMessage::Request {
                    method,
                    uri,
                    version: self.version,
                    headers: self.headers,
                    body: self.body,
                })
            } else {
                Err(IoError::new(
                    IoErrorKind::InvalidInput,
                    "`uri` method call required",
                ))
            }
        } else {
            Err(IoError::new(
                IoErrorKind::InvalidInput,
                "`method` method call required",
            ))
        }
    }
}
