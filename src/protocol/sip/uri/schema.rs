use std::fmt;

use nom::{
    branch::alt,
    bytes::complete::tag_no_case,
    combinator::map,
    error::ParseError,
    IResult
};
use serde::{Deserialize, Serialize};

/// Sip URI Schema.
#[derive(Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
pub enum UriSchema {
    Sip,
    Sips,
}

impl fmt::Display for UriSchema {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            UriSchema::Sip => write!(f, "sip"),
            UriSchema::Sips => write!(f, "sips"),
        }
    }
}

/// Parse SIP URI schema. Only Accepts 'sip' and 'sips'.
pub fn parse_schema<'a, E: ParseError<&'a [u8]>>(input: &'a [u8]) -> IResult<&'a [u8], UriSchema, E> {
    alt::<_, _, E, _>((
        map(tag_no_case::<_, _, E>("sip"), |_| UriSchema::Sip),
        map(tag_no_case::<_, _, E>("sips"), |_| UriSchema::Sips)
    ))(input)
}
