#![feature(int_error_matching)]
#![recursion_limit="256"]
// #![feature(ready_macro)]
#![allow(unused_imports)]
#![allow(dead_code)]
#![allow(non_camel_case_types)]
#![allow(unreachable_code)]

#[macro_use]
extern crate trackable;

#[macro_use]
extern crate nom;


use log::{info, error};
use log4rs;

use std::convert::TryFrom;
use std::error::Error;
use std::io;
use std::net::{TcpStream, SocketAddr, SocketAddrV4, UdpSocket as StdUdpSocket};
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
use tokio::time::sleep;
use crate::proto::rtsp::connection::{OperationError, Authenticator};
use regex::Regex;
use crate::proto::rtsp::message::status::StatusCode;
use crate::proto::rtsp::message::header::map::HeaderMapExtension;
use crate::proto::rtsp::message::header::types::authenticate::{WWWAuthenticate, WWWAuthenticateSingle, AuthenticateMethod};
use crate::proto::rtsp::message::header::name::HeaderName;
use crate::proto::rtsp::message::header::value::HeaderValue;
use crate::proto::rtsp::message::header::types::Session;
use crate::proto::sdp::{SdpLine, parse_sdp};
use nom::AsBytes;
use std::ops::Deref;
use linked_hash_set::LinkedHashSet;
use crate::proto::sdp::attribute_type::SdpAttributeType;
use crate::proto::sdp::media_type::SdpMediaValue;
use crate::proto::sdp::attribute_type::SdpAttribute;

// use crate::rtsp_client::RTSPClient;
// use crate::errors::ConnectionError;



mod proto;

mod worker;

use worker::rtp_session::RTPSession;
use std::str::FromStr;
use tokio::net::UdpSocket;

use std::env;


#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {

    let curr_dir = env::current_dir();
    print!("{:?}", curr_dir);

    let s = "realm=\"IP Camera(C6496)\"";
    let r = Regex::new("(\\w+)=\"(.+)\"").unwrap();
    if let Some(caps) = r.captures(&s){
        for j in 0..caps.len() {
            println!("group {} : {}",  j, &caps[j]);
        }
    }



    log4rs::init_file("log4rs.yaml", Default::default()).unwrap();
    info!("INFO");

// fn main() -> impl Future<Output = i32> {
    // Connect to a peer
    // let url = "rtsp://admin:dm666666@192.168.30.224:554/h264/ch1/main/av_stream";
    let url = "rtsp://192.168.1.125:554/h264/ch1/main/av_stream";
    // let url = "rtsp://192.168.30.224:554/h264/ch1/main/av_stream";
    let mut client = RTSPClient::new(String::from(url));
    let result = client.connect().await;
    // .then(|result| {
    match result {
        Ok(_) => info!("Connected: {:?}", client.uri()),
        Err(error) => {
            error!("connect error: {}", error);
            // return Err(Box::new("connect error"));
            return Err("connect error".into());
        },
    }


    let addr = client.uri();
    // println!("Connected to server: {}", addr.unwrap());

    let mut builder = Request::builder();
    builder.method(Method::Describe).uri(URI::try_from(url).unwrap()).body(BytesMut::new());
    let request = builder.build().unwrap();

    // println!("request:{}", request);

    let result = client.send_request(request.clone()).await;

    // .then(|result| {
    match result {
        Ok(response) => {
            // info!("response: {:?}", response);

            if response.status_code() == StatusCode::Unauthorized{

                let www_auth_header = response.headers().typed_try_get::<WWWAuthenticate>();

                match www_auth_header{
                    Ok(Some(authenticate))=>{

                        let username = "admin".to_string();
                        // let password = "dm666666".to_string();
                        let password = "ZXECHW".to_string();

                        let auths=  authenticate.deref();

                        let mut res:Option<WWWAuthenticateSingle>= None;

                        for au in auths.iter() {
                            if res == None {
                                res = Some((*au).clone());
                            } else if au.method == AuthenticateMethod::Digest {
                                res = Some((*au).clone());
                                break;
                            }
                        }



                        client.setAuthenticator(Authenticator{username, password, authenticate: res.unwrap() });

                        let result = client.send_request(request.clone()).await;

                        match result {
                            Ok(response) => unsafe {
                                // info!("response: {:?}", response);

                                if response.status_code() == StatusCode::OK {
                                    let mut lines_vec: Vec<SdpLine> = Vec::new();

                                    let body = response.body();

                                    let content = str::from_utf8_unchecked(body.as_bytes());
                                    let result = parse_sdp(content, false);

                                    let mut video_url:Option<String> = None;

                                    match result{
                                        Ok(sesion) =>{
                                            println!("{:?}", sesion);

                                            for msection in sesion.media{

                                                println!("msection.get_type():{:?}", msection.get_type());
                                                println!("msection.get_attribute(SdpAttributeType::Control):{:?}", msection.get_attribute(SdpAttributeType::Control));


                                                if *(msection.get_type()) == SdpMediaValue::Video && msection.get_attribute(SdpAttributeType::Control).is_some() {

                                                    let url = msection.get_attribute(SdpAttributeType::Control).unwrap();

                                                    if let SdpAttribute::Control(url_string) = url.clone(){
                                                        println!("found video:{:?}", url_string.as_str());
                                                        video_url = Some(url_string);
                                                    }


                                                    break;
                                                }
                                                // else{
                                                //     println!("msection.get_type():{:?}", msection.get_type());
                                                //     println!("msection.get_attribute(SdpAttributeType::Control):{:?}", msection.get_attribute(SdpAttributeType::Control));
                                                // }
                                            }


                                        },
                                        Err(err) => {
                                            println!("{:?}", err);
                                        }
                                    }




                                    // assert!(parse_sdp_vector(&mut line_vec).is_ok());


                                    // lines.push(parse_sdp_line("v=0", 1)?);
                                    // for _ in 0..3 {
                                    //     lines.push(parse_sdp_line("a=sendrecv", 1)?);
                                    // }


                                    let mut builder = Request::builder();
                                    builder.method(Method::Setup).uri(URI::try_from(video_url.clone().unwrap().as_str()).unwrap()).body(BytesMut::new());
                                    builder.header(
                                        HeaderName::Transport,
                                        HeaderValue::try_from("RTP/AVP;unicast;client_port=63994-63995").unwrap(),
                                    );
                                    // Transport: RTP/AVP;unicast;client_port=63994-63995
                                    let request = builder.build().unwrap();

                                    // println!("request:{}", request);

                                    let result = client.send_request(request.clone()).await;

                                    match result {
                                        Ok(response) => {
                                            info!("response: {:?}", response);

                                            if response.status_code() == StatusCode::OK {
                                                let session = response.headers().typed_try_get::<Session>();
                                                let transport = response.headers().get(&HeaderName::Transport);

                                                info!("Transport:{:?}", transport.unwrap());

                                                let trans_str = transport.unwrap().as_str();
                                                let servers:Vec<&str> = trans_str.split(";").filter(|x|x.starts_with("server_port")).collect();

                                                // let s = "realm=\"IP Camera(C6496)\"";

                                                let mut port1:i32 = 0;
                                                let mut port2:i32  = 0;
                                                let s = servers[0];
                                                let r = Regex::new("(.+)=(\\d+)-(\\d+)").unwrap();
                                                if let Some(caps) = r.captures(s){
                                                    for j in 0..caps.len() {
                                                        println!("group {} : {}",  j, &caps[j]);
                                                    }

                                                    port1 = caps[2].parse().unwrap();
                                                    port2 = caps[3].parse().unwrap();
                                                }


                                                let local_addr = SocketAddrV4::from_str("192.168.1.135:63994").unwrap();
                                                let remote_addr = SocketAddrV4::from_str(format!("192.168.1.125:{}", port1).as_str()).unwrap();



                                                let local_addr2 = SocketAddrV4::from_str("192.168.1.135:63995").unwrap();
                                                let remote_addr2 = SocketAddrV4::from_str(format!("192.168.1.125:{}", port2).as_str()).unwrap();

                                                let mut rtp_session = RTPSession::newSession(SocketAddr::V4(local_addr), Some(SocketAddr::V4(remote_addr)));
                                                rtp_session.connect().await;
                                                // // let mut rtcp_session = RTPSession::newSession(SocketAddr::V4(local_addr2), Some(SocketAddr::V4(remote_addr2)));
                                                // // rtcp_session.connect().await;

                                                // tokio::spawn(Box::new(rtcp_session));


                                                // let mut socket = UdpSocket::bind(local_addr).await.unwrap();

                                                // socket.set_nonblocking(false).unwrap();




                                                if let Ok(Some(s)) = session {
                                                    let session_id = s.id();
                                                    let mut builder = Request::builder();
                                                    builder.method(Method::Play).uri(URI::try_from(video_url.clone().unwrap().as_str()).unwrap()).body(BytesMut::new());

                                                    builder.header(
                                                        HeaderName::Session,
                                                        HeaderValue::try_from(session_id.as_str()).unwrap(),
                                                    );
                                                    builder.header(
                                                        HeaderName::Range,
                                                        HeaderValue::try_from("npt=0.000-").unwrap(),
                                                    );
                                                    // Transport: RTP/AVP;unicast;client_port=63994-63995
                                                    let request = builder.build().unwrap();
                                                    client.send_request(request.clone()).await;

                                                    tokio::spawn(Box::new(rtp_session));

                                                    // let niter=4096;
                                                    // let mut buf = vec![0_u8;16384*niter];
                                                    // let mut shift=0_usize;
                                                    // for _i in 0..niter{
                                                    //     info!("udp recv_from");
                                                    //     let (num_bytes, _src_addr) = socket.recv_from(&mut buf[shift..]).await.unwrap();
                                                    //     shift+=num_bytes;
                                                    //     info!("udp received bytes:{}", shift);
                                                    //     // no other code here
                                                    // }
                                                }

                                                // while 0< 1 {
                                                //     sleep(Duration::from_millis(1000 * 1)).await;
                                                // }
                                            }
                                        },

                                        Err(err) => {
                                            info!("WWWAuthenticate error: {:?} ", err);
                                        }
                                    }
                                } else {}
                            }

                            _ => {}
                        }


                    },

                    Err(err) =>{
                        info!("WWWAuthenticate error: {:?} ", err);
                    }
                    _ => {}
                }

            }


        },
        Err(error) => info!("error sending request: {}", error),
    }

    // futures::future::ready(())
    // });

    sleep(Duration::from_millis(1000 * 1000)).await;
    Ok(())


}
