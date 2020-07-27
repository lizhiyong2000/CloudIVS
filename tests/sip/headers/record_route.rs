use nom::error::VerboseError;

use cloudmedia::protocol::sip::{Header, headers::parse::parse_record_route_header};

#[test]
fn write() {
    let header = Header::RecordRoute("Softphone 1.0".into());
    assert_eq!(
        "Record-Route: Softphone 1.0".to_string(),
        format!("{}", header)
    );
}

#[test]
fn read() {
    let remains = vec![];
    let header = Header::RecordRoute("Softphone 1.0".into());
    assert_eq!(
        Ok((remains.as_ref(), header)),
        parse_record_route_header::<VerboseError<&[u8]>>(b"Record-Route: Softphone 1.0\r\n")
    );
}
