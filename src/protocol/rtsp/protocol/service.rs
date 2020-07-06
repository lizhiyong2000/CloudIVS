use bytes::BytesMut;
use std::io;
use tower_service::Service;

use crate::protocol::rtsp::request::Request;
use crate::protocol::rtsp::response::Response;
use futures::stream::empty;
use futures::task::{Poll, Context};
use futures::Future;
use tokio::macros::support::Pin;
use futures::future::BoxFuture;


pub struct EmptyService;

// struct MyType<T>(T);
// impl<T> Future for MyType<T> where T: Future{
//     type Output = T::Output;
//
//     fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
//         unimplemented!()
//     }
// }

// impl Future for Box<dyn Future<Output = Response<BytesMut>>> {
//     type Output = Response<BytesMut>;
//
//     fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
//         unimplemented!()
//     }
// }


// type BoxFutureObj<Output> = Box<dyn Future<Output = Output>>;

impl Service<Request<BytesMut>> for EmptyService {
    type Response = Response<BytesMut>;
    type Error = io::Error;
    // type Future = Box<dyn Future<Output = Self::Response> + Send + 'static>;

    // type Future: Future<Output = Result<Self::Response, Self::Error>>;


    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: Request<BytesMut>) -> Self::Future {
        Pin::new(Box::new(empty()))
    }

    // fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<()> {
    //     Poll::Ready(())
    // }
    //
    // fn call(&mut self, _: Request<BytesMut>) -> Self::Future {
    //     Pin::new(Box::new(empty()))
    // }
}
