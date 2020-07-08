use bytes::BytesMut;
use futures::future::Future;
use std::io;
use std::net::{SocketAddr};
use tokio_executor::{DefaultExecutor, Executor};
use tokio::net::TcpStream;
use crate::protocol::rtsp::protocol::connection::{Connection, ConnectionHandle, OperationError};
use crate::protocol::rtsp::protocol::service::EmptyService;
use crate::protocol::rtsp::request::Request;
use crate::protocol::rtsp::response::Response;
use std::pin::Pin;

pub struct Client {
    handle: ConnectionHandle,
    server_address: SocketAddr,
}

impl Client {
    pub async fn connect(server_address: SocketAddr) -> impl Future<Output = Result<Client, io::Error>> {

        TcpStream::connect(&server_address).await.unwrap().and_then(move |tcp_stream| {
            let mut executor = DefaultExecutor::current();
            let (connection, handler, handle) = Connection::new::<EmptyService>(tcp_stream, None);

            executor.spawn(Box::pin(connection)).unwrap();

            if let Some(handler) = handler {
                executor.spawn(Box::pin(handler)).unwrap();
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
    ) -> impl Future<Output = Result<Response<BytesMut>, OperationError>>
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
