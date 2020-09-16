// use tokio_codec::Framed;
use std::net::{SocketAddr};
use tokio::net::TcpStream;
use crate::proto::rtsp::codec::{Message, Codec};
use futures::{SinkExt, Future, TryFutureExt, FutureExt};
use std::io;
use std::io::ErrorKind;
use url::Url;
use std::ptr::null;
// use tokio_util::codec::;
use tokio_util::codec::Framed;

type RTSPFramed = Framed<TcpStream, Codec>;

// #[derive(Default)]
pub struct RTSPClient {
    pub url: String,
    pub connected: bool,
    framed: Option<RTSPFramed>,
    _url : Option<Url>,
}

impl RTSPClient {

    pub fn new(url:String) -> RTSPClient {

        return RTSPClient{
            url,
            connected:false,
            framed: None,
            _url: None
        }
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
                let codec = Codec::new();
                // self.framed= Some(Framed::new(stream.unwrap(), codec));

                println!("{}", format!("connected to {}:{}", host_str, host_port));
                //     Ok(())
                match stream{
                    Ok(c) => {
                        self.framed= Some(Framed::new(c, codec));
                        self.connected = true;
                    },
                    Err(e) => return Err(e),
                }

                    // .map_err(|e| e.into())
                    // .map(move |stream| {
                    //     let codec = Codec::new();
                    //     self.framed= Some(Framed::new(stream, codec));
                    //     self.connected = true;
                    //     Ok(())
                    //     // RTSPClient { url, connected, framed }
                    // });



                // let stream = TcpStream::connect(format!("{}:{}", host_str, host_port))?;
                //
                // self._socket = Some(stream);
                //
                // self.connected = true;
                // println!("{}", format!("connected to {}:{}", host_str, host_port));
                return Ok(());
            }
        }


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