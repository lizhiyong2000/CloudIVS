// Strum contains all the trait definitions
#![allow(non_snake_case)]
#![recursion_limit="256"]

#[cfg(test)]
#[macro_use]
extern crate assert_matches;
extern crate bytecodec;
#[macro_use]
extern crate cloudmedia;
#[macro_use]
extern crate nom;
extern crate serde;
extern crate strum;
#[macro_use]
extern crate strum_macros;
#[macro_use]
extern crate trackable;


// Instead of #[macro_use], newer versions of rust should prefer
use strum_macros::{Display, EnumIter};

// etc.
//


mod sip;
mod sdp;
