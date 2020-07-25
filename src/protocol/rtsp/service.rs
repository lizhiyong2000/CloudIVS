use bytes::BytesMut;
use futures::{future, Future};
use std::io;
use tower_service::Service;

// use futures::io::empty;

use crate::protocol::rtsp::request::Request;
use crate::protocol::rtsp::response::Response;
use std::task::{Context, Poll};

pub struct EmptyService;

impl Service<Request<BytesMut>> for EmptyService {
    type Response = Response<BytesMut>;
    type Error = io::Error;
    type Future = Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send + 'static>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>>{
        Poll::Ready(Ok(()))
    }


    fn call(&mut self, _: Request<BytesMut>) -> Self::Future {
        Box::new(futures::io::empty())
    }

    // fn poll_ready(&mut self) -> Poll<(), Self::Error> {
    //     Poll::Ready(Ok(()))
    // }
}
