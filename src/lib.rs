// Strum contains all the trait definitions
#![allow(non_snake_case)]
#![allow(dead_code)]
#![allow(unused)]
#![recursion_limit="256"]
#![feature(int_error_matching)]

#[cfg(test)]
#[macro_use]
extern crate assert_matches;
extern crate bytecodec;
// etc.
//
extern crate byteorder;
#[macro_use]
extern crate futures;
#[macro_use]
extern crate log;
#[macro_use]
extern crate nom;
#[cfg(feature = "serialize")]
extern crate serde;
#[cfg(feature = "serialize")]
#[macro_use]
extern crate serde_derive;
extern crate strum;
#[macro_use]
extern crate strum_macros;
#[macro_use]
extern crate trackable;
extern crate url;


// Instead of #[macro_use], newer versions of rust should prefer
use strum_macros::{Display, EnumIter};


// extern crate serde;


mod common;
mod protocol;