use bytes::BytesMut;
use futures::{future, Future};
use std::io;
use tower_service::Service;

use crate::protocol::rtsp::request::Request;
use crate::protocol::rtsp::response::Response;
use std::task::Poll;

pub struct EmptyService;

impl Service<Request<BytesMut>> for EmptyService {
    type Response = Response<BytesMut>;
    type Error = io::Error;
    type Future = Box<dyn Future<Output = Self::Response> + Send + 'static>;

    fn call(&mut self, _: Request<BytesMut>) -> Self::Future {
        Box::new(futures::empty())
    }

    fn poll_ready(&mut self) -> Poll<()> {
        Poll::Ready(())
    }
}
