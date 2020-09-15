use std::sync::Arc;
use std::net::{IpAddr, SocketAddr};
use std::future::Future;
use std::fmt;
use std::str;

// use yansi::Paint;
// use state::{Container, Storage};
// use futures::future::BoxFuture;
use atomic::{Atomic, Ordering};

// use crate::request::{FromParam, FromSegments, FromRequest, Outcome};
// use crate::request::{FromFormValue, FormItems, FormItem};

// use crate::{Rocket, Config, Shutdown, Route};
// use crate::http::{hyper, uri::{Origin, Segments}};
// use crate::http::{Method, Header, HeaderMap};
// use crate::http::{RawStr, ContentType, Accept, MediaType, CookieJar, Cookie};
// use crate::http::private::{Indexed, SmallVec};
// use crate::data::Limits;
use crate::method::Method;
use crate::header::{HeaderMap, Header};
// use std::sync::atomic::Ordering;

/// The type of an incoming web request.
///
/// This should be used sparingly in Rocket applications. In particular, it
/// should likely only be used when writing [`FromRequest`] implementations. It
/// contains all of the information for a given web request except for the body
/// data. This includes the HTTP method, URI, cookies, headers, and more.
pub struct Request<'r> {
    method: Atomic<Method>,
    uri: String,
    headers: HeaderMap<'r>
}


impl Request<'_> {
    pub(crate) fn clone(&self) -> Self {
        Request {
            method: Atomic::new(self.method()),
            uri: self.uri.clone(),
            headers: self.headers.clone()
        }
    }
}


impl<'r> Request<'r> {
    /// Create a new `Request` with the given `method` and `uri`.
    #[inline(always)]
    pub(crate) fn new<'s: 'r>(
        method: Method,
        uri: String
    ) -> Request<'r> {
        let mut request = Request {
            uri,
            method: Atomic::new(method),
            headers: HeaderMap::new(),

        };

        request
    }

    /// Retrieve the method from `self`.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use rocket::Request;
    /// use rocket::http::Method;
    ///
    /// # Request::example(Method::Get, "/uri", |request| {
    /// request.set_method(Method::Get);
    /// assert_eq!(request.method(), Method::Get);
    /// # });
    /// ```
    #[inline(always)]
    pub fn method(&self) -> Method {
        self.method.load(Ordering::Acquire)
    }

    /// Set the method of `self`.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use rocket::Request;
    /// use rocket::http::Method;
    ///
    /// # Request::example(Method::Get, "/uri", |request| {
    /// assert_eq!(request.method(), Method::Get);
    ///
    /// request.set_method(Method::Post);
    /// assert_eq!(request.method(), Method::Post);
    /// # });
    /// ```
    #[inline(always)]
    pub fn set_method(&mut self, method: Method) {
        self._set_method(method);
    }

    /// Borrow the [`Origin`] URI from `self`.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use rocket::Request;
    /// # use rocket::http::Method;
    /// # Request::example(Method::Get, "/uri", |request| {
    /// assert_eq!(request.uri().path(), "/uri");
    /// # });
    /// ```
    #[inline(always)]
    pub fn uri(&self) -> &String {
        &self.uri
    }

    /// Set the URI in `self` to `uri`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use rocket::http::uri::Origin;
    ///
    /// # use rocket::Request;
    /// # use rocket::http::Method;
    /// # Request::example(Method::Get, "/uri", |mut request| {
    /// let uri = Origin::parse("/hello/Sergio?type=greeting").unwrap();
    /// request.set_uri(uri);
    /// assert_eq!(request.uri().path(), "/hello/Sergio");
    /// assert_eq!(request.uri().query(), Some("type=greeting"));
    /// # });
    /// ```
    pub fn set_uri<'u: 'r>(&mut self, uri: String) {
        self.uri = uri;
    }



    /// Returns a [`HeaderMap`] of all of the headers in `self`.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use rocket::Request;
    /// # use rocket::http::Method;
    /// # Request::example(Method::Get, "/uri", |request| {
    /// let header_map = request.headers();
    /// assert!(header_map.is_empty());
    /// # });
    /// ```
    #[inline(always)]
    pub fn headers(&self) -> &HeaderMap<'r> {
        &self.headers
    }

    /// Add `header` to `self`'s headers. The type of `header` can be any type
    /// that implements the `Into<Header>` trait. This includes common types
    /// such as [`ContentType`] and [`Accept`].
    ///
    /// # Example
    ///
    /// ```rust
    /// # use rocket::Request;
    /// # use rocket::http::Method;
    /// use rocket::http::ContentType;
    ///
    /// # Request::example(Method::Get, "/uri", |mut request| {
    /// assert!(request.headers().is_empty());
    ///
    /// request.add_header(ContentType::HTML);
    /// assert!(request.headers().contains("Content-Type"));
    /// assert_eq!(request.headers().len(), 1);
    /// # });
    /// ```
    #[inline(always)]
    pub fn add_header<'h: 'r, H: Into<Header<'h>>>(&mut self, header: H) {
        self.headers.add(header.into());
    }

    /// Replaces the value of the header with name `header.name` with
    /// `header.value`. If no such header exists, `header` is added as a header
    /// to `self`.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use rocket::Request;
    /// # use rocket::http::Method;
    /// use rocket::http::ContentType;
    ///
    /// # Request::example(Method::Get, "/uri", |mut request| {
    /// assert!(request.headers().is_empty());
    ///
    /// request.add_header(ContentType::Any);
    /// assert_eq!(request.headers().get_one("Content-Type"), Some("*/*"));
    ///
    /// request.replace_header(ContentType::PNG);
    /// assert_eq!(request.headers().get_one("Content-Type"), Some("image/png"));
    /// # });
    /// ```
    #[inline(always)]
    pub fn replace_header<'h: 'r, H: Into<Header<'h>>>(&mut self, header: H) {
        self.headers.replace(header.into());
    }

}

// All of these methods only exist for internal, including codegen, purposes.
// They _are not_ part of the stable API. Please, don't use these.
#[doc(hidden)]
impl<'r> Request<'r> {

    /// Set the method of `self`, even when `self` is a shared reference. Used
    /// during routing to override methods for re-routing.
    #[inline(always)]
    pub(crate) fn _set_method(&self, method: Method) {
        self.method.store(method, Ordering::Release)
    }
}

impl fmt::Debug for Request<'_> {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt.debug_struct("Request")
            .field("method", &self.method)
            .field("uri", &self.uri)
            .field("headers", &self.headers())
            .finish()
    }
}

impl fmt::Display for Request<'_> {
    /// Pretty prints a Request. This is primarily used by Rocket's logging
    /// infrastructure.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.method(), &self.uri)?;
        Ok(())
    }
}

// type Indices = (usize, usize);
//
// #[derive(Clone)]
// pub(crate) struct IndexedFormItem {
//     raw: Indices,
//     key: Indices,
//     value: Indices
// }

// impl IndexedFormItem {
//     #[inline(always)]
//     fn from(s: &str, i: FormItem<'_>) -> Self {
//         let (r, k, v) = (indices(i.raw, s), indices(i.key, s), indices(i.value, s));
//         IndexedFormItem { raw: r, key: k, value: v }
//     }
//
//     // #[inline(always)]
//     // fn convert<'s>(&self, source: &'s str) -> FormItem<'s> {
//     //     FormItem {
//     //         raw: source[self.raw.0..self.raw.1].into(),
//     //         key: source[self.key.0..self.key.1].into(),
//     //         value: source[self.value.0..self.value.1].into(),
//     //     }
//     // }
// }

// fn indices(needle: &str, haystack: &str) -> (usize, usize) {
//     Indexed::checked_from(needle, haystack)
//         .expect("segments inside of path/query")
//         .indices()
// }