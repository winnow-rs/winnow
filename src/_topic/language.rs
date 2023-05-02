//! # Elements of Programming Languages
//!
//! These are short recipes for accomplishing common tasks.
//!
//! * [Whitespace](#whitespace)
//!   + [Wrapper combinators that eat whitespace before and after a parser](#wrapper-combinators-that-eat-whitespace-before-and-after-a-parser)
//! * [Comments](#comments)
//!   + [`// C++/EOL-style comments`](#-ceol-style-comments)
//!   + [`/* C-style comments */`](#-c-style-comments-)
//! * [Identifiers](#identifiers)
//!   + [`Rust-Style Identifiers`](#rust-style-identifiers)
//! * [Literal Values](#literal-values)
//!   + [Escaped Strings](#escaped-strings)
//!   + [Integers](#integers)
//!     - [Hexadecimal](#hexadecimal)
//!     - [Octal](#octal)
//!     - [Binary](#binary)
//!     - [Decimal](#decimal)
//!   + [Floating Point Numbers](#floating-point-numbers)
//!
//! ## Whitespace
//!
//!
//!
//! ### Wrapper combinators that eat whitespace before and after a parser
//!
//! ```rust
//! use winnow::prelude::*;
//! use winnow::{
//!   error::ParseError,
//!   combinator::delimited,
//!   ascii::multispace0,
//! };
//!
//! /// A combinator that takes a parser `inner` and produces a parser that also consumes both leading and
//! /// trailing whitespace, returning the output of `inner`.
//! fn ws<'a, F, O, E: ParseError<&'a str>>(inner: F) -> impl Parser<&'a str, O, E>
//!   where
//!   F: Parser<&'a str, O, E>,
//! {
//!   delimited(
//!     multispace0,
//!     inner,
//!     multispace0
//!   )
//! }
//! ```
//!
//! To eat only trailing whitespace, replace `delimited(...)` with `terminated(&inner, multispace0)`.
//! Likewise, the eat only leading whitespace, replace `delimited(...)` with `preceded(multispace0,
//! &inner)`. You can use your own parser instead of `multispace0` if you want to skip a different set
//! of lexemes.
//!
//! ## Comments
//!
//! ### `// C++/EOL-style comments`
//!
//! This version uses `%` to start a comment, does not consume the newline character, and returns an
//! output of `()`.
//!
//! ```rust
//! use winnow::prelude::*;
//! use winnow::{
//!   error::ParseError,
//!   token::take_till1,
//! };
//!
//! pub fn peol_comment<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, (), E>
//! {
//!   ('%', take_till1("\n\r"))
//!     .void() // Output is thrown away.
//!     .parse_next(i)
//! }
//! ```
//!
//! ### `/* C-style comments */`
//!
//! Inline comments surrounded with sentinel tags `(*` and `*)`. This version returns an output of `()`
//! and does not handle nested comments.
//!
//! ```rust
//! use winnow::prelude::*;
//! use winnow::{
//!   error::ParseError,
//!   token::{tag, take_until0},
//! };
//!
//! pub fn pinline_comment<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, (), E> {
//!   (
//!     "(*",
//!     take_until0("*)"),
//!     "*)"
//!   )
//!     .void() // Output is thrown away.
//!     .parse_next(i)
//! }
//! ```
//!
//! ## Identifiers
//!
//! ### `Rust-Style Identifiers`
//!
//! Parsing identifiers that may start with a letter (or underscore) and may contain underscores,
//! letters and numbers may be parsed like this:
//!
//! ```rust
//! use winnow::prelude::*;
//! use winnow::{
//!   stream::AsChar,
//!   token::take_while,
//!   token::one_of,
//! };
//!
//! pub fn identifier(input: &str) -> IResult<&str, &str> {
//!   (
//!       one_of(|c: char| c.is_alpha() || c == '_'),
//!       take_while(0.., |c: char| c.is_alphanum() || c == '_')
//!   )
//!   .recognize()
//!   .parse_next(input)
//! }
//! ```
//!
//! Let's say we apply this to the identifier `hello_world123abc`. The first element of the tuple
//! would uses [`one_of`][crate::token::one_of] which would recognize `h`. The tuple ensures that
//! `ello_world123abc` will be piped to the next [`take_while`][crate::token::take_while] parser,
//! which recognizes every remaining character. However, the tuple returns a tuple of the results
//! of its sub-parsers. The [`recognize`][crate::Parser::recognize] parser produces a `&str` of the
//! input text that was parsed, which in this case is the entire `&str` `hello_world123abc`.
//!
//! ## Literal Values
//!
//! ### Escaped Strings
//!
//! ```rust
#![doc = include_str!("../../examples/string/parser.rs")]
//! ```
//!
//! ### Integers
//!
//! The following recipes all return string slices rather than integer values. How to obtain an
//! integer value instead is demonstrated for hexadecimal integers. The others are similar.
//!
//! The parsers allow the grouping character `_`, which allows one to group the digits by byte, for
//! example: `0xA4_3F_11_28`. If you prefer to exclude the `_` character, the lambda to convert from a
//! string slice to an integer value is slightly simpler. You can also strip the `_` from the string
//! slice that is returned, which is demonstrated in the second hexadecimal number parser.
//!
//! #### Hexadecimal
//!
//! The parser outputs the string slice of the digits without the leading `0x`/`0X`.
//!
//! ```rust
//! use winnow::prelude::*;
//! use winnow::{
//!   combinator::alt,
//!   combinator::{repeat},
//!   combinator::{preceded, terminated},
//!   token::one_of,
//!   token::tag,
//! };
//!
//! fn hexadecimal(input: &str) -> IResult<&str, &str> { // <'a, E: ParseError<&'a str>>
//!   preceded(
//!     alt(("0x", "0X")),
//!     repeat(1..,
//!       terminated(one_of("0123456789abcdefABCDEF"), repeat(0.., '_').map(|()| ()))
//!     ).map(|()| ()).recognize()
//!   ).parse_next(input)
//! }
//! ```
//!
//! If you want it to return the integer value instead, use map:
//!
//! ```rust
//! use winnow::prelude::*;
//! use winnow::{
//!   combinator::alt,
//!   combinator::{repeat},
//!   combinator::{preceded, terminated},
//!   token::one_of,
//!   token::tag,
//! };
//!
//! fn hexadecimal_value(input: &str) -> IResult<&str, i64> {
//!   preceded(
//!     alt(("0x", "0X")),
//!     repeat(1..,
//!       terminated(one_of("0123456789abcdefABCDEF"), repeat(0.., '_').map(|()| ()))
//!     ).map(|()| ()).recognize()
//!   ).try_map(
//!     |out: &str| i64::from_str_radix(&str::replace(&out, "_", ""), 16)
//!   ).parse_next(input)
//! }
//! ```
//!
//! #### Octal
//!
//! ```rust
//! use winnow::prelude::*;
//! use winnow::{
//!   combinator::alt,
//!   combinator::{repeat},
//!   combinator::{preceded, terminated},
//!   token::one_of,
//!   token::tag,
//! };
//!
//! fn octal(input: &str) -> IResult<&str, &str> {
//!   preceded(
//!     alt(("0o", "0O")),
//!     repeat(1..,
//!       terminated(one_of("01234567"), repeat(0.., '_').map(|()| ()))
//!     ).map(|()| ()).recognize()
//!   ).parse_next(input)
//! }
//! ```
//!
//! #### Binary
//!
//! ```rust
//! use winnow::prelude::*;
//! use winnow::{
//!   combinator::alt,
//!   combinator::{repeat},
//!   combinator::{preceded, terminated},
//!   token::one_of,
//!   token::tag,
//! };
//!
//! fn binary(input: &str) -> IResult<&str, &str> {
//!   preceded(
//!     alt(("0b", "0B")),
//!     repeat(1..,
//!       terminated(one_of("01"), repeat(0.., '_').map(|()| ()))
//!     ).map(|()| ()).recognize()
//!   ).parse_next(input)
//! }
//! ```
//!
//! #### Decimal
//!
//! ```rust
//! use winnow::prelude::*;
//! use winnow::{
//!   IResult,
//!   combinator::{repeat},
//!   combinator::terminated,
//!   token::one_of,
//! };
//!
//! fn decimal(input: &str) -> IResult<&str, &str> {
//!   repeat(1..,
//!     terminated(one_of("0123456789"), repeat(0.., '_').map(|()| ()))
//!   ).map(|()| ())
//!     .recognize()
//!     .parse_next(input)
//! }
//! ```
//!
//! ### Floating Point Numbers
//!
//! The following is adapted from [the Python parser by Valentin Lorentz](https://github.com/ProgVal/rust-python-parser/blob/master/src/numbers.rs).
//!
//! ```rust
//! use winnow::prelude::*;
//! use winnow::{
//!   combinator::alt,
//!   combinator::{repeat},
//!   combinator::opt,
//!   combinator::{preceded, terminated},
//!   token::one_of,
//! };
//!
//! fn float(input: &str) -> IResult<&str, &str> {
//!   alt((
//!     // Case one: .42
//!     (
//!       '.',
//!       decimal,
//!       opt((
//!         one_of("eE"),
//!         opt(one_of("+-")),
//!         decimal
//!       ))
//!     ).recognize()
//!     , // Case two: 42e42 and 42.42e42
//!     (
//!       decimal,
//!       opt(preceded(
//!         '.',
//!         decimal,
//!       )),
//!       one_of("eE"),
//!       opt(one_of("+-")),
//!       decimal
//!     ).recognize()
//!     , // Case three: 42. and 42.42
//!     (
//!       decimal,
//!       '.',
//!       opt(decimal)
//!     ).recognize()
//!   )).parse_next(input)
//! }
//!
//! fn decimal(input: &str) -> IResult<&str, &str> {
//!   repeat(1..,
//!     terminated(one_of("0123456789"), repeat(0.., '_').map(|()| ()))
//!   ).
//!   map(|()| ())
//!     .recognize()
//!     .parse_next(input)
//! }
//! ```
