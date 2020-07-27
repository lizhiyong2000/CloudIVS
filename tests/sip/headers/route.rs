use nom::error::VerboseError;

use cloudmedia::protocol::sip::{Header, headers::parse::parse_route_header};

#[test]
fn write() {
    let header = Header::Route("Softphone 1.0".into());
    assert_eq!("Route: Softphone 1.0".to_string(), format!("{}", header));
}

#[test]
fn read() {
    let remains = vec![];
    let header = Header::Route("Softphone 1.0".into());
    assert_eq!(
        Ok((remains.as_ref(), header)),
        parse_route_header::<VerboseError<&[u8]>>(b"Route: Softphone 1.0\r\n")
    );
}
