use nom::error::VerboseError;

use cloudmedia::protocol::sip::{Header, headers::parse::parse_content_length_header};

#[test]
fn write() {
    let header = Header::ContentLength(70);
    assert_eq!("Content-Length: 70".to_string(), format!("{}", header));
}

#[test]
fn read() {
    let remains = vec![];
    let header = Header::ContentLength(60);
    assert_eq!(
        Ok((remains.as_ref(), header)),
        parse_content_length_header::<VerboseError<&[u8]>>(b"Content-Length: 60\r\n")
    );
}
