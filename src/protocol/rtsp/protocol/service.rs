use bytes::BytesMut;
use std::io;
use tower_service::Service;

use crate::protocol::rtsp::request::Request;
use crate::protocol::rtsp::response::Response;
use futures::stream::empty;
use futures::task::Poll;
use futures::Future;


pub struct EmptyService;

impl Service<Request<BytesMut>> for EmptyService {
    type Response = Response<BytesMut>;
    type Error = io::Error;
    type Future = Box<Future<Output = Self::Response> + Send + 'static>;

    fn call(&mut self, _: Request<BytesMut>) -> Self::Future {
        Box::new(empty())
    }

    fn poll_ready(&mut self) -> Poll<()> {
        Poll::Ready(())
    }
}
