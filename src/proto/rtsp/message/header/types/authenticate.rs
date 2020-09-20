use crate::proto::rtsp::message::header::map::TypedHeader;
use crate::proto::rtsp::message::header::value::HeaderValue;
use crate::proto::rtsp::message::header::name::HeaderName;
use crate::proto::rtsp::message::syntax;
use linked_hash_set::LinkedHashSet;
// use crate::proto::rtsp::message::header::types::accept_ranges::WWWAuthenticatePart;
use std::fmt::{Formatter, Display};
use std::fmt;
use std::str;
use std::convert::TryFrom;
use regex::Regex;
use std::str::FromStr;
use log::{info, error};
use crate::proto::rtsp::message::method::Method;
use itertools::Itertools;
use std::iter::once;
// use md5::{Md5, Digest};

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum WWWAuthenticateMethod{
    Digest,
    Basic,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct WWWAuthenticate{
    pub(crate) method: WWWAuthenticateMethod,
    pub(crate) parts: LinkedHashSet<WWWAuthenticatePart>,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Authorization{
    pub(crate) method: WWWAuthenticateMethod,
    pub(crate) parts: LinkedHashSet<WWWAuthenticatePart>,
}

impl Authorization{


    pub fn gen_basic_response(username:&str, password :&str) -> String{
        let temp = format!("{}:{}", username, password);
        return base64::encode(temp.as_bytes());
    }
    //     // H(A1) = MD5(user:realm:pass)
    //     // H(A2) = MD5(method:digestURI)
    //     // response = MD5(H(A1):nonce:H(A2)
    pub fn gen_digest_response(username:&str, password :&str, realm :&str, nonce :&str, method:&str, uri:&str,) -> String{
        let ha1 = format!("{:x}", md5::compute(format!("{}:{}:{}", username, realm, password)));
        let ha2 = format!("{:x}", md5::compute(format!("{}:{}", method, uri)));

        let result = format!("{:x}",  md5::compute(format!("{}:{}:{}", ha1, nonce, ha2)));

        result
    }
}

impl WWWAuthenticate{
    pub fn realm(&self) -> String{
        for part in self.parts.iter(){
            if let WWWAuthenticatePart::Realm(value) = part{
                return value.to_string();
            }
        }

        return "".to_string();
    }

    pub fn nonce(&self) -> String{
        for part in self.parts.iter(){
            if let WWWAuthenticatePart::Nonce(value) = part{
                return value.to_string();
            }
        }

        return "".to_string();
    }
}



impl TypedHeader for Authorization{
    type DecodeError = WWWAuthenticateError;

    fn decode<'header, Iter>(values: &mut Iter) -> Result<Option<Self>, Self::DecodeError> where
        Self: Sized,
        Iter: Iterator<Item=&'header HeaderValue> {
        let value = match values.next() {
            Some(value) => value,
            None => return Ok(None),
        };

        if values.next().is_some() {
            return Err(WWWAuthenticateError::MoreThanOneHeader);
        }

        let authenticate = decode_autherization(value.as_str())?;
        Ok(Some(authenticate))
    }

    fn encode<Target>(&self, values: &mut Target) where
        Target: Extend<HeaderValue> {

        println!("{:?}", self);

        let value = self.parts.iter().map(WWWAuthenticatePart::to_string).join(", ");



        match self.method{
            WWWAuthenticateMethod::Basic =>{
                values.extend(once(unsafe { HeaderValue::from_string_unchecked(format!("Basic {}", value)) }));
            } ,

            WWWAuthenticateMethod::Digest =>{
                values.extend(once(unsafe { HeaderValue::from_string_unchecked(format!("Digest {}", value)) }));
            },
        }




        // unimplemented!()
    }

    /// Returns the statically assigned [`HeaderName`] for this header.
    fn header_name() -> &'static HeaderName {
        &HeaderName::Authorization
    }
}


pub fn decoce_www_authenticate(value:&str) -> Result<WWWAuthenticate, WWWAuthenticateError>{



    let v: Vec<&str> = value.splitn(2, |c| c == ' ').collect();

    info!("decoce_www_authenticate: {:?}", v);

    if v.len() <2 {
        return Err(WWWAuthenticateError::Incomplete);
    }

    let mut auth_parts = LinkedHashSet::new();

    match v[0].to_uppercase().as_str()
    {
        "DIGEST" =>{

            let authenticates = v[1].split(',');
            for auth_part in authenticates {
                let part = WWWAuthenticatePart::try_from(syntax::trim_whitespace(auth_part))?;
                auth_parts.insert(part);
            }
            return Ok(WWWAuthenticate{ method:WWWAuthenticateMethod::Digest, parts:auth_parts})
        },

        "BASIC" =>{

            let authenticates = v[1].split(',');
            for auth_part in authenticates {
                let part = WWWAuthenticatePart::try_from(syntax::trim_whitespace(auth_part))?;
                auth_parts.insert(part);
            }
            return Ok(WWWAuthenticate{ method:WWWAuthenticateMethod::Basic, parts:auth_parts})
        },

        _=>{
            info!("decoce_www_authenticate InvalidAuthenticateMethod");
            return Err(WWWAuthenticateError::InvalidAuthenticateMethod)
        } ,
    }


}


pub fn decode_autherization(value:&str) -> Result<Authorization, WWWAuthenticateError>{



    let v: Vec<&str> = value.splitn(2, |c| c == ' ').collect();

    info!("decode_autherization: {:?}", v);

    if v.len() <2 {
        return Err(WWWAuthenticateError::Incomplete);
    }

    let mut auth_parts = LinkedHashSet::new();

    match v[0].to_uppercase().as_str()
    {
        "DIGEST" =>{

            let authenticates = v[1].split(',');
            for auth_part in authenticates {
                let part = WWWAuthenticatePart::try_from(syntax::trim_whitespace(auth_part))?;
                auth_parts.insert(part);
            }
            // return Ok(Authorization::Digest(auth_parts))
            return Ok(Authorization{ method:WWWAuthenticateMethod::Digest, parts:auth_parts})
        },

        "BASIC" =>{

            let authenticates = v[1].split(',');
            for auth_part in authenticates {
                let part = WWWAuthenticatePart::try_from(syntax::trim_whitespace(auth_part))?;
                auth_parts.insert(part);
            }
            // return Ok(Authorization::Basic(auth_parts))
            return Ok(Authorization{ method:WWWAuthenticateMethod::Basic, parts:auth_parts})
        },

        _=>{
            info!("decode_autherization InvalidAuthenticateMethod");
            return Err(WWWAuthenticateError::InvalidAuthenticateMethod)
        } ,
    }


}


/// A possible error value when converting to a [`CSeq`] from [`HeaderName`]s.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum WWWAuthenticateError {
    /// The `"CSeq"` header was empty.
    Empty,

    /// The `"CSeq"` header was parsed, but the length exceeds the maximum length a CSeq can be.
    Incomplete,

    /// There was more than one `"CSeq"` header.
    MoreThanOneHeader,

    /// The `"CSeq"` header was parsed, but the length exceeds the maximum length a CSeq can be.
    InvalidAuthenticateMethod,

    /// The `"CSeq"` header was parsed, but the length exceeds the maximum length a CSeq can be.
    InvalidAuthenticateItem,
}


impl TypedHeader for WWWAuthenticate{
    type DecodeError = WWWAuthenticateError;

    fn decode<'header, Iter>(values: &mut Iter) -> Result<Option<Self>, Self::DecodeError> where
        Self: Sized,
        Iter: Iterator<Item=&'header HeaderValue> {
        let value = match values.next() {
            Some(value) => value,
            None => return Ok(None),
        };

        if values.next().is_some() {
            return Err(WWWAuthenticateError::MoreThanOneHeader);
        }

        let authenticate = decoce_www_authenticate(value.as_str())?;
        Ok(Some(authenticate))
    }

    fn encode<Target>(&self, values: &mut Target) where
        Target: Extend<HeaderValue> {
        unimplemented!()
    }

    /// Returns the statically assigned [`HeaderName`] for this header.
    fn header_name() -> &'static HeaderName {
        &HeaderName::WWWAuthenticate
    }
}

// Authorization: Digest username="admin", realm="IP Camera(C6496)", nonce="75ebba210a21f5d87902abcc3343d9d0", uri="rtsp://192.168.30.224:554/h264/ch1/main/av_stream&channelId=2/", response="98962c804dbb3a95d7cdbbbe1a2234a4"\r\n

/// Possible range formats that can be used in the `"Accept-Ranges"` header.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum WWWAuthenticatePart {

    Realm(String),

    Nonce(String),

    Stale(bool),

    Username(String),

    Uri(String),

    Response(String),

    BasicResponse(String),

    Extension(String, String)


}

impl WWWAuthenticatePart {
    // Returns a `&str` representation of the range format.
    //
    // The returned string is lowercase even if the range format originally was a non-lowercase
    // extension range format.
    //
    // # Examples
    //
    // ```
    // use std::convert::TryFrom;
    //
    // use rtsp::header::types::accept_ranges::WWWAuthenticatePart;
    //
    // assert_eq!(WWWAuthenticatePart::Clock.as_str(), "clock");
    // assert_eq!(WWWAuthenticatePart::try_from("EXTENSION").unwrap().as_str(), "extension");
    // ```
    pub fn to_string(&self) -> String {
        use self::WWWAuthenticatePart::*;

        match self {
            Realm(value) => format!("realm=\"{}\"", value),
            Nonce(value) => format!("nonce=\"{}\"", value),
            Stale(value) => format!("stale=\"{}\"", value),
            Username(value) => format!("username=\"{}\"", value),
            Uri(value) => format!("uri=\"{}\"", value),
            Response(value) => format!("response=\"{}\"", value),
            BasicResponse(value) => format!("{}", value),
            Extension(name, value)=> format!("{}=\"{}\"", name, value),

        }
    }

}

// impl AsRef<[u8]> for WWWAuthenticatePart {
//     fn as_ref(&self) -> &[u8] {
//         self.as_str().as_bytes()
//     }
// }
//
// impl AsRef<str> for WWWAuthenticatePart {
//     fn as_ref(&self) -> &str {
//         self.as_str()
//     }
// }

// impl Display for WWWAuthenticatePart {
//     fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
//         write!(formatter, "{}", self.as_str())
//     }
// }

impl From<WWWAuthenticatePart> for String {
    fn from(value: WWWAuthenticatePart) -> Self {
        value.to_string()
    }
}
//
// impl PartialEq<[u8]> for WWWAuthenticatePart {
//     fn eq(&self, other: &[u8]) -> bool {
//         self.as_str().as_bytes().eq_ignore_ascii_case(other)
//     }
// }
//
// impl PartialEq<WWWAuthenticatePart> for [u8] {
//     fn eq(&self, other: &WWWAuthenticatePart) -> bool {
//         self.eq_ignore_ascii_case(other.as_str().as_bytes())
//     }
// }
//
// impl<'auth_part> PartialEq<&'auth_part [u8]> for WWWAuthenticatePart {
//     fn eq(&self, other: &&'auth_part [u8]) -> bool {
//         self.as_str().as_bytes().eq_ignore_ascii_case(other)
//     }
// }
//
// impl<'auth_part> PartialEq<WWWAuthenticatePart> for &'auth_part [u8] {
//     fn eq(&self, other: &WWWAuthenticatePart) -> bool {
//         self.eq_ignore_ascii_case(other.as_str().as_bytes())
//     }
// }
//
// impl PartialEq<str> for WWWAuthenticatePart {
//     fn eq(&self, other: &str) -> bool {
//         self.as_str().eq_ignore_ascii_case(other)
//     }
// }
//
// impl PartialEq<WWWAuthenticatePart> for str {
//     fn eq(&self, other: &WWWAuthenticatePart) -> bool {
//         self.eq_ignore_ascii_case(other.as_str())
//     }
// }
//
// impl<'auth_part> PartialEq<&'auth_part str> for WWWAuthenticatePart {
//     fn eq(&self, other: &&'auth_part str) -> bool {
//         self.as_str().eq_ignore_ascii_case(other)
//     }
// }
//
// impl<'auth_part> PartialEq<WWWAuthenticatePart> for &'auth_part str {
//     fn eq(&self, other: &WWWAuthenticatePart) -> bool {
//         self.eq_ignore_ascii_case(other.as_str())
//     }
// }

impl<'auth_part> TryFrom<&'auth_part [u8]> for WWWAuthenticatePart {
    type Error = WWWAuthenticateError;

    fn try_from(value: &'auth_part [u8]) -> Result<Self, Self::Error> {
        use self::WWWAuthenticatePart::*;

        let s = unsafe { str::from_utf8_unchecked(value) };
        let r = Regex::new("(\\w+)=\"(.+)\"").unwrap();
        if let Some(caps) = r.captures(s){

            let item = &caps[1];
            let val = &caps[2];

            info!("WWWAuthenticatePart-{} {}", item, val);

            match item.to_lowercase().as_str() {
                "realm" => return Ok(WWWAuthenticatePart::Realm(val.to_string())),
                "nonce" => return Ok(WWWAuthenticatePart::Nonce(val.to_string())),
                "stale" => {
                    let stale = bool::from_str(val.to_lowercase().as_str()) ;
                    match stale{
                        Ok(b) => return Ok(WWWAuthenticatePart::Stale(b)),
                        Err(_) => return Err(WWWAuthenticateError::InvalidAuthenticateItem)
                    }

                },
                _ => return Ok(WWWAuthenticatePart::Extension(item.to_string(), val.to_string()))
            }

        }
        else{
            error!("WWWAuthenticatePart error- {}", s, );
            return Err(WWWAuthenticateError::InvalidAuthenticateItem);
        }

        return Err(WWWAuthenticateError::InvalidAuthenticateItem);
    }
}

impl<'auth_part> TryFrom<&'auth_part str> for WWWAuthenticatePart {
    type Error = WWWAuthenticateError;

    fn try_from(value: &'auth_part str) -> Result<Self, Self::Error> {
        WWWAuthenticatePart::try_from(value.as_bytes())
    }
}
