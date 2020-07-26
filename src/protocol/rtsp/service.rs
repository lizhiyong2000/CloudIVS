use bytes::BytesMut;
use futures::{future, Future};
use std::io;
use tower_service::Service;

// use futures::io::empty;

use crate::protocol::rtsp::request::Request;
// use crate::protocol::rtsp::response::Response;
use std::task::{Context, Poll};
use std::pin::Pin;

use crate::protocol::rtsp::response::{Response, NOT_IMPLEMENTED_RESPONSE};

pub struct EmptyService;

impl Service<Request<BytesMut>> for EmptyService {

    // type Response;
    //
    // /// Errors produced by the service.
    // type Error;
    //
    // /// The future response value.
    // type Future: Future<Output = Result<Self::Response, Self::Error>>;


    type Response = Response<BytesMut>;
    type Error = io::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send + 'static>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>>{
        Poll::Ready(Ok(()))
    }


    fn call(&mut self, _: Request<BytesMut>) -> Self::Future {
        // Box::pin(futures::io::empty().into())

        Box::pin(future::ok(NOT_IMPLEMENTED_RESPONSE.clone()))
        // Box::pin(future::)

    }

    // fn poll_ready(&mut self) -> Poll<(), Self::Error> {
    //     Poll::Ready(Ok(()))
    // }
}
