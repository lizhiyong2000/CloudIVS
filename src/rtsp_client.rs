use std::fs::read;
use std::io;
use std::io::{BufRead, BufReader, ErrorKind, Write};
// use crate::errors::ConnectionError;
use std::net::TcpStream;
use std::str::from_utf8;

use url::{ParseError, Url};
use url::quirks::host;

// method            direction        object     requirement
// DESCRIBE          C->S             P,S        recommended
// ANNOUNCE          C->S, S->C       P,S        optional
// GET_PARAMETER     C->S, S->C       P,S        optional
// OPTIONS           C->S, S->C       P,S        required
// (S->C: optional)
// PAUSE             C->S             P,S        recommended
// PLAY              C->S             P,S        required
// RECORD            C->S             P,S        optional
// REDIRECT          S->C             P,S        optional
// SETUP             C->S             S          required
// SET_PARAMETER     C->S, S->C       P,S        optional
// TEARDOWN          C->S             P,S        required

#[derive(Default)]
pub struct RTSPClient{

    pub url: String,
    pub connected: bool,

    _url : Option<Url>,
    _socket: Option<TcpStream>,
    _cseq: u32,


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

                self._url = Some(_url.clone());

                let stream = TcpStream::connect(format!("{}:{}", host_str, host_port))?;

                self._socket = Some(stream);

                self.connected = true;
                // info!(format!("connected to {}:{}", host_str, host_port));
                return Ok(());
            }
        }


    }

    // OPTIONS rtsp://192.168.30.224:554/h264/ch1/main/av_stream&channelId=2 RTSP/1.0\r\n
    // CSeq: 2\r\n
    // User-Agent: LibVLC/3.0.6 (LIVE555 Streaming Media v2016.11.28)\r\n
    // \r\n
    pub fn sendOptions(&mut self){
        let req = format!("OPTIONS {} RTSP/1.0\r\nCSeq: {}\r\nUser-Agent: {}\r\n\r\n",
                             self.getRequestUrl(), self.getSeq(), self.getUserAgent());

        let mut buffer: Vec<u8> = Vec::new();

        let mut stream = self._socket.as_mut().unwrap();

        stream.write(req.as_bytes())
            .expect("Failed to write to server");
        let mut reader = BufReader::new(stream);

        loop{
            reader.read_until(b'\n', &mut buffer)
                .expect("Could not read into buffer");

            let b = &buffer[buffer.len()-4..];
            let end = format!("{}", from_utf8(&b).unwrap());

            // print!("{}", end);

            if end.ends_with("\r\n\r\n"){

                print!("Message Received:\r\n{}", from_utf8(&buffer).unwrap());
                break;
            }


        }

        // print!("{}", from_utf8(&buffer)
        //     .expect("Could not write buffer as string"));
    }

    // DESCRIBE rtsp://192.168.30.224:554/h264/ch1/main/av_stream&channelId=2 RTSP/1.0\r\n
    // CSeq: 3\r\n
    // User-Agent: LibVLC/3.0.6 (LIVE555 Streaming Media v2016.11.28)\r\n
    // Accept: application/sdp\r\n
    // \r\n

    // Authorization: Digest username="admin", realm="IP Camera(C6496)", nonce="75ebba210a21f5d87902abcc3343d9d0", uri="rtsp://192.168.30.224:554/h264/ch1/main/av_stream&channelId=2", response="6b876cf2eede9d4611e70b38ca531b3d"\r\n
    //
    pub fn sendDescribe(&mut self) {
        let req = format!("DESCRIBE {} RTSP/1.0\r\nCSeq: {}\r\nUser-Agent: {}\r\nAccept: application/sdp\r\n\r\n",
                          self.getRequestUrl(), self.getSeq(), self.getUserAgent());

        let mut buffer: Vec<u8> = Vec::new();

        let mut stream = self._socket.as_mut().unwrap();

        stream.write(req.as_bytes())
            .expect("Failed to write to server");
        let mut reader = BufReader::new(stream);

        loop{
            reader.read_until(b'\n', &mut buffer)
                .expect("Could not read into buffer");

            let b = &buffer[buffer.len()-4..];
            let end = format!("{}", from_utf8(&b).unwrap());

            // print!("{}", end);

            if end.ends_with("\r\n\r\n"){

                print!("Message Received:\r\n{}", from_utf8(&buffer).unwrap());
                break;
            }


        }

        // print!("{}", from_utf8(&buffer)
        //     .expect("Could not write buffer as string"));
    }

    pub fn sendSetup(&mut self) {

    }

    pub fn getUserAgent(&self) -> &str{
        return "LibCloudMedia/0.0.1";
    }

    pub fn getRequestUrl(&self) -> String {

        let temp_url = self._url.as_ref();
        let mut ref_url = temp_url.unwrap().clone();
        ref_url.set_username("");
        ref_url.set_password(Some(""));

        return ref_url.as_str().to_string();
    }

    pub fn getSeq(&mut self) -> u32{
        self._cseq += 1;
        return self._cseq;
    }
}