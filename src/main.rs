#![feature(int_error_matching)]
// #![feature(ready_macro)]

use log::info;
use log4rs;

use std::convert::TryFrom;
use std::error::Error;
use std::io;
use std::net::TcpStream;
use std::str;
use std::time::Duration;

use bytes::BytesMut;
use futures::{Future, FutureExt, TryFutureExt};
use futures::executor::block_on;
use tokio::time;

use crate::proto::rtsp::client::RTSPClient;
use crate::proto::rtsp::message::method::Method;
use crate::proto::rtsp::message::request::Request;
use crate::proto::rtsp::message::uri::request::URI;
use tokio::time::delay_for;
use crate::proto::rtsp::connection::OperationError;

// use crate::rtsp_client::RTSPClient;
// use crate::errors::ConnectionError;

mod rtsp_client;
mod proto;


#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {

    log4rs::init_file("log4rs.yaml", Default::default()).unwrap();
    info!("INFO");

// fn main() -> impl Future<Output = i32> {
    // Connect to a peer
    let url = "rtsp://admin:dm666666@192.168.30.224:554/h264/ch1/main/av_stream";
    let mut client = RTSPClient::new(String::from(url));
    let result = client.connect().await;
    // .then(|result| {
    match result {
        Ok(_) => println!("Connected: {:?}", client.uri()),
        Err(error) => {
            println!("connect error: {}", error);
            // return Err(Box::new("connect error"));
            return Err("connect error".into());
        },
    }


    let addr = client.uri();
    // println!("Connected to server: {}", addr.unwrap());

    let mut builder = Request::builder();
    builder.method(Method::Setup).uri(URI::try_from(url).unwrap()).body(BytesMut::new());
    let request = builder.build().unwrap();

    // println!("request:{}", request);

    let result = client.send_request(request).await;

    // .then(|result| {
    match result {
        Ok(response) => println!("response: {:?}", response),
        Err(error) => println!("error sending request: {}", error),
    }

    // futures::future::ready(())
    // });

    delay_for(Duration::from_millis(1000 * 1000)).await;
    Ok(())


}

///

fn main1() -> Result<(), io::Error>{
    //rtsp://admin:dm666666@192.168.30.224:554/h264/ch1/main/av_stream
    let url = "rtsp://admin:dm666666@192.168.30.224:554/h264/ch1/main/av_stream";
    let mut client = RTSPClient::new(String::from(url));

    let rt = tokio::runtime::Runtime::new().unwrap();

    rt.enter(|| {client.connect()});

    // block_on(client.connect());
    // client.sendOptions();
    // client.sendDescribe();

    // OPTIONS rtsp://192.168.30.224:554/h264/ch1/main/av_stream&channelId=2 RTSP/1.0\r\n
    // CSeq: 2\r\n
    // User-Agent: LibVLC/3.0.6 (LIVE555 Streaming Media v2016.11.28)\r\n
    // \r\n

    // DESCRIBE rtsp://192.168.30.224:554/h264/ch1/main/av_stream&channelId=2 RTSP/1.0\r\n
    // CSeq: 3\r\n
    // User-Agent: LibVLC/3.0.6 (LIVE555 Streaming Media v2016.11.28)\r\n
    // Accept: application/sdp\r\n
    // \r\n


    // RTSP/1.0 401 Unauthorized\r\n
    // CSeq: 3\r\n
    // WWW-Authenticate: Digest realm="IP Camera(C6496)", nonce="75ebba210a21f5d87902abcc3343d9d0", stale="FALSE"\r\n
    // Date:  Thu, Jul 23 2020 16:03:04 GMT\r\n
    // \r\n


    // DESCRIBE rtsp://192.168.30.224:554/h264/ch1/main/av_stream&channelId=2 RTSP/1.0\r\n
    // CSeq: 4\r\n
    // Authorization: Digest username="admin", realm="IP Camera(C6496)", nonce="75ebba210a21f5d87902abcc3343d9d0", uri="rtsp://192.168.30.224:554/h264/ch1/main/av_stream&channelId=2", response="6b876cf2eede9d4611e70b38ca531b3d"\r\n
    // User-Agent: LibVLC/3.0.6 (LIVE555 Streaming Media v2016.11.28)\r\n
    // Accept: application/sdp\r\n
    // \r\n


    // SETUP rtsp://192.168.30.224:554/h264/ch1/main/av_stream&channelId=2/trackID=1 RTSP/1.0\r\n
    // CSeq: 5\r\n
    // Authorization: Digest username="admin", realm="IP Camera(C6496)", nonce="75ebba210a21f5d87902abcc3343d9d0", uri="rtsp://192.168.30.224:554/h264/ch1/main/av_stream&channelId=2/", response="b207757f4152a1b793670dceb37c11d4"\r\n
    // User-Agent: LibVLC/3.0.6 (LIVE555 Streaming Media v2016.11.28)\r\n
    // Transport: RTP/AVP;unicast;client_port=63994-63995
    // \r\n



    // PLAY rtsp://192.168.30.224:554/h264/ch1/main/av_stream&channelId=2/ RTSP/1.0\r\n
    // CSeq: 6\r\n
    // Authorization: Digest username="admin", realm="IP Camera(C6496)", nonce="75ebba210a21f5d87902abcc3343d9d0", uri="rtsp://192.168.30.224:554/h264/ch1/main/av_stream&channelId=2/", response="98962c804dbb3a95d7cdbbbe1a2234a4"\r\n
    // User-Agent: LibVLC/3.0.6 (LIVE555 Streaming Media v2016.11.28)\r\n
    // Session: 771996478
    // Range: npt=0.000-\r\n
    // \r\n


    // let mut stream = TcpStream::connect("192.168.30.224:554")
    //     .expect("Could not connect to server");
    // loop {
    //
    //     let mut input = String::new();
    //     let mut buffer: Vec<u8> = Vec::new();
    //     io::stdin().read_line(&mut input)
    //         .expect("Failed to read from stdin");
    //     stream.write(input.as_bytes())
    //         .expect("Failed to write to server");
    //     let mut reader = BufReader::new(&stream);
    //     reader.read_until(b'\n', &mut buffer)
    //         .expect("Could not read into buffer");
    //     print!("{}", str::from_utf8(&buffer)
    //         .expect("Could not write buffer as string"));
    // }

    Ok(())
}