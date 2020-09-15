use crate::request::Request;
use crate::response::Response;

pub enum RTSPMessage<'r>{
    Request(Request<'r>),
    Response(Response<'r>)
}
