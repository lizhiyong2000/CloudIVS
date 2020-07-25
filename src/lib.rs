// Strum contains all the trait definitions
#![allow(non_snake_case)]
#![allow(dead_code)]
#![allow(unused)]
#![recursion_limit="256"]
#![feature(int_error_matching)]

#[macro_use]
extern crate strum_macros;


extern crate strum;

#[macro_use]
extern crate trackable;
// Instead of #[macro_use], newer versions of rust should prefer
use strum_macros::{Display, EnumIter}; // etc.
//
extern crate byteorder;
extern crate bytecodec;


#[macro_use]
extern crate nom;
// extern crate serde;

#[macro_use]
extern crate log;

#[cfg(feature = "serialize")]
#[macro_use]
extern crate serde_derive;
#[cfg(feature = "serialize")]
extern crate serde;
extern crate url;

#[macro_use]
extern crate futures;



#[cfg(test)]
#[macro_use]
extern crate assert_matches;


mod common;
mod protocol;