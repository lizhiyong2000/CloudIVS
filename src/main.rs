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
use crate::proto::rtsp::connection::{OperationError, Authenticator};
use regex::Regex;
use crate::proto::rtsp::message::status::StatusCode;
use crate::proto::rtsp::message::header::map::HeaderMapExtension;
use crate::proto::rtsp::message::header::types::authenticate::WWWAuthenticate;
use crate::proto::rtsp::message::header::name::HeaderName;
use crate::proto::rtsp::message::header::value::HeaderValue;
use crate::proto::rtsp::message::header::types::Session;

// use crate::rtsp_client::RTSPClient;
// use crate::errors::ConnectionError;

mod proto;


#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {

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
    let url = "rtsp://192.168.30.224:554/h264/ch1/main/av_stream";
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
                        let password = "dm666666".to_string();

                        client.setAuthenticator(Authenticator{username, password, authenticate});

                        let result = client.send_request(request.clone()).await;

                        match result {
                            Ok(response) => {
                                // info!("response: {:?}", response);

                                if response.status_code() == StatusCode::OK {
                                    let mut builder = Request::builder();
                                    builder.method(Method::Setup).uri(URI::try_from("rtsp://192.168.30.224:554/h264/ch1/main/av_stream/trackID=1").unwrap()).body(BytesMut::new());
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
                                            // info!("response: {:?}", response);

                                            if response.status_code() == StatusCode::OK {

                                                let session = response.headers().typed_try_get::<Session>();

                                                if let Ok(Some(s)) = session{
                                                    let session_id = s.id();
                                                    let mut builder = Request::builder();
                                                    builder.method(Method::Play).uri(URI::try_from("rtsp://192.168.30.224:554/h264/ch1/main/av_stream").unwrap()).body(BytesMut::new());

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
                                                }




                                            }
                                        },

                                        Err(err) => {
                                            info!("WWWAuthenticate error: {:?} ", err);
                                        }
                                    }
                                }
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

    delay_for(Duration::from_millis(1000 * 1000)).await;
    Ok(())


}
