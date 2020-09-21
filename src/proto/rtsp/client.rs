use std::io;
use std::io::ErrorKind;
// use tokio_codec::Framed;
use std::net::SocketAddr;
use std::ptr::null;
use std::rc::Rc;

use log::{info, error};

use bytes::BytesMut;
use futures::{Future, FutureExt, SinkExt, StreamExt, TryFutureExt, future};
use futures::stream::SplitSink;
use futures::stream::SplitStream;
use tokio::net::TcpStream;
// use tokio_util::codec::;
use tokio_util::codec::Framed;
use url::Url;

use crate::proto::rtsp::codec::{Codec, Message};
use crate::proto::rtsp::connection::{Connection, OperationError, ConnectionHandle, Authenticator};
use crate::proto::rtsp::message::request::Request;
use crate::proto::rtsp::message::response::Response;
use itertools::Either;
use std::sync::Arc;

// use DefultExecutor;

type RTSPFramed = Framed<TcpStream, Codec>;

// #[derive(Default)]
pub struct RTSPClient {
    pub url: String,
    pub connected: bool,
    connection: Option<ConnectionHandle>,
    _url : Option<Url>,
}

impl RTSPClient {

    pub fn new(url:String) -> RTSPClient {

        return RTSPClient{
            url,
            connected:false,
            connection: None,
            _url: None
        }
    }
    pub fn uri(&self) -> Option<Url>
    {
        return self._url.clone();
    }

    pub fn setAuthenticator(&mut self, auth:Authenticator){
        let conn = self.connection.as_mut().unwrap();
        conn.setAuthenticator(auth);
    }

    pub async fn connect(&mut self) -> Result<(), io::Error>{
        let parsed_url = Url::parse(self.url.as_str());

        match parsed_url{
            Err(_) =>{
                return Err(io::Error::new(ErrorKind::ConnectionAborted, "Invalid URL."));
            },
            Ok(_url) =>{
                let host_str = _url.host_str().unwrap();
                let host_port = match _url.port() {
                    Some(_port)=>
                        _port,
                    None=> 554
                };

                self._url = Some(_url.clone());


                let stream = TcpStream::connect(format!("{}:{}", host_str, host_port)).await;
                // let codec = Codec::new();

                info!("{}", format!("connected to {}:{}", host_str, host_port));
                //     Ok(())
                match stream{
                    Ok(c) => {
                        // self.framed= Some(Framed::new(c, codec));

                        // let mut executor = DefaultExecutor::current();

                        let (connection, handle)  = Connection::new(c);

                        self.connection = Some(handle);

                        info!("client connection set");

                        // let mut runtime = tokio::runtime::Runtime::new().unwrap();

                        // runtime.block_on(connection);

                        tokio::spawn(Box::new(connection));



                        // if let Some(handler) = handler {
                        //     executor.spawn(Box::new(handler)).unwrap();
                        // }

                        self.connected = true;
                    },
                    Err(e) => return Err(e),
                }

                return Ok(());
            }
        }


    }

    pub async fn send_request<R, B>(&mut self, request: R) -> Result<Response<BytesMut>, OperationError>
        where
            R: Into<Request<B>>,
            B: AsRef<[u8]>,
    {
        let conneciton = self.connection.as_mut();

        // info!(conn);

        if let Some(conn) = conneciton{
            return conn.send_request(request).await;
        }
        else{
            info!("connection not set");
        }


        Err(OperationError::Closed)
        // return Either::left(Either::Left(OperationError::Closed));


    }
    // pub fn connect() -> impl Future<Output=RTSPClient> {
    //     TcpStream::connect(&SocketAddr::new("127.0.0.1".parse().unwrap(), CLIENT_PORT))
    //         .map_err(|e| e.into())
    //         .map(move |stream| {
    //             let codec = Codec::new();
    //             let framed= stream.framed(codec);
    //             let connected = true;
    //             RTSPClient { url, connected, framed }
    //         })
    // }

    // pub async fn send_message(&mut self, message: Message) -> &Send<'_, RTSPFramed, Message> {
    //     println!("MESSAGE: {:#?}", message);
    //     self.framed.send(message)
    // }
}