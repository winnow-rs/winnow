//! Character specific parsers and combinators
//!
//! Functions recognizing specific characters

#![allow(deprecated)] // will just become `pub(crate)` later

pub mod complete;
pub mod streaming;
#[cfg(test)]
mod tests;

use crate::error::ParseError;
use crate::input::Compare;
use crate::input::{AsBytes, AsChar, Input, InputIsStreaming, Offset, ParseTo, SliceLen};
use crate::IResult;

/// Recognizes the string "\r\n".
///
/// *Complete version*: Will return an error if there's not enough input data.
///
/// *Streaming version*: Will return `Err(winnow::Err::Incomplete(_))` if there's not enough input data.
///
/// # Example
///
/// ```
/// # use winnow::{Err, error::{Error, ErrorKind}, IResult};
/// # use winnow::character::crlf;
/// fn parser(input: &str) -> IResult<&str, &str> {
///     crlf(input)
/// }
///
/// assert_eq!(parser("\r\nc"), Ok(("c", "\r\n")));
/// assert_eq!(parser("ab\r\nc"), Err(Err::Error(Error::new("ab\r\nc", ErrorKind::CrLf))));
/// assert_eq!(parser(""), Err(Err::Error(Error::new("", ErrorKind::CrLf))));
/// ```
///
/// ```
/// # use winnow::{Err, error::ErrorKind, error::Error, IResult, Needed};
/// # use winnow::input::Streaming;
/// # use winnow::character::crlf;
/// assert_eq!(crlf::<_, Error<_>, true>(Streaming("\r\nc")), Ok((Streaming("c"), "\r\n")));
/// assert_eq!(crlf::<_, Error<_>, true>(Streaming("ab\r\nc")), Err(Err::Error(Error::new(Streaming("ab\r\nc"), ErrorKind::CrLf))));
/// assert_eq!(crlf::<_, Error<_>, true>(Streaming("")), Err(Err::Incomplete(Needed::new(2))));
/// ```
#[inline(always)]
pub fn crlf<I, E: ParseError<I>, const STREAMING: bool>(
  input: I,
) -> IResult<I, <I as Input>::Slice, E>
where
  I: InputIsStreaming<STREAMING>,
  I: Input,
  I: Compare<&'static str>,
{
  if STREAMING {
    streaming::crlf(input)
  } else {
    complete::crlf(input)
  }
}

/// Recognizes a string of any char except '\r\n' or '\n'.
///
/// *Complete version*: Will return an error if there's not enough input data.
///
/// *Streaming version*: Will return `Err(winnow::Err::Incomplete(_))` if there's not enough input data.
///
/// # Example
///
/// ```
/// # use winnow::{Err, error::{Error, ErrorKind}, IResult, Needed};
/// # use winnow::character::not_line_ending;
/// fn parser(input: &str) -> IResult<&str, &str> {
///     not_line_ending(input)
/// }
///
/// assert_eq!(parser("ab\r\nc"), Ok(("\r\nc", "ab")));
/// assert_eq!(parser("ab\nc"), Ok(("\nc", "ab")));
/// assert_eq!(parser("abc"), Ok(("", "abc")));
/// assert_eq!(parser(""), Ok(("", "")));
/// assert_eq!(parser("a\rb\nc"), Err(Err::Error(Error { input: "a\rb\nc", kind: ErrorKind::Tag })));
/// assert_eq!(parser("a\rbc"), Err(Err::Error(Error { input: "a\rbc", kind: ErrorKind::Tag })));
/// ```
///
/// ```
/// # use winnow::{Err, error::{Error, ErrorKind}, IResult, Needed};
/// # use winnow::input::Streaming;
/// # use winnow::character::not_line_ending;
/// assert_eq!(not_line_ending::<_, Error<_>, true>(Streaming("ab\r\nc")), Ok((Streaming("\r\nc"), "ab")));
/// assert_eq!(not_line_ending::<_, Error<_>, true>(Streaming("abc")), Err(Err::Incomplete(Needed::Unknown)));
/// assert_eq!(not_line_ending::<_, Error<_>, true>(Streaming("")), Err(Err::Incomplete(Needed::Unknown)));
/// assert_eq!(not_line_ending::<_, Error<_>, true>(Streaming("a\rb\nc")), Err(Err::Error(Error::new(Streaming("a\rb\nc"), ErrorKind::Tag ))));
/// assert_eq!(not_line_ending::<_, Error<_>, true>(Streaming("a\rbc")), Err(Err::Error(Error::new(Streaming("a\rbc"), ErrorKind::Tag ))));
/// ```
#[inline(always)]
pub fn not_line_ending<I, E: ParseError<I>, const STREAMING: bool>(
  input: I,
) -> IResult<I, <I as Input>::Slice, E>
where
  I: InputIsStreaming<STREAMING>,
  I: Input + AsBytes,
  I: Compare<&'static str>,
  <I as Input>::Token: AsChar,
{
  if STREAMING {
    streaming::not_line_ending(input)
  } else {
    complete::not_line_ending(input)
  }
}

/// Recognizes an end of line (both '\n' and '\r\n').
///
/// *Complete version*: Will return an error if there's not enough input data.
///
/// *Streaming version*: Will return `Err(winnow::Err::Incomplete(_))` if there's not enough input data.
///
/// # Example
///
/// ```
/// # use winnow::{Err, error::{Error, ErrorKind}, IResult, Needed};
/// # use winnow::character::line_ending;
/// fn parser(input: &str) -> IResult<&str, &str> {
///     line_ending(input)
/// }
///
/// assert_eq!(parser("\r\nc"), Ok(("c", "\r\n")));
/// assert_eq!(parser("ab\r\nc"), Err(Err::Error(Error::new("ab\r\nc", ErrorKind::CrLf))));
/// assert_eq!(parser(""), Err(Err::Error(Error::new("", ErrorKind::CrLf))));
/// ```
///
/// ```
/// # use winnow::{Err, error::ErrorKind, error::Error, IResult, Needed};
/// # use winnow::input::Streaming;
/// # use winnow::character::line_ending;
/// assert_eq!(line_ending::<_, Error<_>, true>(Streaming("\r\nc")), Ok((Streaming("c"), "\r\n")));
/// assert_eq!(line_ending::<_, Error<_>, true>(Streaming("ab\r\nc")), Err(Err::Error(Error::new(Streaming("ab\r\nc"), ErrorKind::CrLf))));
/// assert_eq!(line_ending::<_, Error<_>, true>(Streaming("")), Err(Err::Incomplete(Needed::new(1))));
/// ```
#[inline(always)]
pub fn line_ending<I, E: ParseError<I>, const STREAMING: bool>(
  input: I,
) -> IResult<I, <I as Input>::Slice, E>
where
  I: InputIsStreaming<STREAMING>,
  I: Input,
  I: Compare<&'static str>,
{
  if STREAMING {
    streaming::line_ending(input)
  } else {
    complete::line_ending(input)
  }
}

/// Matches a newline character '\n'.
///
/// *Complete version*: Will return an error if there's not enough input data.
///
/// *Streaming version*: Will return `Err(winnow::Err::Incomplete(_))` if there's not enough input data.
///
/// # Example
///
/// ```
/// # use winnow::{Err, error::{Error, ErrorKind}, IResult, Needed};
/// # use winnow::character::newline;
/// fn parser(input: &str) -> IResult<&str, char> {
///     newline(input)
/// }
///
/// assert_eq!(parser("\nc"), Ok(("c", '\n')));
/// assert_eq!(parser("\r\nc"), Err(Err::Error(Error::new("\r\nc", ErrorKind::Char))));
/// assert_eq!(parser(""), Err(Err::Error(Error::new("", ErrorKind::Char))));
/// ```
///
/// ```
/// # use winnow::{Err, error::ErrorKind, error::Error, IResult, Needed};
/// # use winnow::input::Streaming;
/// # use winnow::character::newline;
/// assert_eq!(newline::<_, Error<_>, true>(Streaming("\nc")), Ok((Streaming("c"), '\n')));
/// assert_eq!(newline::<_, Error<_>, true>(Streaming("\r\nc")), Err(Err::Error(Error::new(Streaming("\r\nc"), ErrorKind::Char))));
/// assert_eq!(newline::<_, Error<_>, true>(Streaming("")), Err(Err::Incomplete(Needed::new(1))));
/// ```
#[inline(always)]
pub fn newline<I, Error: ParseError<I>, const STREAMING: bool>(input: I) -> IResult<I, char, Error>
where
  I: InputIsStreaming<STREAMING>,
  I: Input,
  <I as Input>::Token: AsChar,
{
  if STREAMING {
    streaming::newline(input)
  } else {
    complete::newline(input)
  }
}

/// Matches a tab character '\t'.
///
/// *Complete version*: Will return an error if there's not enough input data.
///
/// *Streaming version*: Will return `Err(winnow::Err::Incomplete(_))` if there's not enough input data.
///
/// # Example
///
/// ```
/// # use winnow::{Err, error::{Error, ErrorKind}, IResult, Needed};
/// # use winnow::character::tab;
/// fn parser(input: &str) -> IResult<&str, char> {
///     tab(input)
/// }
///
/// assert_eq!(parser("\tc"), Ok(("c", '\t')));
/// assert_eq!(parser("\r\nc"), Err(Err::Error(Error::new("\r\nc", ErrorKind::Char))));
/// assert_eq!(parser(""), Err(Err::Error(Error::new("", ErrorKind::Char))));
/// ```
///
/// ```
/// # use winnow::{Err, error::ErrorKind, error::Error, IResult, Needed};
/// # use winnow::input::Streaming;
/// # use winnow::character::tab;
/// assert_eq!(tab::<_, Error<_>, true>(Streaming("\tc")), Ok((Streaming("c"), '\t')));
/// assert_eq!(tab::<_, Error<_>, true>(Streaming("\r\nc")), Err(Err::Error(Error::new(Streaming("\r\nc"), ErrorKind::Char))));
/// assert_eq!(tab::<_, Error<_>, true>(Streaming("")), Err(Err::Incomplete(Needed::new(1))));
/// ```
#[inline(always)]
pub fn tab<I, Error: ParseError<I>, const STREAMING: bool>(input: I) -> IResult<I, char, Error>
where
  I: InputIsStreaming<STREAMING>,
  I: Input,
  <I as Input>::Token: AsChar,
{
  if STREAMING {
    streaming::tab(input)
  } else {
    complete::tab(input)
  }
}

/// Recognizes zero or more lowercase and uppercase ASCII alphabetic characters: a-z, A-Z
///
/// *Complete version*: Will return the whole input if no terminating token is found (a non
/// alphabetic character).
///
/// *Streaming version*: Will return `Err(winnow::Err::Incomplete(_))` if there's not enough input data,
/// or if no terminating token is found (a non alphabetic character).
///
/// # Example
///
/// ```
/// # use winnow::{Err, error::ErrorKind, error::Error, IResult, Needed};
/// # use winnow::character::alpha0;
/// fn parser(input: &str) -> IResult<&str, &str> {
///     alpha0(input)
/// }
///
/// assert_eq!(parser("ab1c"), Ok(("1c", "ab")));
/// assert_eq!(parser("1c"), Ok(("1c", "")));
/// assert_eq!(parser(""), Ok(("", "")));
/// ```
///
/// ```
/// # use winnow::{Err, error::ErrorKind, error::Error, IResult, Needed};
/// # use winnow::input::Streaming;
/// # use winnow::character::alpha0;
/// assert_eq!(alpha0::<_, Error<_>, true>(Streaming("ab1c")), Ok((Streaming("1c"), "ab")));
/// assert_eq!(alpha0::<_, Error<_>, true>(Streaming("1c")), Ok((Streaming("1c"), "")));
/// assert_eq!(alpha0::<_, Error<_>, true>(Streaming("")), Err(Err::Incomplete(Needed::new(1))));
/// ```
#[inline(always)]
pub fn alpha0<I, E: ParseError<I>, const STREAMING: bool>(
  input: I,
) -> IResult<I, <I as Input>::Slice, E>
where
  I: InputIsStreaming<STREAMING>,
  I: Input,
  <I as Input>::Token: AsChar,
{
  if STREAMING {
    streaming::alpha0(input)
  } else {
    complete::alpha0(input)
  }
}

/// Recognizes one or more lowercase and uppercase ASCII alphabetic characters: a-z, A-Z
///
/// *Complete version*: Will return an error if there's not enough input data,
/// or the whole input if no terminating token is found  (a non alphabetic character).
///
/// *Streaming version*: Will return `Err(winnow::Err::Incomplete(_))` if there's not enough input data,
/// or if no terminating token is found (a non alphabetic character).
///
/// # Example
///
/// ```
/// # use winnow::{Err, error::{Error, ErrorKind}, IResult, Needed};
/// # use winnow::character::alpha1;
/// fn parser(input: &str) -> IResult<&str, &str> {
///     alpha1(input)
/// }
///
/// assert_eq!(parser("aB1c"), Ok(("1c", "aB")));
/// assert_eq!(parser("1c"), Err(Err::Error(Error::new("1c", ErrorKind::Alpha))));
/// assert_eq!(parser(""), Err(Err::Error(Error::new("", ErrorKind::Alpha))));
/// ```
///
/// ```
/// # use winnow::{Err, error::ErrorKind, error::Error, IResult, Needed};
/// # use winnow::input::Streaming;
/// # use winnow::character::alpha1;
/// assert_eq!(alpha1::<_, Error<_>, true>(Streaming("aB1c")), Ok((Streaming("1c"), "aB")));
/// assert_eq!(alpha1::<_, Error<_>, true>(Streaming("1c")), Err(Err::Error(Error::new(Streaming("1c"), ErrorKind::Alpha))));
/// assert_eq!(alpha1::<_, Error<_>, true>(Streaming("")), Err(Err::Incomplete(Needed::new(1))));
/// ```
#[inline(always)]
pub fn alpha1<I, E: ParseError<I>, const STREAMING: bool>(
  input: I,
) -> IResult<I, <I as Input>::Slice, E>
where
  I: InputIsStreaming<STREAMING>,
  I: Input,
  <I as Input>::Token: AsChar,
{
  if STREAMING {
    streaming::alpha1(input)
  } else {
    complete::alpha1(input)
  }
}

/// Recognizes zero or more ASCII numerical characters: 0-9
///
/// *Complete version*: Will return an error if there's not enough input data,
/// or the whole input if no terminating token is found (a non digit character).
///
/// *Streaming version*: Will return `Err(winnow::Err::Incomplete(_))` if there's not enough input data,
/// or if no terminating token is found (a non digit character).
///
/// # Example
///
/// ```
/// # use winnow::{Err, error::ErrorKind, error::Error, IResult, Needed};
/// # use winnow::character::digit0;
/// fn parser(input: &str) -> IResult<&str, &str> {
///     digit0(input)
/// }
///
/// assert_eq!(parser("21c"), Ok(("c", "21")));
/// assert_eq!(parser("21"), Ok(("", "21")));
/// assert_eq!(parser("a21c"), Ok(("a21c", "")));
/// assert_eq!(parser(""), Ok(("", "")));
/// ```
///
/// ```
/// # use winnow::{Err, error::ErrorKind, error::Error, IResult, Needed};
/// # use winnow::input::Streaming;
/// # use winnow::character::digit0;
/// assert_eq!(digit0::<_, Error<_>, true>(Streaming("21c")), Ok((Streaming("c"), "21")));
/// assert_eq!(digit0::<_, Error<_>, true>(Streaming("a21c")), Ok((Streaming("a21c"), "")));
/// assert_eq!(digit0::<_, Error<_>, true>(Streaming("")), Err(Err::Incomplete(Needed::new(1))));
/// ```
#[inline(always)]
pub fn digit0<I, E: ParseError<I>, const STREAMING: bool>(
  input: I,
) -> IResult<I, <I as Input>::Slice, E>
where
  I: InputIsStreaming<STREAMING>,
  I: Input,
  <I as Input>::Token: AsChar,
{
  if STREAMING {
    streaming::digit0(input)
  } else {
    complete::digit0(input)
  }
}

/// Recognizes one or more ASCII numerical characters: 0-9
///
/// *Complete version*: Will return an error if there's not enough input data,
/// or the whole input if no terminating token is found (a non digit character).
///
/// *Streaming version*: Will return `Err(winnow::Err::Incomplete(_))` if there's not enough input data,
/// or if no terminating token is found (a non digit character).
///
/// # Example
///
/// ```
/// # use winnow::{Err, error::{Error, ErrorKind}, IResult, Needed};
/// # use winnow::character::digit1;
/// fn parser(input: &str) -> IResult<&str, &str> {
///     digit1(input)
/// }
///
/// assert_eq!(parser("21c"), Ok(("c", "21")));
/// assert_eq!(parser("c1"), Err(Err::Error(Error::new("c1", ErrorKind::Digit))));
/// assert_eq!(parser(""), Err(Err::Error(Error::new("", ErrorKind::Digit))));
/// ```
///
/// ```
/// # use winnow::{Err, error::ErrorKind, error::Error, IResult, Needed};
/// # use winnow::input::Streaming;
/// # use winnow::character::digit1;
/// assert_eq!(digit1::<_, Error<_>, true>(Streaming("21c")), Ok((Streaming("c"), "21")));
/// assert_eq!(digit1::<_, Error<_>, true>(Streaming("c1")), Err(Err::Error(Error::new(Streaming("c1"), ErrorKind::Digit))));
/// assert_eq!(digit1::<_, Error<_>, true>(Streaming("")), Err(Err::Incomplete(Needed::new(1))));
/// ```
///
/// ## Parsing an integer
///
/// You can use `digit1` in combination with [`Parser::map_res`][crate::Parser::map_res] to parse an integer:
///
/// ```
/// # use winnow::{Err, error::{Error, ErrorKind}, IResult, Needed, Parser};
/// # use winnow::character::digit1;
/// fn parser(input: &str) -> IResult<&str, u32> {
///   digit1.map_res(str::parse).parse_next(input)
/// }
///
/// assert_eq!(parser("416"), Ok(("", 416)));
/// assert_eq!(parser("12b"), Ok(("b", 12)));
/// assert!(parser("b").is_err());
/// ```
#[inline(always)]
pub fn digit1<I, E: ParseError<I>, const STREAMING: bool>(
  input: I,
) -> IResult<I, <I as Input>::Slice, E>
where
  I: InputIsStreaming<STREAMING>,
  I: Input,
  <I as Input>::Token: AsChar,
{
  if STREAMING {
    streaming::digit1(input)
  } else {
    complete::digit1(input)
  }
}

/// Recognizes zero or more ASCII hexadecimal numerical characters: 0-9, A-F, a-f
///
/// *Complete version*: Will return the whole input if no terminating token is found (a non hexadecimal digit character).
///
/// *Streaming version*: Will return `Err(winnow::Err::Incomplete(_))` if there's not enough input data,
/// or if no terminating token is found (a non hexadecimal digit character).
///
/// # Example
///
/// ```
/// # use winnow::{Err, error::ErrorKind, error::Error, IResult, Needed};
/// # use winnow::character::hex_digit0;
/// fn parser(input: &str) -> IResult<&str, &str> {
///     hex_digit0(input)
/// }
///
/// assert_eq!(parser("21cZ"), Ok(("Z", "21c")));
/// assert_eq!(parser("Z21c"), Ok(("Z21c", "")));
/// assert_eq!(parser(""), Ok(("", "")));
/// ```
///
/// ```
/// # use winnow::{Err, error::ErrorKind, error::Error, IResult, Needed};
/// # use winnow::input::Streaming;
/// # use winnow::character::hex_digit0;
/// assert_eq!(hex_digit0::<_, Error<_>, true>(Streaming("21cZ")), Ok((Streaming("Z"), "21c")));
/// assert_eq!(hex_digit0::<_, Error<_>, true>(Streaming("Z21c")), Ok((Streaming("Z21c"), "")));
/// assert_eq!(hex_digit0::<_, Error<_>, true>(Streaming("")), Err(Err::Incomplete(Needed::new(1))));
/// ```
#[inline(always)]
pub fn hex_digit0<I, E: ParseError<I>, const STREAMING: bool>(
  input: I,
) -> IResult<I, <I as Input>::Slice, E>
where
  I: InputIsStreaming<STREAMING>,
  I: Input,
  <I as Input>::Token: AsChar,
{
  if STREAMING {
    streaming::hex_digit0(input)
  } else {
    complete::hex_digit0(input)
  }
}

/// Recognizes one or more ASCII hexadecimal numerical characters: 0-9, A-F, a-f
///
/// *Complete version*: Will return an error if there's not enough input data,
/// or the whole input if no terminating token is found (a non hexadecimal digit character).
///
/// *Streaming version*: Will return `Err(winnow::Err::Incomplete(_))` if there's not enough input data,
/// or if no terminating token is found (a non hexadecimal digit character).
///
/// # Example
///
/// ```
/// # use winnow::{Err, error::{Error, ErrorKind}, IResult, Needed};
/// # use winnow::character::hex_digit1;
/// fn parser(input: &str) -> IResult<&str, &str> {
///     hex_digit1(input)
/// }
///
/// assert_eq!(parser("21cZ"), Ok(("Z", "21c")));
/// assert_eq!(parser("H2"), Err(Err::Error(Error::new("H2", ErrorKind::HexDigit))));
/// assert_eq!(parser(""), Err(Err::Error(Error::new("", ErrorKind::HexDigit))));
/// ```
///
/// ```
/// # use winnow::{Err, error::ErrorKind, error::Error, IResult, Needed};
/// # use winnow::input::Streaming;
/// # use winnow::character::hex_digit1;
/// assert_eq!(hex_digit1::<_, Error<_>, true>(Streaming("21cZ")), Ok((Streaming("Z"), "21c")));
/// assert_eq!(hex_digit1::<_, Error<_>, true>(Streaming("H2")), Err(Err::Error(Error::new(Streaming("H2"), ErrorKind::HexDigit))));
/// assert_eq!(hex_digit1::<_, Error<_>, true>(Streaming("")), Err(Err::Incomplete(Needed::new(1))));
/// ```
#[inline(always)]
pub fn hex_digit1<I, E: ParseError<I>, const STREAMING: bool>(
  input: I,
) -> IResult<I, <I as Input>::Slice, E>
where
  I: InputIsStreaming<STREAMING>,
  I: Input,
  <I as Input>::Token: AsChar,
{
  if STREAMING {
    streaming::hex_digit1(input)
  } else {
    complete::hex_digit1(input)
  }
}

/// Recognizes zero or more octal characters: 0-7
///
/// *Complete version*: Will return the whole input if no terminating token is found (a non octal
/// digit character).
///
/// *Streaming version*: Will return `Err(winnow::Err::Incomplete(_))` if there's not enough input data,
/// or if no terminating token is found (a non octal digit character).
///
/// # Example
///
/// ```
/// # use winnow::{Err, error::ErrorKind, error::Error, IResult, Needed};
/// # use winnow::character::oct_digit0;
/// fn parser(input: &str) -> IResult<&str, &str> {
///     oct_digit0(input)
/// }
///
/// assert_eq!(parser("21cZ"), Ok(("cZ", "21")));
/// assert_eq!(parser("Z21c"), Ok(("Z21c", "")));
/// assert_eq!(parser(""), Ok(("", "")));
/// ```
///
/// ```
/// # use winnow::{Err, error::ErrorKind, error::Error, IResult, Needed};
/// # use winnow::input::Streaming;
/// # use winnow::character::oct_digit0;
/// assert_eq!(oct_digit0::<_, Error<_>, true>(Streaming("21cZ")), Ok((Streaming("cZ"), "21")));
/// assert_eq!(oct_digit0::<_, Error<_>, true>(Streaming("Z21c")), Ok((Streaming("Z21c"), "")));
/// assert_eq!(oct_digit0::<_, Error<_>, true>(Streaming("")), Err(Err::Incomplete(Needed::new(1))));
/// ```
#[inline(always)]
pub fn oct_digit0<I, E: ParseError<I>, const STREAMING: bool>(
  input: I,
) -> IResult<I, <I as Input>::Slice, E>
where
  I: InputIsStreaming<STREAMING>,
  I: Input,
  <I as Input>::Token: AsChar,
{
  if STREAMING {
    streaming::oct_digit0(input)
  } else {
    complete::oct_digit0(input)
  }
}

/// Recognizes one or more octal characters: 0-7
///
/// *Complete version*: Will return an error if there's not enough input data,
/// or the whole input if no terminating token is found (a non octal digit character).
///
/// *Streaming version*: Will return `Err(winnow::Err::Incomplete(_))` if there's not enough input data,
/// or if no terminating token is found (a non octal digit character).
///
/// # Example
///
/// ```
/// # use winnow::{Err, error::{Error, ErrorKind}, IResult, Needed};
/// # use winnow::character::oct_digit1;
/// fn parser(input: &str) -> IResult<&str, &str> {
///     oct_digit1(input)
/// }
///
/// assert_eq!(parser("21cZ"), Ok(("cZ", "21")));
/// assert_eq!(parser("H2"), Err(Err::Error(Error::new("H2", ErrorKind::OctDigit))));
/// assert_eq!(parser(""), Err(Err::Error(Error::new("", ErrorKind::OctDigit))));
/// ```
///
/// ```
/// # use winnow::{Err, error::ErrorKind, error::Error, IResult, Needed};
/// # use winnow::input::Streaming;
/// # use winnow::character::oct_digit1;
/// assert_eq!(oct_digit1::<_, Error<_>, true>(Streaming("21cZ")), Ok((Streaming("cZ"), "21")));
/// assert_eq!(oct_digit1::<_, Error<_>, true>(Streaming("H2")), Err(Err::Error(Error::new(Streaming("H2"), ErrorKind::OctDigit))));
/// assert_eq!(oct_digit1::<_, Error<_>, true>(Streaming("")), Err(Err::Incomplete(Needed::new(1))));
/// ```
#[inline(always)]
pub fn oct_digit1<I, E: ParseError<I>, const STREAMING: bool>(
  input: I,
) -> IResult<I, <I as Input>::Slice, E>
where
  I: InputIsStreaming<STREAMING>,
  I: Input,
  <I as Input>::Token: AsChar,
{
  if STREAMING {
    streaming::oct_digit1(input)
  } else {
    complete::oct_digit1(input)
  }
}

/// Recognizes zero or more ASCII numerical and alphabetic characters: 0-9, a-z, A-Z
///
/// *Complete version*: Will return the whole input if no terminating token is found (a non
/// alphanumerical character).
///
/// *Streaming version*: Will return `Err(winnow::Err::Incomplete(_))` if there's not enough input data,
/// or if no terminating token is found (a non alphanumerical character).
///
/// # Example
///
/// ```
/// # use winnow::{Err, error::ErrorKind, error::Error, IResult, Needed};
/// # use winnow::character::alphanumeric0;
/// fn parser(input: &str) -> IResult<&str, &str> {
///     alphanumeric0(input)
/// }
///
/// assert_eq!(parser("21cZ%1"), Ok(("%1", "21cZ")));
/// assert_eq!(parser("&Z21c"), Ok(("&Z21c", "")));
/// assert_eq!(parser(""), Ok(("", "")));
/// ```
///
/// ```
/// # use winnow::{Err, error::ErrorKind, error::Error, IResult, Needed};
/// # use winnow::input::Streaming;
/// # use winnow::character::alphanumeric0;
/// assert_eq!(alphanumeric0::<_, Error<_>, true>(Streaming("21cZ%1")), Ok((Streaming("%1"), "21cZ")));
/// assert_eq!(alphanumeric0::<_, Error<_>, true>(Streaming("&Z21c")), Ok((Streaming("&Z21c"), "")));
/// assert_eq!(alphanumeric0::<_, Error<_>, true>(Streaming("")), Err(Err::Incomplete(Needed::new(1))));
/// ```
#[inline(always)]
pub fn alphanumeric0<I, E: ParseError<I>, const STREAMING: bool>(
  input: I,
) -> IResult<I, <I as Input>::Slice, E>
where
  I: InputIsStreaming<STREAMING>,
  I: Input,
  <I as Input>::Token: AsChar,
{
  if STREAMING {
    streaming::alphanumeric0(input)
  } else {
    complete::alphanumeric0(input)
  }
}

/// Recognizes one or more ASCII numerical and alphabetic characters: 0-9, a-z, A-Z
///
/// *Complete version*: Will return an error if there's not enough input data,
/// or the whole input if no terminating token is found (a non alphanumerical character).
///
/// *Streaming version*: Will return `Err(winnow::Err::Incomplete(_))` if there's not enough input data,
/// or if no terminating token is found (a non alphanumerical character).
///
/// # Example
///
/// ```
/// # use winnow::{Err, error::{Error, ErrorKind}, IResult, Needed};
/// # use winnow::character::alphanumeric1;
/// fn parser(input: &str) -> IResult<&str, &str> {
///     alphanumeric1(input)
/// }
///
/// assert_eq!(parser("21cZ%1"), Ok(("%1", "21cZ")));
/// assert_eq!(parser("&H2"), Err(Err::Error(Error::new("&H2", ErrorKind::AlphaNumeric))));
/// assert_eq!(parser(""), Err(Err::Error(Error::new("", ErrorKind::AlphaNumeric))));
/// ```
///
/// ```
/// # use winnow::{Err, error::ErrorKind, error::Error, IResult, Needed};
/// # use winnow::input::Streaming;
/// # use winnow::character::alphanumeric1;
/// assert_eq!(alphanumeric1::<_, Error<_>, true>(Streaming("21cZ%1")), Ok((Streaming("%1"), "21cZ")));
/// assert_eq!(alphanumeric1::<_, Error<_>, true>(Streaming("&H2")), Err(Err::Error(Error::new(Streaming("&H2"), ErrorKind::AlphaNumeric))));
/// assert_eq!(alphanumeric1::<_, Error<_>, true>(Streaming("")), Err(Err::Incomplete(Needed::new(1))));
/// ```
#[inline(always)]
pub fn alphanumeric1<I, E: ParseError<I>, const STREAMING: bool>(
  input: I,
) -> IResult<I, <I as Input>::Slice, E>
where
  I: InputIsStreaming<STREAMING>,
  I: Input,
  <I as Input>::Token: AsChar,
{
  if STREAMING {
    streaming::alphanumeric1(input)
  } else {
    complete::alphanumeric1(input)
  }
}

/// Recognizes zero or more spaces and tabs.
///
/// *Complete version*: Will return the whole input if no terminating token is found (a non space
/// character).
///
/// *Streaming version*: Will return `Err(winnow::Err::Incomplete(_))` if there's not enough input data,
/// or if no terminating token is found (a non space character).
///
/// # Example
///
/// ```
/// # use winnow::{Err, error::ErrorKind, error::Error, IResult, Needed};
/// # use winnow::input::Streaming;
/// # use winnow::character::space0;
/// assert_eq!(space0::<_, Error<_>, true>(Streaming(" \t21c")), Ok((Streaming("21c"), " \t")));
/// assert_eq!(space0::<_, Error<_>, true>(Streaming("Z21c")), Ok((Streaming("Z21c"), "")));
/// assert_eq!(space0::<_, Error<_>, true>(Streaming("")), Err(Err::Incomplete(Needed::new(1))));
/// ```
#[inline(always)]
pub fn space0<I, E: ParseError<I>, const STREAMING: bool>(
  input: I,
) -> IResult<I, <I as Input>::Slice, E>
where
  I: InputIsStreaming<STREAMING>,
  I: Input,
  <I as Input>::Token: AsChar,
{
  if STREAMING {
    streaming::space0(input)
  } else {
    complete::space0(input)
  }
}

/// Recognizes one or more spaces and tabs.
///
/// *Complete version*: Will return an error if there's not enough input data,
/// or the whole input if no terminating token is found (a non space character).
///
/// *Streaming version*: Will return `Err(winnow::Err::Incomplete(_))` if there's not enough input data,
/// or if no terminating token is found (a non space character).
///
/// # Example
///
/// ```
/// # use winnow::{Err, error::{Error, ErrorKind}, IResult, Needed};
/// # use winnow::character::space1;
/// fn parser(input: &str) -> IResult<&str, &str> {
///     space1(input)
/// }
///
/// assert_eq!(parser(" \t21c"), Ok(("21c", " \t")));
/// assert_eq!(parser("H2"), Err(Err::Error(Error::new("H2", ErrorKind::Space))));
/// assert_eq!(parser(""), Err(Err::Error(Error::new("", ErrorKind::Space))));
/// ```
///
/// ```
/// # use winnow::{Err, error::ErrorKind, error::Error, IResult, Needed};
/// # use winnow::input::Streaming;
/// # use winnow::character::space1;
/// assert_eq!(space1::<_, Error<_>, true>(Streaming(" \t21c")), Ok((Streaming("21c"), " \t")));
/// assert_eq!(space1::<_, Error<_>, true>(Streaming("H2")), Err(Err::Error(Error::new(Streaming("H2"), ErrorKind::Space))));
/// assert_eq!(space1::<_, Error<_>, true>(Streaming("")), Err(Err::Incomplete(Needed::new(1))));
/// ```
#[inline(always)]
pub fn space1<I, E: ParseError<I>, const STREAMING: bool>(
  input: I,
) -> IResult<I, <I as Input>::Slice, E>
where
  I: InputIsStreaming<STREAMING>,
  I: Input,
  <I as Input>::Token: AsChar,
{
  if STREAMING {
    streaming::space1(input)
  } else {
    complete::space1(input)
  }
}

/// Recognizes zero or more spaces, tabs, carriage returns and line feeds.
///
/// *Complete version*: will return the whole input if no terminating token is found (a non space
/// character).
///
/// *Streaming version*: Will return `Err(winnow::Err::Incomplete(_))` if there's not enough input data,
/// or if no terminating token is found (a non space character).
///
/// # Example
///
/// ```
/// # use winnow::{Err, error::ErrorKind, error::Error, IResult, Needed};
/// # use winnow::character::multispace0;
/// fn parser(input: &str) -> IResult<&str, &str> {
///     multispace0(input)
/// }
///
/// assert_eq!(parser(" \t\n\r21c"), Ok(("21c", " \t\n\r")));
/// assert_eq!(parser("Z21c"), Ok(("Z21c", "")));
/// assert_eq!(parser(""), Ok(("", "")));
/// ```
///
/// ```
/// # use winnow::{Err, error::ErrorKind, error::Error, IResult, Needed};
/// # use winnow::input::Streaming;
/// # use winnow::character::multispace0;
/// assert_eq!(multispace0::<_, Error<_>, true>(Streaming(" \t\n\r21c")), Ok((Streaming("21c"), " \t\n\r")));
/// assert_eq!(multispace0::<_, Error<_>, true>(Streaming("Z21c")), Ok((Streaming("Z21c"), "")));
/// assert_eq!(multispace0::<_, Error<_>, true>(Streaming("")), Err(Err::Incomplete(Needed::new(1))));
/// ```
#[inline(always)]
pub fn multispace0<I, E: ParseError<I>, const STREAMING: bool>(
  input: I,
) -> IResult<I, <I as Input>::Slice, E>
where
  I: InputIsStreaming<STREAMING>,
  I: Input,
  <I as Input>::Token: AsChar,
{
  if STREAMING {
    streaming::multispace0(input)
  } else {
    complete::multispace0(input)
  }
}

/// Recognizes one or more spaces, tabs, carriage returns and line feeds.
///
/// *Complete version*: will return an error if there's not enough input data,
/// or the whole input if no terminating token is found (a non space character).
///
/// *Streaming version*: Will return `Err(winnow::Err::Incomplete(_))` if there's not enough input data,
/// or if no terminating token is found (a non space character).
///
/// # Example
///
/// ```
/// # use winnow::{Err, error::{Error, ErrorKind}, IResult, Needed};
/// # use winnow::character::multispace1;
/// fn parser(input: &str) -> IResult<&str, &str> {
///     multispace1(input)
/// }
///
/// assert_eq!(parser(" \t\n\r21c"), Ok(("21c", " \t\n\r")));
/// assert_eq!(parser("H2"), Err(Err::Error(Error::new("H2", ErrorKind::MultiSpace))));
/// assert_eq!(parser(""), Err(Err::Error(Error::new("", ErrorKind::MultiSpace))));
/// ```
///
/// ```
/// # use winnow::{Err, error::ErrorKind, error::Error, IResult, Needed};
/// # use winnow::input::Streaming;
/// # use winnow::character::multispace1;
/// assert_eq!(multispace1::<_, Error<_>, true>(Streaming(" \t\n\r21c")), Ok((Streaming("21c"), " \t\n\r")));
/// assert_eq!(multispace1::<_, Error<_>, true>(Streaming("H2")), Err(Err::Error(Error::new(Streaming("H2"), ErrorKind::MultiSpace))));
/// assert_eq!(multispace1::<_, Error<_>, true>(Streaming("")), Err(Err::Incomplete(Needed::new(1))));
/// ```
#[inline(always)]
pub fn multispace1<I, E: ParseError<I>, const STREAMING: bool>(
  input: I,
) -> IResult<I, <I as Input>::Slice, E>
where
  I: InputIsStreaming<STREAMING>,
  I: Input,
  <I as Input>::Token: AsChar,
{
  if STREAMING {
    streaming::multispace1(input)
  } else {
    complete::multispace1(input)
  }
}

#[doc(hidden)]
macro_rules! ints {
    ($($t:tt)+) => {
        $(
        /// will parse a number in text form to a number
        ///
        /// *Complete version*: can parse until the end of input.
        ///
        /// *Streaming version*: Will return `Err(winnow::Err::Incomplete(_))` if there's not enough input data.
        #[inline(always)]
        pub fn $t<I, E: ParseError<I>, const STREAMING: bool>(input: I) -> IResult<I, $t, E>
            where
              I: InputIsStreaming<STREAMING>,
              I: Input,
              <I as Input>::Token: AsChar + Copy,
            {
                if STREAMING {
                  streaming::$t(input)
                } else {
                  complete::$t(input)
                }
            }
        )+
    }
}

ints! { i8 i16 i32 i64 i128 }

#[doc(hidden)]
macro_rules! uints {
    ($($t:tt)+) => {
        $(
        /// will parse a number in text form to a number
        ///
        /// *Complete version*: can parse until the end of input.
        ///
        /// *Streaming version*: Will return `Err(winnow::Err::Incomplete(_))` if there's not enough input data.
        #[inline(always)]
        pub fn $t<I, E: ParseError<I>, const STREAMING: bool>(input: I) -> IResult<I, $t, E>
            where
              I: InputIsStreaming<STREAMING>,
              I: Input,
              <I as Input>::Token: AsChar,
            {
                if STREAMING {
                  streaming::$t(input)
                } else {
                  complete::$t(input)
                }
            }
        )+
    }
}

uints! { u8 u16 u32 u64 u128 }

/// Recognizes floating point number in text format and returns a f32.
///
/// *Complete version*: Can parse until the end of input.
///
/// *Streaming version*: Will return `Err(winnow::Err::Incomplete(_))` if there is not enough data.
///
/// # Example
///
/// ```rust
/// # use winnow::{Err, error::ErrorKind, error::Error, Needed};
/// # use winnow::Needed::Size;
/// use winnow::character::f32;
///
/// let parser = |s| {
///   f32(s)
/// };
///
/// assert_eq!(parser("11e-1"), Ok(("", 1.1)));
/// assert_eq!(parser("123E-02"), Ok(("", 1.23)));
/// assert_eq!(parser("123K-01"), Ok(("K-01", 123.0)));
/// assert_eq!(parser("abc"), Err(Err::Error(Error::new("abc", ErrorKind::Float))));
/// ```
///
/// ```rust
/// # use winnow::{Err, error::ErrorKind, error::Error, Needed};
/// # use winnow::Needed::Size;
/// # use winnow::input::Streaming;
/// use winnow::character::f32;
///
/// let parser = |s| {
///   f32(s)
/// };
///
/// assert_eq!(parser(Streaming("11e-1 ")), Ok((Streaming(" "), 1.1)));
/// assert_eq!(parser(Streaming("11e-1")), Err(Err::Incomplete(Needed::new(1))));
/// assert_eq!(parser(Streaming("123E-02")), Err(Err::Incomplete(Needed::new(1))));
/// assert_eq!(parser(Streaming("123K-01")), Ok((Streaming("K-01"), 123.0)));
/// assert_eq!(parser(Streaming("abc")), Err(Err::Error(Error::new(Streaming("abc"), ErrorKind::Float))));
/// ```
#[inline(always)]
pub fn f32<I, E: ParseError<I>, const STREAMING: bool>(input: I) -> IResult<I, f32, E>
where
  I: InputIsStreaming<STREAMING>,
  I: Input,
  I: Offset + Compare<&'static str>,
  <I as Input>::Slice: ParseTo<f32>,
  <I as Input>::Token: AsChar + Copy,
  <I as Input>::IterOffsets: Clone,
  I: AsBytes,
{
  if STREAMING {
    crate::number::streaming::float(input)
  } else {
    crate::number::complete::float(input)
  }
}

/// Recognizes floating point number in text format and returns a f64.
///
/// *Complete version*: Can parse until the end of input.
///
/// *Streaming version*: Will return `Err(winnow::Err::Incomplete(_))` if there is not enough data.
///
/// # Example
///
/// ```rust
/// # use winnow::{Err, error::ErrorKind, error::Error, Needed};
/// # use winnow::Needed::Size;
/// use winnow::character::f64;
///
/// let parser = |s| {
///   f64(s)
/// };
///
/// assert_eq!(parser("11e-1"), Ok(("", 1.1)));
/// assert_eq!(parser("123E-02"), Ok(("", 1.23)));
/// assert_eq!(parser("123K-01"), Ok(("K-01", 123.0)));
/// assert_eq!(parser("abc"), Err(Err::Error(Error::new("abc", ErrorKind::Float))));
/// ```
///
/// ```rust
/// # use winnow::{Err, error::ErrorKind, error::Error, Needed};
/// # use winnow::Needed::Size;
/// # use winnow::input::Streaming;
/// use winnow::character::f64;
///
/// let parser = |s| {
///   f64(s)
/// };
///
/// assert_eq!(parser(Streaming("11e-1 ")), Ok((Streaming(" "), 1.1)));
/// assert_eq!(parser(Streaming("11e-1")), Err(Err::Incomplete(Needed::new(1))));
/// assert_eq!(parser(Streaming("123E-02")), Err(Err::Incomplete(Needed::new(1))));
/// assert_eq!(parser(Streaming("123K-01")), Ok((Streaming("K-01"), 123.0)));
/// assert_eq!(parser(Streaming("abc")), Err(Err::Error(Error::new(Streaming("abc"), ErrorKind::Float))));
/// ```
#[inline(always)]
pub fn f64<I, E: ParseError<I>, const STREAMING: bool>(input: I) -> IResult<I, f64, E>
where
  I: InputIsStreaming<STREAMING>,
  I: Input,
  I: Offset + Compare<&'static str>,
  <I as Input>::Slice: ParseTo<f64>,
  <I as Input>::Token: AsChar + Copy,
  <I as Input>::IterOffsets: Clone,
  I: AsBytes,
{
  if STREAMING {
    crate::number::streaming::double(input)
  } else {
    crate::number::complete::double(input)
  }
}

/// Recognizes floating point number in a byte string and returns the corresponding slice.
///
/// *Complete version*: Can parse until the end of input.
///
/// *Streaming version*: Will return `Err(winnow::Err::Incomplete(_))` if there is not enough data.
///
/// # Example
///
/// ```rust
/// # use winnow::{Err, error::ErrorKind, error::Error, Needed};
/// # use winnow::Needed::Size;
/// use winnow::character::recognize_float;
///
/// let parser = |s| {
///   recognize_float(s)
/// };
///
/// assert_eq!(parser("11e-1"), Ok(("", "11e-1")));
/// assert_eq!(parser("123E-02"), Ok(("", "123E-02")));
/// assert_eq!(parser("123K-01"), Ok(("K-01", "123")));
/// assert_eq!(parser("abc"), Err(Err::Error(Error::new("abc", ErrorKind::Char))));
/// ```
///
/// ```rust
/// # use winnow::{Err, error::ErrorKind, error::Error, Needed};
/// # use winnow::input::Streaming;
/// use winnow::character::recognize_float;
///
/// let parser = |s| {
///   recognize_float(s)
/// };
///
/// assert_eq!(parser(Streaming("11e-1;")), Ok((Streaming(";"), "11e-1")));
/// assert_eq!(parser(Streaming("123E-02;")), Ok((Streaming(";"), "123E-02")));
/// assert_eq!(parser(Streaming("123K-01")), Ok((Streaming("K-01"), "123")));
/// assert_eq!(parser(Streaming("abc")), Err(Err::Error(Error::new(Streaming("abc"), ErrorKind::Char))));
/// ```
#[inline(always)]
pub fn recognize_float<I, E: ParseError<I>, const STREAMING: bool>(
  input: I,
) -> IResult<I, <I as Input>::Slice, E>
where
  I: InputIsStreaming<STREAMING>,
  I: Input,
  I: Offset + Compare<&'static str>,
  <I as Input>::Token: AsChar + Copy,
  <I as Input>::IterOffsets: Clone,
  I: AsBytes,
{
  if STREAMING {
    crate::number::streaming::recognize_float(input)
  } else {
    crate::number::complete::recognize_float(input)
  }
}

/// Recognizes a floating point number in text format
///
/// It returns a tuple of (`sign`, `integer part`, `fraction part` and `exponent`) of the input
/// data.
///
/// *Complete version*: Can parse until the end of input.
///
/// *Streaming version*: Will return `Err(winnow::Err::Incomplete(_))` if there is not enough data.
///
#[inline(always)]
#[allow(clippy::type_complexity)]
pub fn recognize_float_parts<I, E: ParseError<I>, const STREAMING: bool>(
  input: I,
) -> IResult<I, (bool, <I as Input>::Slice, <I as Input>::Slice, i32), E>
where
  I: InputIsStreaming<STREAMING>,
  I: Input + Compare<&'static [u8]> + AsBytes,
  <I as Input>::Token: AsChar + Copy,
  <I as Input>::Slice: SliceLen,
{
  if STREAMING {
    crate::number::streaming::recognize_float_parts(input)
  } else {
    crate::number::complete::recognize_float_parts(input)
  }
}

#[inline]
#[doc(hidden)]
#[deprecated(since = "8.0.0", note = "Replaced with `AsChar::is_alpha`")]
pub fn is_alphabetic(chr: u8) -> bool {
  matches!(chr, 0x41..=0x5A | 0x61..=0x7A)
}

#[inline]
#[doc(hidden)]
#[deprecated(since = "8.0.0", note = "Replaced with `AsChar::is_dec_digit`")]
pub fn is_digit(chr: u8) -> bool {
  matches!(chr, 0x30..=0x39)
}

#[inline]
#[doc(hidden)]
#[deprecated(since = "8.0.0", note = "Replaced with `AsChar::is_hex_digit`")]
pub fn is_hex_digit(chr: u8) -> bool {
  matches!(chr, 0x30..=0x39 | 0x41..=0x46 | 0x61..=0x66)
}

#[inline]
#[doc(hidden)]
#[deprecated(since = "8.0.0", note = "Replaced with `AsChar::is_oct_digit`")]
pub fn is_oct_digit(chr: u8) -> bool {
  matches!(chr, 0x30..=0x37)
}

#[inline]
#[doc(hidden)]
#[deprecated(since = "8.0.0", note = "Replaced with `AsChar::is_alphanum`")]
pub fn is_alphanumeric(chr: u8) -> bool {
  #![allow(deprecated)]
  is_alphabetic(chr) || is_digit(chr)
}

#[inline]
#[doc(hidden)]
#[deprecated(since = "8.0.0", note = "Replaced with `AsChar::is_space`")]
pub fn is_space(chr: u8) -> bool {
  chr == b' ' || chr == b'\t'
}

#[inline]
#[doc(hidden)]
#[deprecated(since = "8.0.0", note = "Replaced with `AsChar::is_newline`")]
pub fn is_newline(chr: u8) -> bool {
  chr == b'\n'
}
