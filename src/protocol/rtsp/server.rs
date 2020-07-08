use bytes::BytesMut;
use chrono::{self, offset, DateTime, Utc};
use futures::Stream;
use futures::{future, Future};
use std::collections::HashMap;
use std::convert::TryFrom;
use std::error::Error;
use std::net::{SocketAddr};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::net::TcpListener;
use tower_service::Service;

use crate::protocol::rtsp::header::map::HeaderMapExtension;
use crate::protocol::rtsp::header::types::{AcceptRanges, Public};
use crate::protocol::rtsp::method::Method;
use crate::protocol::rtsp::protocol::connection::Connection;
use crate::protocol::rtsp::protocol::connection::ConnectionHandle;
use crate::protocol::rtsp::request::Request;
use crate::protocol::rtsp::response::{Response, BAD_REQUEST_RESPONSE, NOT_IMPLEMENTED_RESPONSE};
use crate::protocol::rtsp::session::{Session, SessionID, SessionIDError, DEFAULT_SESSION_TIMEOUT};
use crate::protocol::rtsp::status::StatusCode;
use futures::task::{Poll, Context};
use futures::future::BoxFuture;
use std::pin::Pin;
// use tokio::prelude::{Stream, Future};

use futures::stream::StreamExt;

pub const SUPPORTED_METHODS: [Method; 2] = [Method::Options, Method::Setup];

/// Experimental high-level server implementation
pub struct Server {
    connections: Vec<ConnectionHandle>,
    sessions: HashMap<SessionID, Arc<Mutex<ServerSession>>>,
}

impl Server {
    fn new() -> Self {
        Server {
            connections: Vec::new(),
            sessions: HashMap::new(),
        }
    }

    pub async fn run(address: SocketAddr) {
        let server = Arc::new(Mutex::new(Server::new()));
        let listener = TcpListener::bind(&address).await.unwrap();;

        let serve = async move {
            let mut incoming = listener.incoming();
            while let Some(socket_res) = incoming.next().await {
                match socket_res {
                    Ok(socket) => {
                        println!("Accepted connection from {:?}", socket.peer_addr());
                        // TODO: Process socket
                        let service = ConnectionService {
                            session: None,
                            server: server.clone(),
                        };
                        let (connection, handler, handle) = Connection::new(socket, Some(service));

                        server.lock().unwrap().connections.push(handle);

                        tokio::spawn(connection);
                        tokio::spawn(handler.unwrap());
                    }
                    Err(err) => {
                        // Handle error by printing to STDOUT.
                        println!("accept error = {:?}", err);
                    }
                }
            }

        };

        serve.await;

        // let serve = listener.incoming().for_each(move |socket| {
        //     let server = server.clone();
        //     let service = ConnectionService {
        //         session: None,
        //         server: server.clone(),
        //     };
        //     let (connection, handler, handle) = Connection::new(socket, Some(service));
        //
        //     server.lock().unwrap().connections.push(handle);
        //
        //     tokio::spawn(connection);
        //     tokio::spawn(handler.unwrap());
        //
        //     Ok(())
        // });

        // tokio::main(serve.map_err(|_| ()));
        // tokio::start(serve.map_err(|_| ()))
    }
}

struct ConnectionService {
    session: Option<Arc<Mutex<ServerSession>>>,
    server: Arc<Mutex<Server>>,
}

impl ConnectionService {
    fn handle_method_options(
        &mut self,
        request: Request<BytesMut>,
    ) -> <Self as Service<Request<BytesMut>>>::Future {
        if let Some(session) = self.session.as_mut() {
            session.lock().unwrap().touch();
        }

        // Drop the body.
        let request = request.map(|_| BytesMut::new());

        // We do not support any media streams right now, so just always 404 on non-asterisk URI.
        let response = if request.uri().is_asterisk() {
            Response::<()>::builder()
                .with_typed_header(SUPPORTED_METHODS.iter().cloned().collect::<Public>())
                .with_body(BytesMut::new())
                .build()
                .unwrap()
        } else {
            Response::<()>::builder()
                .with_status_code(StatusCode::NotFound)
                .with_body(BytesMut::new())
                .build()
                .unwrap()
        };

        // Box::new(future::ok(response))
        Box::pin(future::ok(response))
    }

    fn handle_method_setup(
        &mut self,
        request: Request<BytesMut>,
    ) -> <Self as Service<Request<BytesMut>>>::Future {
        // Drop the body.
        let request = request.map(|_| BytesMut::new());

        // Client must set `"Accept-Ranges"` header with acceptable time range formats.
        match request.headers().typed_try_get::<AcceptRanges>() {
            Ok(range_formats) => {
                // TODO: Check that this resource supports one of these ranges.
                let _ = range_formats.unwrap_or(AcceptRanges::new());

                // For now, just say none are supported.
                // TODO: Fill out `"Accept-Ranges"` header based on requested resource.
                let response = Response::<()>::builder()
                    .with_status_code(StatusCode::HeaderFieldNotValidForResource)
                    .with_typed_header(AcceptRanges::new())
                    .with_body(BytesMut::new())
                    .build()
                    .unwrap();
                // return Box::new(future::ok(response));
                return Box::pin(future::ok(response));
            }

            // Err(_) => return Box::new(future::ok(BAD_REQUEST_RESPONSE.clone())),
            Err(_) => return Box::pin(future::ok(BAD_REQUEST_RESPONSE.clone())),
        }
    }
}

impl Service<Request<BytesMut>> for ConnectionService {
    type Response = Response<BytesMut>;
    type Error = Box<dyn Error + Send + 'static>;
    // type Future = Box<dyn Future<Output = Self::Response> + Send + 'static>;

    // type Future = BoxFuture<'static, Self::Response>;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;


    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        // Poll::Ready(())
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, mut request: Request<BytesMut>) -> Self::Future {

        request.uri_mut().normalize();

        match request.method() {
            Method::Options => self.handle_method_options(request),
            Method::Setup => self.handle_method_setup(request),

            // PLAY_NOTIFY and REDIRECT are handled here as servers do not respond to such requests.
            // _ => Box::new(future::ok(NOT_IMPLEMENTED_RESPONSE.clone())),
            _ => Box::pin(future::ok(NOT_IMPLEMENTED_RESPONSE.clone())),
        }
    }
}

pub struct ServerSession {
    expire_time: DateTime<Utc>,
    id: SessionID,
}

impl ServerSession {
    pub fn new<T>(active_client: ConnectionHandle) -> Self
    where
        SessionID: TryFrom<T, Error = SessionIDError>,
    {
        let expire_time = offset::Utc::now()
            .checked_add_signed(chrono::Duration::from_std(DEFAULT_SESSION_TIMEOUT).unwrap())
            .unwrap();

        ServerSession::with_timeout(expire_time, active_client)
    }

    pub fn with_timeout<T>(expire_time: DateTime<Utc>, active_client: ConnectionHandle) -> Self
    where
        SessionID: TryFrom<T, Error = SessionIDError>,
    {
        ServerSession {
            expire_time,
            id: SessionID::random(),
        }
    }

    fn touch(&mut self) {
        self.set_timeout(DEFAULT_SESSION_TIMEOUT).unwrap();
    }
}

impl Session for ServerSession {
    fn expire_time(&self) -> DateTime<Utc> {
        self.expire_time
    }

    fn id(&self) -> &SessionID {
        &self.id
    }

    fn set_expire_time(&mut self, expire_time: DateTime<Utc>) {
        self.expire_time = expire_time;
    }

    fn set_timeout(&mut self, timeout: Duration) -> Result<(), ()> {
        let timeout = chrono::Duration::from_std(timeout).map_err(|_| ())?;
        self.expire_time = offset::Utc::now().checked_add_signed(timeout).ok_or(())?;
        Ok(())
    }
}
