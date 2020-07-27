use nom::error::VerboseError;

use cloudmedia::protocol::sip::{Header, headers::parse::parse_supported_header};

#[test]
fn write() {
    let header = Header::Supported(vec!["allowed".into(), "events".into()]);
    assert_eq!(
        "Supported: allowed,events".to_string(),
        format!("{}", header)
    );
}

#[test]
fn read() {
    let remains = vec![];
    let header = Header::Supported(vec!["allowed".into(), "events".into()]);
    assert_eq!(
        Ok((remains.as_ref(), header)),
        parse_supported_header::<VerboseError<&[u8]>>(b"Supported: allowed, events\r\n")
    );
}
