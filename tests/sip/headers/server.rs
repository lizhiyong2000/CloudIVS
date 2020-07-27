use nom::error::VerboseError;

use cloudmedia::protocol::sip::{Header, headers::parse::parse_server_header};

#[test]
fn write() {
    let header = Header::Server("Softphone 1.0".into());
    assert_eq!("Server: Softphone 1.0".to_string(), format!("{}", header));
}

#[test]
fn read() {
    let remains = vec![];
    let header = Header::Server("Softphone 1.0".into());
    assert_eq!(
        Ok((remains.as_ref(), header)),
        parse_server_header::<VerboseError<&[u8]>>(b"Server: Softphone 1.0\r\n")
    );
}
