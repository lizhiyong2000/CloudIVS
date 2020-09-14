use std::net::TcpStream;
use std::str;
use std::io::{self, BufRead, BufReader, Write};
use crate::rtsp_client::RTSPClient;
// use crate::errors::ConnectionError;

mod rtsp_client;

fn main() -> Result<(), io::Error>{
    //rtsp://admin:dm666666@192.168.30.224:554/h264/ch1/main/av_stream
    let url = "rtsp://admin:dm666666@192.168.30.224:554/h264/ch1/main/av_stream";
    let mut client = RTSPClient::new(String::from(url));
    client.connect()?;
    client.sendSetup();

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