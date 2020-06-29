// Strum contains all the trait definitions
#![allow(non_snake_case)]
#![recursion_limit="256"]

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