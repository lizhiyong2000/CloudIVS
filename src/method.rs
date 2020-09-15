use std::fmt;
use std::str::FromStr;

use self::Method::*;

// TODO: Support non-standard methods, here and in codegen.

/// Representation of HTTP methods.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Method {
    Describe,
    Announce,
    GetParameter,
    Options,
    Pause,
    Play,
    Redirect,
    Setup,
    SetParameter,
    Teardown,
}

impl Method {


    /// Returns `true` if an HTTP request with the method represented by `self`
    /// always supports a payload.
    ///
    /// The following methods always support payloads:
    ///
    ///   * `PUT`, `POST`, `DELETE`, `PATCH`
    ///
    /// The following methods _do not_ always support payloads:
    ///
    ///   * `GET`, `HEAD`, `CONNECT`, `TRACE`, `OPTIONS`
    ///
    /// # Example
    ///
    /// ```rust
    /// # extern crate rocket;
    /// use rocket::http::Method;
    ///
    /// assert_eq!(Method::Get.supports_payload(), false);
    /// assert_eq!(Method::Post.supports_payload(), true);
    /// ```
    // #[inline]
    // pub fn supports_payload(self) -> bool {
    //     match self {
    //         Put | Post | Delete | Patch => true,
    //         Get | Head | Connect | Trace | Options => false,
    //     }
    // }

    /// Returns the string representation of `self`.
    ///
    /// # Example
    ///
    /// ```rust
    /// # extern crate rocket;
    /// use rocket::http::Method;
    ///
    /// assert_eq!(Method::Get.as_str(), "GET");
    /// ```
    #[inline]
    pub fn as_str(self) -> &'static str {
        match self {
            Describe => "DESCRIBE",
            Announce => "ANNOUNCE",
            GetParameter => "GET_PARAMETER",
            Options => "OPTIONS",
            Pause => "PAUSE",
            Play => "PLAY",
            Redirect => "REDIRECT",
            Setup => "SETUP",
            SetParameter => "SET_PARAMETER",
            Teardown => "TEARDOWN",
        }
    }
}

impl FromStr for Method {
    type Err = ();

    // According to the RFC, method names are case-sensitive. But some old
    // clients don't follow this, so we just do a case-insensitive match here.
    fn from_str(s: &str) -> Result<Method, ()> {
        match s {
            x if uncased::eq(x, Describe.as_str()) => Ok(Describe),
            x if uncased::eq(x, Announce.as_str()) => Ok(Announce),
            x if uncased::eq(x, GetParameter.as_str()) => Ok(GetParameter),
            x if uncased::eq(x, Options.as_str()) => Ok(Options),
            x if uncased::eq(x, Pause.as_str()) => Ok(Pause),
            x if uncased::eq(x, Play.as_str()) => Ok(Play),
            x if uncased::eq(x, Redirect.as_str()) => Ok(Redirect),
            x if uncased::eq(x, Setup.as_str()) => Ok(Setup),
            x if uncased::eq(x, SetParameter.as_str()) => Ok(SetParameter),
            x if uncased::eq(x, Teardown.as_str()) => Ok(Teardown),
            _ => Err(()),
        }
    }
}

impl fmt::Display for Method {
    #[inline(always)]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_str().fmt(f)
    }
}