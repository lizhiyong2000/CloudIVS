#![feature(int_error_matching)]

use std::net::TcpStream;
use std::str;
use std::io::{self, BufRead, BufReader, Write};
use crate::proto::rtsp::client::RTSPClient;
use futures::executor::block_on;
use std::error::Error;
// use crate::rtsp_client::RTSPClient;
// use crate::errors::ConnectionError;

mod rtsp_client;
mod proto;


#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Connect to a peer
    let url = "rtsp://admin:dm666666@192.168.30.224:554/h264/ch1/main/av_stream";
    let mut client = RTSPClient::new(String::from(url));
    client.connect().await;
    Ok(())
}

///

fn main1() -> Result<(), io::Error>{
    //rtsp://admin:dm666666@192.168.30.224:554/h264/ch1/main/av_stream
    let url = "rtsp://admin:dm666666@192.168.30.224:554/h264/ch1/main/av_stream";
    let mut client = RTSPClient::new(String::from(url));
    block_on(client.connect());
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