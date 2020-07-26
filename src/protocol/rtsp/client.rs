use bytes::BytesMut;
use futures::future::Future;
use std::io;
use std::net::SocketAddr;
// use tokio_executor::{DefaultExecutor, Executor};

// use tokio_tcp::TcpStream;

use crate::protocol::rtsp::connection::{Connection, ConnectionHandle, OperationError};
use crate::protocol::rtsp::service::EmptyService;
use crate::protocol::rtsp::request::Request;
use crate::protocol::rtsp::response::Response;
use tokio::net::TcpStream;
use futures::TryStreamExt;

use tokio::runtime::Runtime;

pub struct Client {
    handle: ConnectionHandle,
    server_address: SocketAddr,
}

impl Client {
    pub fn connect(server_address: SocketAddr) -> impl Future<Output=Result<Client,io::Error>> {
        TcpStream::connect(&server_address).and_then(move |tcp_stream| {
            // let mut executor = DefaultExecutor::current();
            let mut runtime = Runtime::new().unwrap();
            let (connection, handler
                , handle) = Connection::new::<EmptyService>(tcp_stream, None);

            runtime.spawn(Box::new(connection));
            // executor.spawn(Box::new(connection)).unwrap();

            if let Some(handler) = handler {
                runtime.spawn(Box::new(handler));
            }
            Ok(Client {
                handle,
                server_address,
            })
        })
    }

    pub fn server_address(&self) -> &SocketAddr {
        &self.server_address
    }

    pub fn send_request<R, B>(
        &mut self,
        request: R,
    ) -> Box<dyn Future<Output=Result<Response<BytesMut>, OperationError>>>
    // ) -> impl Future<Output=Result<Response<BytesMut>, OperationError> >
    where
        R: Into<Request<B>>,
        B: AsRef<[u8]>,
    {
        self.handle.send_request(request)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_bounds() {
        fn check_bounds<T: Send + Send>() {}

        check_bounds::<Client>();
    }
}
