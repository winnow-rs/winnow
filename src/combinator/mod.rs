//! # List of parsers and combinators
//!
//! **Note**: this list is meant to provide a nicer way to find a nom parser than reading through the documentation on docs.rs. Function combinators are organized in module so they are a bit easier to find.
//!
//! Links present in this document will nearly always point to `complete` version of the parser. Most of the parsers also have a `streaming` version.
//!
//! ## Basic elements
//!
//! Those are used to recognize the lowest level elements of your grammar, like, "here is a dot", or "here is an big endian integer".
//!
//! | combinator | usage | input | output | comment |
//! |---|---|---|---|---|
//! | [char][crate::character::complete::char] | `char('a')` |  `"abc"` | `Ok(("bc", 'a'))` |Matches one character (works with non ASCII chars too) |
//! | [is_a][crate::bytes::complete::is_a] | ` is_a("ab")` |  `"ababc"` | `Ok(("c", "abab"))` |Matches a sequence of any of the characters passed as arguments|
//! | [is_not][crate::bytes::complete::is_not] | `is_not("cd")` |  `"ababc"` | `Ok(("c", "abab"))` |Matches a sequence of none of the characters passed as arguments|
//! | [one_of][crate::character::complete::one_of] | `one_of("abc")` |  `"abc"` | `Ok(("bc", 'a'))` |Matches one of the provided characters (works with non ASCII characters too)|
//! | [none_of][crate::character::complete::none_of] | `none_of("abc")` |  `"xyab"` | `Ok(("yab", 'x'))` |Matches anything but the provided characters|
//! | [tag][crate::bytes::complete::tag] | `tag("hello")` |  `"hello world"` | `Ok((" world", "hello"))` |Recognizes a specific suite of characters or bytes|
//! | [tag_no_case][crate::bytes::complete::tag_no_case] | `tag_no_case("hello")` |  `"HeLLo World"` | `Ok((" World", "HeLLo"))` |Case insensitive comparison. Note that case insensitive comparison is not well defined for unicode, and that you might have bad surprises|
//! | [take][crate::bytes::complete::take] | `take(4)` |  `"hello"` | `Ok(("o", "hell"))` |Takes a specific number of bytes or characters|
//! | [take_while][crate::bytes::complete::take_while] | `take_while(is_alphabetic)` |  `"abc123"` | `Ok(("123", "abc"))` |Returns the longest list of bytes for which the provided function returns true. `take_while1` does the same, but must return at least one character|
//! | [take_till][crate::bytes::complete::take_till] | `take_till(is_alphabetic)` |  `"123abc"` | `Ok(("abc", "123"))` |Returns the longest list of bytes or characters until the provided function returns true. `take_till1` does the same, but must return at least one character. This is the reverse behaviour from `take_while`: `take_till(f)` is equivalent to `take_while(\|c\| !f(c))`|
//! | [take_until][crate::bytes::complete::take_until] | `take_until("world")` |  `"Hello world"` | `Ok(("world", "Hello "))` |Returns the longest list of bytes or characters until the provided tag is found. `take_until1` does the same, but must return at least one character|
//!
//! ## Choice combinators
//!
//! | combinator | usage | input | output | comment |
//! |---|---|---|---|---|
//! | [alt][crate::branch::alt] | `alt((tag("ab"), tag("cd")))` |  `"cdef"` | `Ok(("ef", "cd"))` |Try a list of parsers and return the result of the first successful one|
//! | [permutation][crate::branch::permutation] | `permutation(tag("ab"), tag("cd"), tag("12"))` | `"cd12abc"` | `Ok(("c", ("ab", "cd", "12"))` |Succeeds when all its child parser have succeeded, whatever the order|
//!
//! ## Sequence combinators
//!
//! | combinator | usage | input | output | comment |
//! |---|---|---|---|---|
//! | [delimited][crate::sequence::delimited] | `delimited(char('('), take(2), char(')'))` | `"(ab)cd"` | `Ok(("cd", "ab"))` ||
//! | [preceded][crate::sequence::preceded] | `preceded(tag("ab"), tag("XY"))` | `"abXYZ"` | `Ok(("Z", "XY"))` ||
//! | [terminated][crate::sequence::terminated] | `terminated(tag("ab"), tag("XY"))` | `"abXYZ"` | `Ok(("Z", "ab"))` ||
//! | [pair][crate::sequence::pair] | `pair(tag("ab"), tag("XY"))` | `"abXYZ"` | `Ok(("Z", ("ab", "XY")))` ||
//! | [separated_pair][crate::sequence::separated_pair] | `separated_pair(tag("hello"), char(','), tag("world"))` | `"hello,world!"` | `Ok(("!", ("hello", "world")))` ||
//! | [tuple][crate::sequence::tuple] | `tuple((tag("ab"), tag("XY"), take(1)))` | `"abXYZ!"` | `Ok(("!", ("ab", "XY", "Z")))` |Chains parsers and assemble the sub results in a tuple. You can use as many child parsers as you can put elements in a tuple|
//!
//! ## Applying a parser multiple times
//!
//! | combinator | usage | input | output | comment |
//! |---|---|---|---|---|
//! | [count][crate::multi::count] | `count(take(2), 3)` | `"abcdefgh"` | `Ok(("gh", vec!["ab", "cd", "ef"]))` |Applies the child parser a specified number of times|
//! | [many0][crate::multi::many0] | `many0(tag("ab"))` |  `"abababc"` | `Ok(("c", vec!["ab", "ab", "ab"]))` |Applies the parser 0 or more times and returns the list of results in a Vec. `many1` does the same operation but must return at least one element|
//! | [many_m_n][crate::multi::many_m_n] | `many_m_n(1, 3, tag("ab"))` | `"ababc"` | `Ok(("c", vec!["ab", "ab"]))` |Applies the parser between m and n times (n included) and returns the list of results in a Vec|
//! | [many_till][crate::multi::many_till] | `many_till(tag( "ab" ), tag( "ef" ))` | `"ababefg"` | `Ok(("g", (vec!["ab", "ab"], "ef")))` |Applies the first parser until the second applies. Returns a tuple containing the list of results from the first in a Vec and the result of the second|
//! | [separated_list0][crate::multi::separated_list0] | `separated_list0(tag(","), tag("ab"))` | `"ab,ab,ab."` | `Ok((".", vec!["ab", "ab", "ab"]))` |`separated_list1` works like `separated_list0` but must returns at least one element|
//! | [fold_many0][crate::multi::fold_many0] | `fold_many0(be_u8, \|\| 0, \|acc, item\| acc + item)` | `[1, 2, 3]` | `Ok(([], 6))` |Applies the parser 0 or more times and folds the list of return values. The `fold_many1` version must apply the child parser at least one time|
//! | [fold_many_m_n][crate::multi::fold_many_m_n] | `fold_many_m_n(1, 2, be_u8, \|\| 0, \|acc, item\| acc + item)` | `[1, 2, 3]` | `Ok(([3], 3))` |Applies the parser between m and n times (n included) and folds the list of return value|
//! | [length_count][crate::multi::length_count] | `length_count(number, tag("ab"))` | `"2ababab"` | `Ok(("ab", vec!["ab", "ab"]))` |Gets a number from the first parser, then applies the second parser that many times|
//!
//! ## Integers
//!
//! Parsing integers from binary formats can be done in two ways: With parser functions, or combinators with configurable endianness.
//!
//! The following parsers could be found on [docs.rs number section][number/complete/index].
//!
//! - **configurable endianness:** [`i16`][crate::number::complete::i16], [`i32`][crate::number::complete::i32], [`i64`][crate::number::complete::i64], [`u16`][crate::number::complete::u16], [`u32`][crate::number::complete::u32], [`u64`][crate::number::complete::u64] are combinators that take as argument a [`nom::number::Endianness`][number/enum.Endianness], like this: `i16(endianness)`. If the parameter is `nom::number::Endianness::Big`, parse a big endian `i16` integer, otherwise a little endian `i16` integer.
//! - **fixed endianness**: The functions are prefixed by `be_` for big endian numbers, and by `le_` for little endian numbers, and the suffix is the type they parse to. As an example, `be_u32` parses a big endian unsigned integer stored in 32 bits.
//!   - [`be_f32`][crate::number::complete::be_f32], [`be_f64`][crate::number::complete::be_f64]: Big endian floating point numbers
//!   - [`le_f32`][crate::number::complete::le_f32], [`le_f64`][crate::number::complete::le_f64]: Little endian floating point numbers
//!   - [`be_i8`][crate::number::complete::be_i8], [`be_i16`][crate::number::complete::be_i16], [`be_i24`][crate::number::complete::be_i24], [`be_i32`][crate::number::complete::be_i32], [`be_i64`][crate::number::complete::be_i64], [`be_i128`][crate::number::complete::be_i128]: Big endian signed integers
//!   - [`be_u8`][crate::number::complete::be_u8], [`be_u16`][crate::number::complete::be_u16], [`be_u24`][crate::number::complete::be_u24], [`be_u32`][crate::number::complete::be_u32], [`be_u64`][crate::number::complete::be_u64], [`be_u128`][crate::number::complete::be_u128]: Big endian unsigned integers
//!   - [`le_i8`][crate::number::complete::le_i8], [`le_i16`][crate::number::complete::le_i16], [`le_i24`][crate::number::complete::le_i24], [`le_i32`][crate::number::complete::le_i32], [`le_i64`][crate::number::complete::le_i64], [`le_i128`][crate::number::complete::le_i128]: Little endian signed integers
//!   - [`le_u8`][crate::number::complete::le_u8], [`le_u16`][crate::number::complete::le_u16], [`le_u24`][crate::number::complete::le_u24], [`le_u32`][crate::number::complete::le_u32], [`le_u64`][crate::number::complete::le_u64], [`le_u128`][crate::number::complete::le_u128]: Little endian unsigned integers
//!
//! ## Streaming related
//!
//! - [`eof`][eof]: Returns its input if it is at the end of input data
//! - [`Parser::complete`][Parser::complete()]: Replaces an `Incomplete` returned by the child parser with an `Error`
//!
//! ## Modifiers
//!
//! - [`cond`][cond]: Conditional combinator. Wraps another parser and calls it if the condition is met
//! - [`Parser::flat_map`][crate::Parser::flat_map]: method to map a new parser from the output of the first parser, then apply that parser over the rest of the input
//! - [`Parser::value`][crate::Parser::value]: method to replace the result of a parser
//! - [`Parser::map`][crate::Parser::map]: method to map a function on the result of a parser
//! - [`Parser::and_then`][crate::Parser::and_then]: Applies a second parser over the output of the first one
//! - [`Parser::map_opt`][Parser::map_opt]: Maps a function returning an `Option` on the output of a parser
//! - [`Parser::map_res`][Parser::map_res]: Maps a function returning a `Result` on the output of a parser
//! - [`not`][not]: Returns a result only if the embedded parser returns `Error` or `Incomplete`. Does not consume the input
//! - [`opt`][opt]: Make the underlying parser optional
//! - [`peek`][peek]: Returns a result without consuming the input
//! - [`recognize`][recognize]: If the child parser was successful, return the consumed input as the produced value
//! - [`consumed`][consumed]: If the child parser was successful, return a tuple of the consumed input and the produced output.
//! - [`verify`][verify]: Returns the result of the child parser if it satisfies a verification function
//!
//! ## Error management and debugging
//!
//! - [`Parser::context`: Add context to the error if the parser fails
//! - [`dbg_dmp`][crate::error::dbg_dmp]: Prints a message and the input if the parser fails
//!
//! ## Text parsing
//!
//! - [`escaped`][crate::bytes::complete::escaped]: Matches a byte string with escaped characters
//! - [`escaped_transform`][crate::bytes::complete::escaped_transform]: Matches a byte string with escaped characters, and returns a new string with the escaped characters replaced
//!
//! ## Binary format parsing
//!
//! - [`length_data`][crate::multi::length_data]: Gets a number from the first parser, then takes a subslice of the input of that size, and returns that subslice
//! - [`length_value`][crate::multi::length_value]: Gets a number from the first parser, takes a subslice of the input of that size, then applies the second parser on that subslice. If the second parser returns `Incomplete`, `length_value` will return an error
//!
//! ## Bit stream parsing
//!
//! - [`bits`][crate::bits::bits]: Transforms the current input type (byte slice `&[u8]`) to a bit stream on which bit specific parsers and more general combinators can be applied
//! - [`bytes`][crate::bits/::bytes]: Transforms its bits stream input back into a byte slice for the underlying parser
//!
//! ## Remaining combinators
//!
//! - [`success`][success]: Returns a value without consuming any input, always succeeds
//! - [`fail`][fail]: Inversion of `success`. Always fails.
//!
//! ## Character test functions
//!
//! Use these functions with a combinator like `take_while`:
//!
//! - [`AsChar::is_alpha`][crate::input::AsChar::is_alpha]: Tests if byte is ASCII alphabetic: `[A-Za-z]`
//! - [`AsChar::is_alphanum`][crate::input::AsChar::is_alphanum]: Tests if byte is ASCII alphanumeric: `[A-Za-z0-9]`
//! - [`AsChar::is_dec_digit`][crate::input::AsChar::is_dec_digit]: Tests if byte is ASCII digit: `[0-9]`
//! - [`AsChar::is_hex_digit`][crate::input::AsChar::is_hex_digit]: Tests if byte is ASCII hex digit: `[0-9A-Fa-f]`
//! - [`AsChar::is_oct_digit`][crate::input::AsChar::is_oct_digit]: Tests if byte is ASCII octal digit: `[0-7]`
//! - [`AsChar::is_space`][crate::input::AsChar::is_space]: Tests if byte is ASCII space or tab: `[ \t]`
//! - [`AsChar::is_newline`][crate::input::AsChar::is_newline]: Tests if byte is ASCII newline: `[\n]`
//!
//! Alternatively there are ready to use functions:
//!
//! - [`alpha0`][crate::character::complete::alpha0]: Recognizes zero or more lowercase and uppercase alphabetic characters: `[a-zA-Z]`. [`alpha1`][crate::character::complete::alpha1] does the same but returns at least one character
//! - [`alphanumeric0`][crate::character::complete::alphanumeric0]: Recognizes zero or more numerical and alphabetic characters: `[0-9a-zA-Z]`. [`alphanumeric1`][crate::character::complete::alphanumeric1] does the same but returns at least one character
//! - [`anychar`][crate::character::complete::anychar]: Matches one byte as a character
//! - [`crlf`][crate::character::complete::crlf]: Recognizes the string `\r\n`
//! - [`digit0`][crate::character::complete::digit0]: Recognizes zero or more numerical characters: `[0-9]`. [`digit1`][crate::character::complete::digit1] does the same but returns at least one character
//! - [`double`][crate::number::complete::double]: Recognizes floating point number in a byte string and returns a `f64`
//! - [`float`][crate::number::complete::float]: Recognizes floating point number in a byte string and returns a `f32`
//! - [`hex_digit0`][crate::character::complete::hex_digit0]: Recognizes zero or more hexadecimal numerical characters: `[0-9A-Fa-f]`. [`hex_digit1`][crate::character::complete::hex_digit1] does the same but returns at least one character
//! - [`hex_u32`][crate::number::complete::hex_u32]: Recognizes a hex-encoded integer
//! - [`line_ending`][crate::character::complete::line_ending]: Recognizes an end of line (both `\n` and `\r\n`)
//! - [`multispace0`][crate::character::complete::multispace0]: Recognizes zero or more spaces, tabs, carriage returns and line feeds. [`multispace1`][crate::character::complete::multispace1] does the same but returns at least one character
//! - [`newline`][crate::character::complete::newline]: Matches a newline character `\n`
//! - [`not_line_ending`][crate::character::complete::not_line_ending]: Recognizes a string of any char except `\r` or `\n`
//! - [`oct_digit0`][crate::character::complete::oct_digit0]: Recognizes zero or more octal characters: `[0-7]`. [`oct_digit1`][crate::character::complete::oct_digit1] does the same but returns at least one character
//! - [`rest`][rest]: Return the remaining input
//! - [`space0`][crate::character::complete::space0]: Recognizes zero or more spaces and tabs. [`space1`][crate::character::complete::space1] does the same but returns at least one character
//! - [`tab`][crate::character::complete::tab]: Matches a tab character `\t`

#![allow(unused_imports)]

#[cfg(feature = "alloc")]
use crate::lib::std::boxed::Box;

use crate::error::{ErrorKind, FromExternalError, ParseError};
use crate::input::IntoOutput;
use crate::input::{AsChar, InputIter, InputLength, InputTakeAtPosition, ParseTo};
use crate::input::{Compare, CompareResult, Offset, Slice};
use crate::lib::std::borrow::Borrow;
use crate::lib::std::convert;
#[cfg(feature = "std")]
use crate::lib::std::fmt::Debug;
use crate::lib::std::mem::transmute;
use crate::lib::std::ops::{Range, RangeFrom, RangeTo};
use crate::IntoOutputIResult;
use crate::*;

#[cfg(test)]
mod tests;

/// Return the remaining input.
///
/// ```rust
/// # use nom::error::ErrorKind;
/// use nom::combinator::rest;
/// assert_eq!(rest::<_,(_, ErrorKind)>("abc"), Ok(("", "abc")));
/// assert_eq!(rest::<_,(_, ErrorKind)>(""), Ok(("", "")));
/// ```
#[inline]
pub fn rest<T, E: ParseError<T>>(input: T) -> IResult<T, <T as IntoOutput>::Output, E>
where
  T: Slice<RangeFrom<usize>>,
  T: InputLength,
  T: IntoOutput,
{
  Ok((input.slice(input.input_len()..), input)).into_output()
}

/// Return the length of the remaining input.
///
/// ```rust
/// # use nom::error::ErrorKind;
/// use nom::combinator::rest_len;
/// assert_eq!(rest_len::<_,(_, ErrorKind)>("abc"), Ok(("abc", 3)));
/// assert_eq!(rest_len::<_,(_, ErrorKind)>(""), Ok(("", 0)));
/// ```
#[inline]
pub fn rest_len<T, E: ParseError<T>>(input: T) -> IResult<T, usize, E>
where
  T: InputLength,
{
  let len = input.input_len();
  Ok((input, len))
}

/// Implementation of [`Parser::as_mut_parser`][Parser::as_mut_parser]
#[cfg_attr(nightly, warn(rustdoc::missing_doc_code_examples))]
pub struct MutParser<'p, P> {
  p: &'p mut P,
}

impl<'p, P> MutParser<'p, P> {
  pub(crate) fn new(p: &'p mut P) -> Self {
    Self { p }
  }
}

impl<'p, I, O, E, P: Parser<I, O, E>> Parser<I, O, E> for MutParser<'p, P> {
  fn parse(&mut self, i: I) -> IResult<I, O, E> {
    self.p.parse(i)
  }
}

/// Maps a function on the result of a parser.
///
/// **WARNING:** Deprecated, replaced with [`Parser::map`]
///
/// ```rust
/// use nom::{Err,error::ErrorKind, IResult,Parser};
/// use nom::character::digit1;
/// use nom::combinator::map;
/// # fn main() {
///
/// let mut parser = map(digit1, |s: &str| s.len());
///
/// // the parser will count how many characters were returned by digit1
/// assert_eq!(parser.parse("123456"), Ok(("", 6)));
///
/// // this will fail if digit1 fails
/// assert_eq!(parser.parse("abc"), Err(Err::Error(("abc", ErrorKind::Digit))));
/// # }
/// ```
#[deprecated(since = "8.0.0", note = "Replaced with `Parser::map")]
pub fn map<I, O1, O2, E, F, G>(mut parser: F, mut f: G) -> impl FnMut(I) -> IResult<I, O2, E>
where
  F: Parser<I, O1, E>,
  G: FnMut(O1) -> O2,
{
  move |input: I| {
    let (input, o1) = parser.parse(input)?;
    Ok((input, f(o1)))
  }
}

/// Implementation of [`Parser::map`]
#[cfg_attr(nightly, warn(rustdoc::missing_doc_code_examples))]
pub struct Map<F, G, O1> {
  f: F,
  g: G,
  phantom: core::marker::PhantomData<O1>,
}

impl<F, G, O1> Map<F, G, O1> {
  pub(crate) fn new(f: F, g: G) -> Self {
    Self {
      f,
      g,
      phantom: Default::default(),
    }
  }
}

impl<'a, I, O1, O2, E, F: Parser<I, O1, E>, G: Fn(O1) -> O2> Parser<I, O2, E> for Map<F, G, O1> {
  fn parse(&mut self, i: I) -> IResult<I, O2, E> {
    match self.f.parse(i) {
      Err(e) => Err(e),
      Ok((i, o)) => Ok((i, (self.g)(o))),
    }
  }
}

/// Applies a function returning a `Result` over the result of a parser.
///
/// ```rust
/// # use nom::{Err,error::ErrorKind, IResult};
/// use nom::character::digit1;
/// use nom::combinator::map_res;
/// # fn main() {
///
/// let mut parse = map_res(digit1, |s: &str| s.parse::<u8>());
///
/// // the parser will convert the result of digit1 to a number
/// assert_eq!(parse("123"), Ok(("", 123)));
///
/// // this will fail if digit1 fails
/// assert_eq!(parse("abc"), Err(Err::Error(("abc", ErrorKind::Digit))));
///
/// // this will fail if the mapped function fails (a `u8` is too small to hold `123456`)
/// assert_eq!(parse("123456"), Err(Err::Error(("123456", ErrorKind::MapRes))));
/// # }
/// ```
pub fn map_res<I: Clone, O1, O2, E: FromExternalError<I, E2>, E2, F, G>(
  mut parser: F,
  mut f: G,
) -> impl FnMut(I) -> IResult<I, O2, E>
where
  F: Parser<I, O1, E>,
  G: FnMut(O1) -> Result<O2, E2>,
{
  move |input: I| {
    let i = input.clone();
    let (input, o1) = parser.parse(input)?;
    match f(o1) {
      Ok(o2) => Ok((input, o2)),
      Err(e) => Err(Err::Error(E::from_external_error(i, ErrorKind::MapRes, e))),
    }
  }
}

/// Implementation of [`Parser::map_res`]
#[cfg_attr(nightly, warn(rustdoc::missing_doc_code_examples))]
pub struct MapRes<F, G, O1> {
  f: F,
  g: G,
  phantom: core::marker::PhantomData<O1>,
}

impl<F, G, O1> MapRes<F, G, O1> {
  pub(crate) fn new(f: F, g: G) -> Self {
    Self {
      f,
      g,
      phantom: Default::default(),
    }
  }
}

impl<I, O1, O2, E, E2, F, G> Parser<I, O2, E> for MapRes<F, G, O1>
where
  I: Clone,
  F: Parser<I, O1, E>,
  G: FnMut(O1) -> Result<O2, E2>,
  E: FromExternalError<I, E2>,
{
  fn parse(&mut self, input: I) -> IResult<I, O2, E> {
    let i = input.clone();
    let (input, o1) = self.f.parse(input)?;
    match (self.g)(o1) {
      Ok(o2) => Ok((input, o2)),
      Err(e) => Err(Err::Error(E::from_external_error(i, ErrorKind::MapRes, e))),
    }
  }
}

/// Applies a function returning an `Option` over the result of a parser.
///
/// ```rust
/// # use nom::{Err,error::ErrorKind, IResult};
/// use nom::character::digit1;
/// use nom::combinator::map_opt;
/// # fn main() {
///
/// let mut parse = map_opt(digit1, |s: &str| s.parse::<u8>().ok());
///
/// // the parser will convert the result of digit1 to a number
/// assert_eq!(parse("123"), Ok(("", 123)));
///
/// // this will fail if digit1 fails
/// assert_eq!(parse("abc"), Err(Err::Error(("abc", ErrorKind::Digit))));
///
/// // this will fail if the mapped function fails (a `u8` is too small to hold `123456`)
/// assert_eq!(parse("123456"), Err(Err::Error(("123456", ErrorKind::MapOpt))));
/// # }
/// ```
pub fn map_opt<I: Clone, O1, O2, E: ParseError<I>, F, G>(
  mut parser: F,
  mut f: G,
) -> impl FnMut(I) -> IResult<I, O2, E>
where
  F: Parser<I, O1, E>,
  G: FnMut(O1) -> Option<O2>,
{
  move |input: I| {
    let i = input.clone();
    let (input, o1) = parser.parse(input)?;
    match f(o1) {
      Some(o2) => Ok((input, o2)),
      None => Err(Err::Error(E::from_error_kind(i, ErrorKind::MapOpt))),
    }
  }
}

/// Implementation of [`Parser::map_opt`]
#[cfg_attr(nightly, warn(rustdoc::missing_doc_code_examples))]
pub struct MapOpt<F, G, O1> {
  f: F,
  g: G,
  phantom: core::marker::PhantomData<O1>,
}

impl<F, G, O1> MapOpt<F, G, O1> {
  pub(crate) fn new(f: F, g: G) -> Self {
    Self {
      f,
      g,
      phantom: Default::default(),
    }
  }
}

impl<I, O1, O2, E, F, G> Parser<I, O2, E> for MapOpt<F, G, O1>
where
  I: Clone,
  F: Parser<I, O1, E>,
  G: FnMut(O1) -> Option<O2>,
  E: ParseError<I>,
{
  fn parse(&mut self, input: I) -> IResult<I, O2, E> {
    let i = input.clone();
    let (input, o1) = self.f.parse(input)?;
    match (self.g)(o1) {
      Some(o2) => Ok((input, o2)),
      None => Err(Err::Error(E::from_error_kind(i, ErrorKind::MapOpt))),
    }
  }
}

/// Applies a parser over the result of another one.
///
/// **WARNING:** Deprecated, replaced with [`Parser::and_then`]
///
/// ```rust
/// # use nom::{Err,error::ErrorKind, IResult};
/// use nom::character::digit1;
/// use nom::bytes::take;
/// use nom::combinator::map_parser;
/// # fn main() {
///
/// let mut parse = map_parser(take(5u8), digit1);
///
/// assert_eq!(parse("12345"), Ok(("", "12345")));
/// assert_eq!(parse("123ab"), Ok(("", "123")));
/// assert_eq!(parse("123"), Err(Err::Error(("123", ErrorKind::Eof))));
/// # }
/// ```
#[deprecated(since = "8.0.0", note = "Replaced with `Parser::and_then")]
pub fn map_parser<I, O1, O2, E: ParseError<I>, F, G>(
  mut parser: F,
  mut applied_parser: G,
) -> impl FnMut(I) -> IResult<I, O2, E>
where
  F: Parser<I, O1, E>,
  G: Parser<O1, O2, E>,
{
  move |input: I| {
    let (input, o1) = parser.parse(input)?;
    let (_, o2) = applied_parser.parse(o1)?;
    Ok((input, o2))
  }
}

/// Implementation of [`Parser::and_then`]
#[cfg_attr(nightly, warn(rustdoc::missing_doc_code_examples))]
pub struct AndThen<F, G, O1> {
  f: F,
  g: G,
  phantom: core::marker::PhantomData<O1>,
}

impl<F, G, O1> AndThen<F, G, O1> {
  pub(crate) fn new(f: F, g: G) -> Self {
    Self {
      f,
      g,
      phantom: Default::default(),
    }
  }
}

impl<'a, I, O1, O2, E, F: Parser<I, O1, E>, G: Parser<O1, O2, E>> Parser<I, O2, E>
  for AndThen<F, G, O1>
{
  fn parse(&mut self, i: I) -> IResult<I, O2, E> {
    let (i, o1) = self.f.parse(i)?;
    let (_, o2) = self.g.parse(o1)?;
    Ok((i, o2))
  }
}

/// Creates a new parser from the output of the first parser, then apply that parser over the rest of the input.
///
/// **WARNING:** Deprecated, replaced with [`Parser::flat_map`]
///
/// ```rust
/// # use nom::{Err,error::ErrorKind, IResult};
/// use nom::bytes::take;
/// use nom::number::u8;
/// use nom::combinator::flat_map;
/// # fn main() {
///
/// let mut parse = flat_map(u8, take);
///
/// assert_eq!(parse(&[2, 0, 1, 2][..]), Ok((&[2][..], &[0, 1][..])));
/// assert_eq!(parse(&[4, 0, 1, 2][..]), Err(Err::Error((&[0, 1, 2][..], ErrorKind::Eof))));
/// # }
/// ```
#[deprecated(since = "8.0.0", note = "Replaced with `Parser::flat_map")]
pub fn flat_map<I, O1, O2, E: ParseError<I>, F, G, H>(
  mut parser: F,
  mut applied_parser: G,
) -> impl FnMut(I) -> IResult<I, O2, E>
where
  F: Parser<I, O1, E>,
  G: FnMut(O1) -> H,
  H: Parser<I, O2, E>,
{
  move |input: I| {
    let (input, o1) = parser.parse(input)?;
    applied_parser(o1).parse(input)
  }
}

/// Implementation of [`Parser::flat_map`]
#[cfg_attr(nightly, warn(rustdoc::missing_doc_code_examples))]
pub struct FlatMap<F, G, O1> {
  f: F,
  g: G,
  phantom: core::marker::PhantomData<O1>,
}

impl<F, G, O1> FlatMap<F, G, O1> {
  pub(crate) fn new(f: F, g: G) -> Self {
    Self {
      f,
      g,
      phantom: Default::default(),
    }
  }
}

impl<'a, I, O1, O2, E, F: Parser<I, O1, E>, G: Fn(O1) -> H, H: Parser<I, O2, E>> Parser<I, O2, E>
  for FlatMap<F, G, O1>
{
  fn parse(&mut self, i: I) -> IResult<I, O2, E> {
    let (i, o1) = self.f.parse(i)?;
    (self.g)(o1).parse(i)
  }
}

/// Optional parser, will return `None` on [`Err::Error`].
///
/// To chain an error up, see [`cut`].
///
/// ```rust
/// # use nom::{Err,error::ErrorKind, IResult};
/// use nom::combinator::opt;
/// use nom::character::alpha1;
/// # fn main() {
///
/// fn parser(i: &str) -> IResult<&str, Option<&str>> {
///   opt(alpha1)(i)
/// }
///
/// assert_eq!(parser("abcd;"), Ok((";", Some("abcd"))));
/// assert_eq!(parser("123;"), Ok(("123;", None)));
/// # }
/// ```
pub fn opt<I: Clone, O, E: ParseError<I>, F>(mut f: F) -> impl FnMut(I) -> IResult<I, Option<O>, E>
where
  F: Parser<I, O, E>,
{
  move |input: I| {
    let i = input.clone();
    match f.parse(input) {
      Ok((i, o)) => Ok((i, Some(o))),
      Err(Err::Error(_)) => Ok((i, None)),
      Err(e) => Err(e),
    }
  }
}

/// Implementation of [`Parser::and`]
#[cfg_attr(nightly, warn(rustdoc::missing_doc_code_examples))]
pub struct And<F, G> {
  f: F,
  g: G,
}

impl<F, G> And<F, G> {
  pub(crate) fn new(f: F, g: G) -> Self {
    Self { f, g }
  }
}

impl<'a, I, O1, O2, E, F: Parser<I, O1, E>, G: Parser<I, O2, E>> Parser<I, (O1, O2), E>
  for And<F, G>
{
  fn parse(&mut self, i: I) -> IResult<I, (O1, O2), E> {
    let (i, o1) = self.f.parse(i)?;
    let (i, o2) = self.g.parse(i)?;
    Ok((i, (o1, o2)))
  }
}

/// Implementation of [`Parser::or`]
#[cfg_attr(nightly, warn(rustdoc::missing_doc_code_examples))]
pub struct Or<F, G> {
  f: F,
  g: G,
}

impl<F, G> Or<F, G> {
  pub(crate) fn new(f: F, g: G) -> Self {
    Self { f, g }
  }
}

impl<'a, I: Clone, O, E: crate::error::ParseError<I>, F: Parser<I, O, E>, G: Parser<I, O, E>>
  Parser<I, O, E> for Or<F, G>
{
  fn parse(&mut self, i: I) -> IResult<I, O, E> {
    match self.f.parse(i.clone()) {
      Err(Err::Error(e1)) => match self.g.parse(i) {
        Err(Err::Error(e2)) => Err(Err::Error(e1.or(e2))),
        res => res,
      },
      res => res,
    }
  }
}

/// Calls the parser if the condition is met.
///
/// ```rust
/// # use nom::{Err, error::{Error, ErrorKind}, IResult};
/// use nom::combinator::cond;
/// use nom::character::alpha1;
/// # fn main() {
///
/// fn parser(b: bool, i: &str) -> IResult<&str, Option<&str>> {
///   cond(b, alpha1)(i)
/// }
///
/// assert_eq!(parser(true, "abcd;"), Ok((";", Some("abcd"))));
/// assert_eq!(parser(false, "abcd;"), Ok(("abcd;", None)));
/// assert_eq!(parser(true, "123;"), Err(Err::Error(Error::new("123;", ErrorKind::Alpha))));
/// assert_eq!(parser(false, "123;"), Ok(("123;", None)));
/// # }
/// ```
pub fn cond<I, O, E: ParseError<I>, F>(
  b: bool,
  mut f: F,
) -> impl FnMut(I) -> IResult<I, Option<O>, E>
where
  F: Parser<I, O, E>,
{
  move |input: I| {
    if b {
      match f.parse(input) {
        Ok((i, o)) => Ok((i, Some(o))),
        Err(e) => Err(e),
      }
    } else {
      Ok((input, None))
    }
  }
}

/// Tries to apply its parser without consuming the input.
///
/// ```rust
/// # use nom::{Err,error::ErrorKind, IResult};
/// use nom::combinator::peek;
/// use nom::character::alpha1;
/// # fn main() {
///
/// let mut parser = peek(alpha1);
///
/// assert_eq!(parser("abcd;"), Ok(("abcd;", "abcd")));
/// assert_eq!(parser("123;"), Err(Err::Error(("123;", ErrorKind::Alpha))));
/// # }
/// ```
pub fn peek<I: Clone, O, E: ParseError<I>, F>(mut f: F) -> impl FnMut(I) -> IResult<I, O, E>
where
  F: Parser<I, O, E>,
{
  move |input: I| {
    let i = input.clone();
    match f.parse(input) {
      Ok((_, o)) => Ok((i, o)),
      Err(e) => Err(e),
    }
  }
}

/// returns its input if it is at the end of input data
///
/// When we're at the end of the data, this combinator
/// will succeed
///
/// ```
/// # use std::str;
/// # use nom::{Err, error::ErrorKind, IResult};
/// # use nom::combinator::eof;
///
/// # fn main() {
/// let parser = eof;
/// assert_eq!(parser("abc"), Err(Err::Error(("abc", ErrorKind::Eof))));
/// assert_eq!(parser(""), Ok(("", "")));
/// # }
/// ```
pub fn eof<I, E: ParseError<I>>(input: I) -> IResult<I, <I as IntoOutput>::Output, E>
where
  I: InputLength,
  I: Clone,
  I: IntoOutput,
{
  if input.input_len() == 0 {
    let clone = input.clone();
    Ok((input, clone)).into_output()
  } else {
    Err(Err::Error(E::from_error_kind(input, ErrorKind::Eof)))
  }
}

/// Transforms Incomplete into `Error`.
///
/// ```rust
/// # use nom::{Err,error::ErrorKind, IResult, input::Streaming};
/// use nom::bytes::take;
/// use nom::combinator::complete;
/// # fn main() {
///
/// let mut parser = complete(take(5u8));
///
/// assert_eq!(parser(Streaming("abcdefg")), Ok((Streaming("fg"), "abcde")));
/// assert_eq!(parser(Streaming("abcd")), Err(Err::Error((Streaming("abcd"), ErrorKind::Complete))));
/// # }
/// ```
pub fn complete<I: Clone, O, E: ParseError<I>, F>(mut f: F) -> impl FnMut(I) -> IResult<I, O, E>
where
  F: Parser<I, O, E>,
{
  move |input: I| {
    let i = input.clone();
    match f.parse(input) {
      Err(Err::Incomplete(_)) => Err(Err::Error(E::from_error_kind(i, ErrorKind::Complete))),
      rest => rest,
    }
  }
}

/// Implementation of [`Parser::complete`]
#[cfg_attr(nightly, warn(rustdoc::missing_doc_code_examples))]
pub struct Complete<F> {
  f: F,
}

impl<F> Complete<F> {
  pub(crate) fn new(f: F) -> Self {
    Self { f }
  }
}

impl<F, I, O, E> Parser<I, O, E> for Complete<F>
where
  I: Clone,
  F: Parser<I, O, E>,
  E: ParseError<I>,
{
  fn parse(&mut self, input: I) -> IResult<I, O, E> {
    let i = input.clone();
    match (self.f).parse(input) {
      Err(Err::Incomplete(_)) => Err(Err::Error(E::from_error_kind(i, ErrorKind::Complete))),
      rest => rest,
    }
  }
}

/// Succeeds if all the input has been consumed by its child parser.
///
/// ```rust
/// # use nom::{Err,error::ErrorKind, IResult};
/// use nom::combinator::all_consuming;
/// use nom::character::alpha1;
/// # fn main() {
///
/// let mut parser = all_consuming(alpha1);
///
/// assert_eq!(parser("abcd"), Ok(("", "abcd")));
/// assert_eq!(parser("abcd;"),Err(Err::Error((";", ErrorKind::Eof))));
/// assert_eq!(parser("123abcd;"),Err(Err::Error(("123abcd;", ErrorKind::Alpha))));
/// # }
/// ```
pub fn all_consuming<I, O, E: ParseError<I>, F>(mut f: F) -> impl FnMut(I) -> IResult<I, O, E>
where
  I: InputLength,
  F: Parser<I, O, E>,
{
  move |input: I| {
    let (input, res) = f.parse(input)?;
    if input.input_len() == 0 {
      Ok((input, res))
    } else {
      Err(Err::Error(E::from_error_kind(input, ErrorKind::Eof)))
    }
  }
}

/// Returns the result of the child parser if it satisfies a verification function.
///
/// The verification function takes as argument a reference to the output of the
/// parser.
///
/// ```rust
/// # use nom::{Err,error::ErrorKind, IResult};
/// use nom::combinator::verify;
/// use nom::character::alpha1;
/// # fn main() {
///
/// let mut parser = verify(alpha1, |s: &str| s.len() == 4);
///
/// assert_eq!(parser("abcd"), Ok(("", "abcd")));
/// assert_eq!(parser("abcde"), Err(Err::Error(("abcde", ErrorKind::Verify))));
/// assert_eq!(parser("123abcd;"),Err(Err::Error(("123abcd;", ErrorKind::Alpha))));
/// # }
/// ```
pub fn verify<I: Clone, O1, O2, E: ParseError<I>, F, G>(
  mut first: F,
  second: G,
) -> impl FnMut(I) -> IResult<I, O1, E>
where
  F: Parser<I, O1, E>,
  G: Fn(&O2) -> bool,
  O1: Borrow<O2>,
  O2: ?Sized,
{
  move |input: I| {
    let i = input.clone();
    let (input, o) = first.parse(input)?;

    if second(o.borrow()) {
      Ok((input, o))
    } else {
      Err(Err::Error(E::from_error_kind(i, ErrorKind::Verify)))
    }
  }
}

/// Implementation of [`Parser::verify`]
#[cfg_attr(nightly, warn(rustdoc::missing_doc_code_examples))]
pub struct Verify<F, G, O2: ?Sized> {
  first: F,
  second: G,
  phantom: core::marker::PhantomData<O2>,
}

impl<F, G, O2: ?Sized> Verify<F, G, O2> {
  pub(crate) fn new(first: F, second: G) -> Self {
    Self {
      first,
      second,
      phantom: Default::default(),
    }
  }
}

impl<I, O1, O2, E, F: Parser<I, O1, E>, G> Parser<I, O1, E> for Verify<F, G, O2>
where
  I: Clone,
  E: ParseError<I>,
  F: Parser<I, O1, E>,
  G: Fn(&O2) -> bool,
  O1: Borrow<O2>,
  O2: ?Sized,
{
  fn parse(&mut self, input: I) -> IResult<I, O1, E> {
    let i = input.clone();
    let (input, o) = (self.first).parse(input)?;

    if (self.second)(o.borrow()) {
      Ok((input, o))
    } else {
      Err(Err::Error(E::from_error_kind(i, ErrorKind::Verify)))
    }
  }
}

/// Returns the provided value if the child parser succeeds.
///
/// ```rust
/// # use nom::{Err,error::ErrorKind, IResult};
/// use nom::combinator::value;
/// use nom::character::alpha1;
/// # fn main() {
///
/// let mut parser = value(1234, alpha1);
///
/// assert_eq!(parser("abcd"), Ok(("", 1234)));
/// assert_eq!(parser("123abcd;"), Err(Err::Error(("123abcd;", ErrorKind::Alpha))));
/// # }
/// ```
pub fn value<I, O1: Clone, O2, E: ParseError<I>, F>(
  val: O1,
  mut parser: F,
) -> impl FnMut(I) -> IResult<I, O1, E>
where
  F: Parser<I, O2, E>,
{
  move |input: I| parser.parse(input).map(|(i, _)| (i, val.clone()))
}

/// Implementation of [`Parser::value`]
#[cfg_attr(nightly, warn(rustdoc::missing_doc_code_examples))]
pub struct Value<F, O1, O2> {
  parser: F,
  val: O2,
  phantom: core::marker::PhantomData<O1>,
}

impl<F, O1, O2> Value<F, O1, O2> {
  pub(crate) fn new(parser: F, val: O2) -> Self {
    Self {
      parser,
      val,
      phantom: Default::default(),
    }
  }
}

impl<I, O1, O2: Clone, E: ParseError<I>, F: Parser<I, O1, E>> Parser<I, O2, E>
  for Value<F, O1, O2>
{
  fn parse(&mut self, input: I) -> IResult<I, O2, E> {
    (self.parser)
      .parse(input)
      .map(|(i, _)| (i, self.val.clone()))
  }
}

/// Succeeds if the child parser returns an error.
///
/// ```rust
/// # use nom::{Err,error::ErrorKind, IResult};
/// use nom::combinator::not;
/// use nom::character::alpha1;
/// # fn main() {
///
/// let mut parser = not(alpha1);
///
/// assert_eq!(parser("123"), Ok(("123", ())));
/// assert_eq!(parser("abcd"), Err(Err::Error(("abcd", ErrorKind::Not))));
/// # }
/// ```
pub fn not<I: Clone, O, E: ParseError<I>, F>(mut parser: F) -> impl FnMut(I) -> IResult<I, (), E>
where
  F: Parser<I, O, E>,
{
  move |input: I| {
    let i = input.clone();
    match parser.parse(input) {
      Ok(_) => Err(Err::Error(E::from_error_kind(i, ErrorKind::Not))),
      Err(Err::Error(_)) => Ok((i, ())),
      Err(e) => Err(e),
    }
  }
}

/// If the child parser was successful, return the consumed input as produced value.
///
/// ```rust
/// # use nom::{Err,error::ErrorKind, IResult};
/// use nom::combinator::recognize;
/// use nom::character::{char, alpha1};
/// use nom::sequence::separated_pair;
/// # fn main() {
///
/// let mut parser = recognize(separated_pair(alpha1, char(','), alpha1));
///
/// assert_eq!(parser("abcd,efgh"), Ok(("", "abcd,efgh")));
/// assert_eq!(parser("abcd;"),Err(Err::Error((";", ErrorKind::Char))));
/// # }
/// ```
pub fn recognize<I, O, E: ParseError<I>, F>(
  mut parser: F,
) -> impl FnMut(I) -> IResult<I, <I as IntoOutput>::Output, E>
where
  I: Clone,
  I: Offset,
  I: Slice<RangeTo<usize>>,
  I: IntoOutput,
  F: Parser<I, O, E>,
{
  move |input: I| {
    let i = input.clone();
    match parser.parse(i) {
      Ok((i, _)) => {
        let index = input.offset(&i);
        Ok((i, input.slice(..index))).into_output()
      }
      Err(e) => Err(e),
    }
  }
}

/// Implementation of [`Parser::recognize`]
#[cfg_attr(nightly, warn(rustdoc::missing_doc_code_examples))]
pub struct Recognize<F, O> {
  parser: F,
  o: core::marker::PhantomData<O>,
}

impl<F, O> Recognize<F, O> {
  pub(crate) fn new(parser: F) -> Self {
    Self {
      parser,
      o: Default::default(),
    }
  }
}

impl<I, O, E, F> Parser<I, <I as IntoOutput>::Output, E> for Recognize<F, O>
where
  I: Clone,
  I: Offset,
  I: Slice<RangeTo<usize>>,
  I: IntoOutput,
  E: ParseError<I>,
  F: Parser<I, O, E>,
{
  fn parse(&mut self, input: I) -> IResult<I, <I as IntoOutput>::Output, E> {
    let i = input.clone();
    match (self.parser).parse(i) {
      Ok((i, _)) => {
        let index = input.offset(&i);
        Ok((i, input.slice(..index))).into_output()
      }
      Err(e) => Err(e),
    }
  }
}

/// if the child parser was successful, return the consumed input with the output
/// as a tuple. Functions similarly to [recognize](fn.recognize.html) except it
/// returns the parser output as well.
///
/// This can be useful especially in cases where the output is not the same type
/// as the input, or the input is a user defined type.
///
/// Returned tuple is of the format `(consumed input, produced output)`.
///
/// ```rust
/// # use nom::prelude::*;
/// # use nom::{Err,error::ErrorKind, IResult};
/// use nom::combinator::{consumed, value, recognize, map};
/// use nom::character::{char, alpha1};
/// use nom::bytes::tag;
/// use nom::sequence::separated_pair;
///
/// fn inner_parser(input: &str) -> IResult<&str, bool> {
///     value(true, tag("1234"))(input)
/// }
///
/// # fn main() {
///
/// let mut consumed_parser = consumed(value(true, separated_pair(alpha1, char(','), alpha1)));
///
/// assert_eq!(consumed_parser("abcd,efgh1"), Ok(("1", ("abcd,efgh", true))));
/// assert_eq!(consumed_parser("abcd;"),Err(Err::Error((";", ErrorKind::Char))));
///
///
/// // the first output (representing the consumed input)
/// // should be the same as that of the `recognize` parser.
/// let mut recognize_parser = recognize(inner_parser);
/// let mut consumed_parser = consumed(inner_parser).map(|(consumed, output)| consumed);
///
/// assert_eq!(recognize_parser("1234"), consumed_parser.parse("1234"));
/// assert_eq!(recognize_parser("abcd"), consumed_parser.parse("abcd"));
/// # }
/// ```
pub fn consumed<I, O, F, E>(
  mut parser: F,
) -> impl FnMut(I) -> IResult<I, (<I as IntoOutput>::Output, O), E>
where
  I: Clone + Offset + Slice<RangeTo<usize>>,
  I: IntoOutput,
  E: ParseError<I>,
  F: Parser<I, O, E>,
{
  move |input: I| {
    let i = input.clone();
    match parser.parse(i) {
      Ok((remaining, result)) => {
        let index = input.offset(&remaining);
        let consumed = input.slice(..index).into_output();
        Ok((remaining, (consumed, result)))
      }
      Err(e) => Err(e),
    }
  }
}

/// Implementation of [`Parser::with_recognized`]
#[cfg_attr(nightly, warn(rustdoc::missing_doc_code_examples))]
pub struct WithRecognized<F, O> {
  parser: F,
  o: core::marker::PhantomData<O>,
}

impl<F, O> WithRecognized<F, O> {
  pub(crate) fn new(parser: F) -> Self {
    Self {
      parser,
      o: Default::default(),
    }
  }
}

impl<I, O, E, F> Parser<I, (O, <I as IntoOutput>::Output), E> for WithRecognized<F, O>
where
  I: Clone,
  I: Offset,
  I: Slice<RangeTo<usize>>,
  I: IntoOutput,
  E: ParseError<I>,
  F: Parser<I, O, E>,
{
  fn parse(&mut self, input: I) -> IResult<I, (O, <I as IntoOutput>::Output), E> {
    let i = input.clone();
    match (self.parser).parse(i) {
      Ok((remaining, result)) => {
        let index = input.offset(&remaining);
        let consumed = input.slice(..index).into_output();
        Ok((remaining, (result, consumed)))
      }
      Err(e) => Err(e),
    }
  }
}

/// Transforms an [`Err::Error`] (recoverable) to [`Err::Failure`] (unrecoverable)
///
/// This commits the parse result, preventing alternative branch paths like with
/// [`nom::branch::alt`][crate::branch::alt].
///
/// # Example
///
/// Without `cut`:
/// ```rust
/// # use nom::{Err,error::ErrorKind, IResult};
/// # use nom::character::one_of;
/// # use nom::character::digit1;
/// # use nom::combinator::rest;
/// # use nom::branch::alt;
/// # use nom::sequence::preceded;
/// # fn main() {
///
/// fn parser(input: &str) -> IResult<&str, &str> {
///   alt((
///     preceded(one_of("+-"), digit1),
///     rest
///   ))(input)
/// }
///
/// assert_eq!(parser("+10 ab"), Ok((" ab", "10")));
/// assert_eq!(parser("ab"), Ok(("", "ab")));
/// assert_eq!(parser("+"), Ok(("", "+")));
/// # }
/// ```
///
/// With `cut`:
/// ```rust
/// # use nom::{Err,error::ErrorKind, IResult, error::Error};
/// # use nom::character::one_of;
/// # use nom::character::digit1;
/// # use nom::combinator::rest;
/// # use nom::branch::alt;
/// # use nom::sequence::preceded;
/// use nom::combinator::cut;
/// # fn main() {
///
/// fn parser(input: &str) -> IResult<&str, &str> {
///   alt((
///     preceded(one_of("+-"), cut(digit1)),
///     rest
///   ))(input)
/// }
///
/// assert_eq!(parser("+10 ab"), Ok((" ab", "10")));
/// assert_eq!(parser("ab"), Ok(("", "ab")));
/// assert_eq!(parser("+"), Err(Err::Failure(Error { input: "", code: ErrorKind::Digit })));
/// # }
/// ```
pub fn cut<I, O, E: ParseError<I>, F>(mut parser: F) -> impl FnMut(I) -> IResult<I, O, E>
where
  F: Parser<I, O, E>,
{
  move |input: I| match parser.parse(input) {
    Err(Err::Error(e)) => Err(Err::Failure(e)),
    rest => rest,
  }
}

/// automatically converts the child parser's result to another type
///
/// it will be able to convert the output value and the error value
/// as long as the `Into` implementations are available
///
/// **WARNING:** Deprecated, replaced with [`Parser::into`]
///
/// ```rust
/// # use nom::IResult;
/// use nom::combinator::into;
/// use nom::character::alpha1;
/// # fn main() {
///
///  fn parser1(i: &str) -> IResult<&str, &str> {
///    alpha1(i)
///  }
///
///  let mut parser2 = into(parser1);
///
/// // the parser converts the &str output of the child parser into a Vec<u8>
/// let bytes: IResult<&str, Vec<u8>> = parser2("abcd");
/// assert_eq!(bytes, Ok(("", vec![97, 98, 99, 100])));
/// # }
/// ```
#[deprecated(since = "8.0.0", note = "Replaced with `Parser::into")]
pub fn into<I, O1, O2, E1, E2, F>(mut parser: F) -> impl FnMut(I) -> IResult<I, O2, E2>
where
  O1: convert::Into<O2>,
  E1: convert::Into<E2>,
  E1: ParseError<I>,
  E2: ParseError<I>,
  F: Parser<I, O1, E1>,
{
  //map(parser, Into::into)
  move |input: I| match parser.parse(input) {
    Ok((i, o)) => Ok((i, o.into())),
    Err(Err::Error(e)) => Err(Err::Error(e.into())),
    Err(Err::Failure(e)) => Err(Err::Failure(e.into())),
    Err(Err::Incomplete(e)) => Err(Err::Incomplete(e)),
  }
}

/// Implementation of [`Parser::into`]
#[cfg_attr(nightly, warn(rustdoc::missing_doc_code_examples))]
pub struct Into<F, O1, O2: From<O1>, E1, E2: From<E1>> {
  f: F,
  phantom_out1: core::marker::PhantomData<O1>,
  phantom_err1: core::marker::PhantomData<E1>,
  phantom_out2: core::marker::PhantomData<O2>,
  phantom_err2: core::marker::PhantomData<E2>,
}

impl<F, O1, O2: From<O1>, E1, E2: From<E1>> Into<F, O1, O2, E1, E2> {
  pub(crate) fn new(f: F) -> Self {
    Self {
      f,
      phantom_out1: Default::default(),
      phantom_err1: Default::default(),
      phantom_out2: Default::default(),
      phantom_err2: Default::default(),
    }
  }
}

impl<
    'a,
    I: Clone,
    O1,
    O2: From<O1>,
    E1,
    E2: crate::error::ParseError<I> + From<E1>,
    F: Parser<I, O1, E1>,
  > Parser<I, O2, E2> for Into<F, O1, O2, E1, E2>
{
  fn parse(&mut self, i: I) -> IResult<I, O2, E2> {
    match self.f.parse(i) {
      Ok((i, o)) => Ok((i, o.into())),
      Err(Err::Error(e)) => Err(Err::Error(e.into())),
      Err(Err::Failure(e)) => Err(Err::Failure(e.into())),
      Err(Err::Incomplete(e)) => Err(Err::Incomplete(e)),
    }
  }
}
/// Creates an iterator from input data and a parser.
///
/// Call the iterator's [ParserIterator::finish] method to get the remaining input if successful,
/// or the error value if we encountered an error.
///
/// On [`Err::Error`], iteration will stop.  To instead chain an error up, see [`cut`].
///
/// ```rust
/// use nom::{combinator::iterator, IResult, bytes::tag, character::alpha1, sequence::terminated};
/// use std::collections::HashMap;
///
/// let data = "abc|defg|hijkl|mnopqr|123";
/// let mut it = iterator(data, terminated(alpha1, tag("|")));
///
/// let parsed = it.map(|v| (v, v.len())).collect::<HashMap<_,_>>();
/// let res: IResult<_,_> = it.finish();
///
/// assert_eq!(parsed, [("abc", 3usize), ("defg", 4), ("hijkl", 5), ("mnopqr", 6)].iter().cloned().collect());
/// assert_eq!(res, Ok(("123", ())));
/// ```
pub fn iterator<Input, Output, Error, F>(
  input: Input,
  f: F,
) -> ParserIterator<Input, Output, Error, F>
where
  F: Parser<Input, Output, Error>,
  Error: ParseError<Input>,
{
  ParserIterator {
    iterator: f,
    input,
    output: Default::default(),
    state: Some(State::Running),
  }
}

/// Main structure associated to the [iterator] function.
pub struct ParserIterator<I, O, E, F> {
  iterator: F,
  input: I,
  output: core::marker::PhantomData<O>,
  state: Option<State<E>>,
}

impl<I: Clone, O, E, F> ParserIterator<I, O, E, F> {
  /// Returns the remaining input if parsing was successful, or the error if we encountered an error.
  pub fn finish(mut self) -> IResult<I, (), E> {
    match self.state.take().unwrap() {
      State::Running | State::Done => Ok((self.input, ())),
      State::Failure(e) => Err(Err::Failure(e)),
      State::Incomplete(i) => Err(Err::Incomplete(i)),
    }
  }
}

impl<'a, Input, Output, Error, F> core::iter::Iterator
  for &'a mut ParserIterator<Input, Output, Error, F>
where
  F: Parser<Input, Output, Error>,
  Input: Clone,
{
  type Item = Output;

  fn next(&mut self) -> Option<Self::Item> {
    if let State::Running = self.state.take().unwrap() {
      let input = self.input.clone();

      match self.iterator.parse(input) {
        Ok((i, o)) => {
          self.input = i;
          self.state = Some(State::Running);
          Some(o)
        }
        Err(Err::Error(_)) => {
          self.state = Some(State::Done);
          None
        }
        Err(Err::Failure(e)) => {
          self.state = Some(State::Failure(e));
          None
        }
        Err(Err::Incomplete(i)) => {
          self.state = Some(State::Incomplete(i));
          None
        }
      }
    } else {
      None
    }
  }
}

enum State<E> {
  Running,
  Done,
  Failure(E),
  Incomplete(Needed),
}

/// a parser which always succeeds with given value without consuming any input.
///
/// It can be used for example as the last alternative in `alt` to
/// specify the default case.
///
/// ```rust
/// # use nom::{Err,error::ErrorKind, IResult};
/// use nom::branch::alt;
/// use nom::combinator::{success, value};
/// use nom::character::char;
/// # fn main() {
///
/// let mut parser = success::<_,_,(_,ErrorKind)>(10);
/// assert_eq!(parser("xyz"), Ok(("xyz", 10)));
///
/// let mut sign = alt((value(-1, char('-')), value(1, char('+')), success::<_,_,(_,ErrorKind)>(1)));
/// assert_eq!(sign("+10"), Ok(("10", 1)));
/// assert_eq!(sign("-10"), Ok(("10", -1)));
/// assert_eq!(sign("10"), Ok(("10", 1)));
/// # }
/// ```
pub fn success<I, O: Clone, E: ParseError<I>>(val: O) -> impl Fn(I) -> IResult<I, O, E> {
  move |input: I| Ok((input, val.clone()))
}

/// A parser which always fails.
///
/// ```rust
/// # use nom::{Err, error::ErrorKind, IResult};
/// use nom::combinator::fail;
///
/// let s = "string";
/// assert_eq!(fail::<_, &str, _>(s), Err(Err::Error((s, ErrorKind::Fail))));
/// ```
pub fn fail<I, O, E: ParseError<I>>(i: I) -> IResult<I, O, E> {
  Err(Err::Error(E::from_error_kind(i, ErrorKind::Fail)))
}
