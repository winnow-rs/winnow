//! This example shows an example of how to parse an escaped string. The
//! rules for the string are similar to JSON and rust. A string is:
//!
//! - Enclosed by double quotes
//! - Can contain any raw unescaped code point besides \ and "
//! - Matches the following escape sequences: \b, \f, \n, \r, \t, \", \\, \/
//! - Matches code points like Rust: \u{XXXX}, where XXXX can be up to 6
//!   hex characters
//! - an escape followed by whitespace consumes all whitespace between the
//!   escape and the next non-whitespace character

#![cfg(feature = "alloc")]

mod parser;

use winnow::prelude::*;

fn main() {
    let data = "\"abc\"";
    let result = parser::parse_string::<()>(data).finish();
    match result {
        Ok(data) => println!("{}", data),
        Err(err) => println!("{:?}", err),
    }
}

#[test]
fn simple() {
    let data = "\"abc\"";
    let result = parser::parse_string::<()>(data).finish();
    assert_eq!(result, Ok(String::from("abc")));
}

#[test]
fn escaped() {
    let data = "\"tab:\\tafter tab, newline:\\nnew line, quote: \\\", emoji: \\u{1F602}, newline:\\nescaped whitespace: \\    abc\"";
    let result = parser::parse_string::<()>(data).finish();
    assert_eq!(
        result,
        Ok(String::from("tab:\tafter tab, newline:\nnew line, quote: \", emoji: 😂, newline:\nescaped whitespace: abc"))
    );
}
