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

fn main() {
    let data = "\"abc\"";
    println!("EXAMPLE 1:\nParsing a simple input string: {}", data);
    let result = parser::parse_string::<()>(data);
    assert_eq!(result, Ok(("", String::from("abc"))));
    println!("Result: {}\n\n", result.unwrap().1);

    let data = "\"tab:\\tafter tab, newline:\\nnew line, quote: \\\", emoji: \\u{1F602}, newline:\\nescaped whitespace: \\    abc\"";
    println!(
    "EXAMPLE 2:\nParsing a string with escape sequences, newline literal, and escaped whitespace:\n\n{}\n",
    data
  );
    let result = parser::parse_string::<()>(data);
    assert_eq!(
    result,
    Ok((
      "",
      String::from("tab:\tafter tab, newline:\nnew line, quote: \", emoji: 😂, newline:\nescaped whitespace: abc")
    ))
  );
    println!("Result:\n\n{}", result.unwrap().1);
}
