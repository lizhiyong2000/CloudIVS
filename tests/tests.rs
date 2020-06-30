// Strum contains all the trait definitions
#![allow(non_snake_case)]
#![recursion_limit="256"]

#[macro_use]
extern crate cloudmedia;



extern crate strum;
#[macro_use]
extern crate strum_macros;

#[macro_use]
extern crate trackable;
// Instead of #[macro_use], newer versions of rust should prefer
use strum_macros::{Display, EnumIter}; // etc.
//

extern crate handy_async;


#[macro_use]
extern crate nom;
extern crate serde;

#[cfg(test)]
#[macro_use]
extern crate assert_matches;


mod sip;
mod sdp;
