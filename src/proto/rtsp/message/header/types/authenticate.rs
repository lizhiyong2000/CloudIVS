use crate::proto::rtsp::message::header::map::TypedHeader;
use crate::proto::rtsp::message::header::value::HeaderValue;
use crate::proto::rtsp::message::header::name::HeaderName;
use crate::proto::rtsp::message::syntax;
use linked_hash_set::LinkedHashSet;
// use crate::proto::rtsp::message::header::types::accept_ranges::AuthenticatePart;
use std::fmt::{Formatter, Display};
use std::fmt;
use std::str;
use std::convert::TryFrom;
use regex::Regex;
use std::str::FromStr;
use log::{info, error};
use crate::proto::rtsp::message::method::Method;
use itertools::Itertools;
use std::iter::{once, FromIterator};
use log4rs::append::Append;
use std::ops::{Deref, DerefMut};
// use md5::{Md5, Digest};

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum AuthenticateMethod{
    Digest,
    Basic,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct WWWAuthenticateSingle{
    pub(crate) method: AuthenticateMethod,
    pub(crate) parts: LinkedHashSet<AuthenticatePart>,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct WWWAuthenticate(LinkedHashSet<WWWAuthenticateSingle>);


impl Deref for WWWAuthenticate {
    type Target = LinkedHashSet<WWWAuthenticateSingle>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for WWWAuthenticate {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl FromIterator<WWWAuthenticateSingle> for WWWAuthenticate {
    fn from_iter<TIterator>(iterator: TIterator) -> Self
        where
            TIterator: IntoIterator<Item = WWWAuthenticateSingle>,
    {
        WWWAuthenticate(LinkedHashSet::from_iter(iterator))
    }
}

// #[derive(Clone, Debug, Eq, Hash, PartialEq)]
// pub struct WWWAuthenticate{
//     pub(crate) method: AuthenticateMethod,
//     pub(crate) parts: LinkedHashSet<AuthenticatePart>,
// }

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Authorization{
    pub(crate) method: AuthenticateMethod,
    pub(crate) parts: LinkedHashSet<AuthenticatePart>,
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

impl WWWAuthenticateSingle{
    pub fn realm(&self) -> String{
        for part in self.parts.iter(){
            if let AuthenticatePart::Realm(value) = part{
                return value.to_string();
            }
        }

        return "".to_string();
    }

    pub fn nonce(&self) -> String{
        for part in self.parts.iter(){
            if let AuthenticatePart::Nonce(value) = part{
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

        let value = self.parts.iter().map(AuthenticatePart::to_string).join(", ");



        match self.method{
            AuthenticateMethod::Basic =>{
                values.extend(once(unsafe { HeaderValue::from_string_unchecked(format!("Basic {}", value)) }));
            } ,

            AuthenticateMethod::Digest =>{
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


pub fn decoce_www_authenticate(value:&str) -> Result<WWWAuthenticateSingle, WWWAuthenticateError>{



    let v: Vec<&str> = value.splitn(2, |c| c == ' ').collect();

    // info!("decoce_www_authenticate: {:?}", v);

    if v.len() <2 {
        return Err(WWWAuthenticateError::Incomplete);
    }

    let mut auth_parts = LinkedHashSet::new();

    match v[0].to_uppercase().as_str()
    {
        "DIGEST" =>{

            let authenticates = v[1].split(',');
            for auth_part in authenticates {
                let part = AuthenticatePart::try_from(syntax::trim_whitespace(auth_part))?;
                auth_parts.insert(part);
            }
            return Ok(WWWAuthenticateSingle{ method:AuthenticateMethod::Digest, parts:auth_parts})
        },

        "BASIC" =>{

            let authenticates = v[1].split(',');
            for auth_part in authenticates {
                let part = AuthenticatePart::try_from(syntax::trim_whitespace(auth_part))?;
                auth_parts.insert(part);
            }
            return Ok(WWWAuthenticateSingle{ method:AuthenticateMethod::Basic, parts:auth_parts})
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
                let part = AuthenticatePart::try_from(syntax::trim_whitespace(auth_part))?;
                auth_parts.insert(part);
            }
            // return Ok(Authorization::Digest(auth_parts))
            return Ok(Authorization{ method:AuthenticateMethod::Digest, parts:auth_parts})
        },

        "BASIC" =>{

            let authenticates = v[1].split(',');
            for auth_part in authenticates {
                let part = AuthenticatePart::try_from(syntax::trim_whitespace(auth_part))?;
                auth_parts.insert(part);
            }
            // return Ok(Authorization::Basic(auth_parts))
            return Ok(Authorization{ method:AuthenticateMethod::Basic, parts:auth_parts})
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


        let mut accept = LinkedHashSet::<WWWAuthenticateSingle>::new();
        let mut present = false;
        for value in values {
            let authenticate = decoce_www_authenticate(value.as_str())?;
            accept.insert(authenticate);
            present = true;
        }
        if present {
            Ok(Some(WWWAuthenticate(accept)))
        } else {
            Ok(None)
        }


        // let value = match values.next() {
        //     Some(value) => value,
        //     None => return Ok(None),
        // };
        //
        // let next_value = values.next();
        //
        // if next_value.is_some() {
        //     print!("value 1:{:?}", value.as_str());
        //     print!("value 2:{:?}", next_value.unwrap().as_str());
        //     return Err(WWWAuthenticateError::MoreThanOneHeader);
        // }
        //
        // let authenticate = decoce_www_authenticate(value.as_str())?;
        // Ok(Some(authenticate))
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
pub enum AuthenticatePart {

    Realm(String),

    Nonce(String),

    Stale(bool),

    Username(String),

    Uri(String),

    Response(String),

    BasicResponse(String),

    Extension(String, String)


}

impl AuthenticatePart {
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
    // use rtsp::header::types::accept_ranges::AuthenticatePart;
    //
    // assert_eq!(AuthenticatePart::Clock.as_str(), "clock");
    // assert_eq!(AuthenticatePart::try_from("EXTENSION").unwrap().as_str(), "extension");
    // ```
    pub fn to_string(&self) -> String {
        use self::AuthenticatePart::*;

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

// impl AsRef<[u8]> for AuthenticatePart {
//     fn as_ref(&self) -> &[u8] {
//         self.as_str().as_bytes()
//     }
// }
//
// impl AsRef<str> for AuthenticatePart {
//     fn as_ref(&self) -> &str {
//         self.as_str()
//     }
// }

// impl Display for AuthenticatePart {
//     fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
//         write!(formatter, "{}", self.as_str())
//     }
// }

impl From<AuthenticatePart> for String {
    fn from(value: AuthenticatePart) -> Self {
        value.to_string()
    }
}
//
// impl PartialEq<[u8]> for AuthenticatePart {
//     fn eq(&self, other: &[u8]) -> bool {
//         self.as_str().as_bytes().eq_ignore_ascii_case(other)
//     }
// }
//
// impl PartialEq<AuthenticatePart> for [u8] {
//     fn eq(&self, other: &AuthenticatePart) -> bool {
//         self.eq_ignore_ascii_case(other.as_str().as_bytes())
//     }
// }
//
// impl<'auth_part> PartialEq<&'auth_part [u8]> for AuthenticatePart {
//     fn eq(&self, other: &&'auth_part [u8]) -> bool {
//         self.as_str().as_bytes().eq_ignore_ascii_case(other)
//     }
// }
//
// impl<'auth_part> PartialEq<AuthenticatePart> for &'auth_part [u8] {
//     fn eq(&self, other: &AuthenticatePart) -> bool {
//         self.eq_ignore_ascii_case(other.as_str().as_bytes())
//     }
// }
//
// impl PartialEq<str> for AuthenticatePart {
//     fn eq(&self, other: &str) -> bool {
//         self.as_str().eq_ignore_ascii_case(other)
//     }
// }
//
// impl PartialEq<AuthenticatePart> for str {
//     fn eq(&self, other: &AuthenticatePart) -> bool {
//         self.eq_ignore_ascii_case(other.as_str())
//     }
// }
//
// impl<'auth_part> PartialEq<&'auth_part str> for AuthenticatePart {
//     fn eq(&self, other: &&'auth_part str) -> bool {
//         self.as_str().eq_ignore_ascii_case(other)
//     }
// }
//
// impl<'auth_part> PartialEq<AuthenticatePart> for &'auth_part str {
//     fn eq(&self, other: &AuthenticatePart) -> bool {
//         self.eq_ignore_ascii_case(other.as_str())
//     }
// }

impl<'auth_part> TryFrom<&'auth_part [u8]> for AuthenticatePart {
    type Error = WWWAuthenticateError;

    fn try_from(value: &'auth_part [u8]) -> Result<Self, Self::Error> {
        use self::AuthenticatePart::*;

        let s = unsafe { str::from_utf8_unchecked(value) };
        let r = Regex::new("(\\w+)=\"(.+)\"").unwrap();
        if let Some(caps) = r.captures(s){

            let item = &caps[1];
            let val = &caps[2];

            // info!("AuthenticatePart-{} {}", item, val);

            match item.to_lowercase().as_str() {
                "realm" => return Ok(AuthenticatePart::Realm(val.to_string())),
                "nonce" => return Ok(AuthenticatePart::Nonce(val.to_string())),
                "stale" => {
                    let stale = bool::from_str(val.to_lowercase().as_str()) ;
                    match stale{
                        Ok(b) => return Ok(AuthenticatePart::Stale(b)),
                        Err(_) => return Err(WWWAuthenticateError::InvalidAuthenticateItem)
                    }

                },
                _ => return Ok(AuthenticatePart::Extension(item.to_string(), val.to_string()))
            }

        }
        else{
            error!("AuthenticatePart error- {}", s, );
            return Err(WWWAuthenticateError::InvalidAuthenticateItem);
        }

        return Err(WWWAuthenticateError::InvalidAuthenticateItem);
    }
}

impl<'auth_part> TryFrom<&'auth_part str> for AuthenticatePart {
    type Error = WWWAuthenticateError;

    fn try_from(value: &'auth_part str) -> Result<Self, Self::Error> {
        AuthenticatePart::try_from(value.as_bytes())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    // Authorization: Digest username="admin", realm="IP Camera(C6496)", nonce="75ebba210a21f5d87902abcc3343d9d0", uri="rtsp://192.168.30.224:554/h264/ch1/main/av_stream&channelId=2/", response="98962c804dbb3a95d7cdbbbe1a2234a4"
    #[test]
    fn test_digest_response() {
        let username = "admin";
        let password = "dm666666";
        let realm = "IP Camera(C6496)"      ;
        let nonce = "75ebba210a21f5d87902abcc3343d9d0";
        let method = "PLAY";
        let uri = "rtsp://192.168.30.224:554/h264/ch1/main/av_stream&channelId=2/";
        let response = Authorization::gen_digest_response(username,
                                                          password,
                                                          realm,
                                                          nonce,
                                                          method, uri);

        println!("response:{}", response);

        assert_eq!(response.as_str(), "98962c804dbb3a95d7cdbbbe1a2234a4");


    }

    #[test]
    fn test_digest_response_fail() {
        let username = "admin";
        let password = "dm666666";
        let realm = "IP Camera(C6496)"      ;
        let nonce = "06f85e0128e71d0b8c48373762bb62ba";
        let method = "SETUP";
        let uri = "rtsp://192.168.30.224:554/h264/ch1/main/av_stream";
        let response = Authorization::gen_digest_response(username,
                                                          password,
                                                          realm,
                                                          nonce,
                                                          method, uri);

        println!("response:{}", response);

        assert_eq!(response.as_str(), "dc3170611844e63d0705eb9cf9d42e7f");


    }

}

