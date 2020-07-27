// Strum contains all the trait definitions
#![allow(non_snake_case)]
#![allow(dead_code)]
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

fn main()
{
    let s = String::from("hello");
    // s.push_str(", world!"); // CANNOT BORROW AS MUTABLE
    println!("The value of s is: {}", s); // hello
    let mut t = String::from("hello");
    t.push_str(", world!"); // CANNOT BORROW AS MUTABLE
    println!("The value of t is: {}", t); // hello, world!
}