use nom::error::VerboseError;

use cloudmedia::protocol::sip::{Header, headers::parse::parse_warning_header};

#[test]
fn write() {
    let header = Header::Warning("Softphone 1.0".into());
    assert_eq!("Warning: Softphone 1.0".to_string(), format!("{}", header));
}

#[test]
fn read() {
    let remains = vec![];
    let header = Header::Warning("Softphone 1.0".into());
    assert_eq!(
        Ok((remains.as_ref(), header)),
        parse_warning_header::<VerboseError<&[u8]>>(b"Warning: Softphone 1.0\r\n")
    );
}
