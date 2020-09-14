// use crate::errors::ConnectionError;
use std::net::TcpStream;
use std::io;
use url::{Url, ParseError};
use std::fs::read;
use std::io::ErrorKind;
use url::quirks::host;

#[derive(Default)]
pub struct RTSPClient{

    url: String,
    connected: bool

}


impl RTSPClient{
    pub fn new(url:String) -> RTSPClient {

        return RTSPClient{
            url,
            ..Default::default()
        }
    }

    pub fn connect(&mut self) -> Result<(), io::Error>{
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

                let mut stream = TcpStream::connect(format!("{}:{}", host_str, host_port))?;

                self.connected = true;
                println!("{}", format!("connected to {}:{}", host_str, host_port));
                return Ok(());
            }
        }


    }

    pub fn sendSetup(&mut self) {

    }
}