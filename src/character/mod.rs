//! Character specific parsers and combinators
//!
//! Functions recognizing specific characters

#![allow(deprecated)] // will just become `pub(crate)` later

pub mod complete;
pub mod streaming;
#[cfg(test)]
mod tests;

use crate::lib::std::ops::{Add, Shl};

use crate::combinator::opt;
use crate::error::ParseError;
use crate::error::{ErrMode, ErrorKind, Needed};
use crate::stream::Compare;
use crate::stream::{AsBStr, AsChar, Input, InputIsPartial, Offset, ParseSlice};
use crate::IResult;
use crate::Parser;

/// Recognizes the string "\r\n".
///
/// *Complete version*: Will return an error if there's not enough input data.
///
/// *Partial version*: Will return `Err(winnow::error::ErrMode::Incomplete(_))` if there's not enough input data.
///
/// # Example
///
/// ```
/// # use winnow::{error::ErrMode, error::{Error, ErrorKind}, IResult};
/// # use winnow::character::crlf;
/// fn parser(input: &str) -> IResult<&str, &str> {
///     crlf(input)
/// }
///
/// assert_eq!(parser("\r\nc"), Ok(("c", "\r\n")));
/// assert_eq!(parser("ab\r\nc"), Err(ErrMode::Backtrack(Error::new("ab\r\nc", ErrorKind::CrLf))));
/// assert_eq!(parser(""), Err(ErrMode::Backtrack(Error::new("", ErrorKind::CrLf))));
/// ```
///
/// ```
/// # use winnow::{error::ErrMode, error::ErrorKind, error::Error, IResult, error::Needed};
/// # use winnow::Partial;
/// # use winnow::character::crlf;
/// assert_eq!(crlf::<_, Error<_>, true>(Partial("\r\nc")), Ok((Partial("c"), "\r\n")));
/// assert_eq!(crlf::<_, Error<_>, true>(Partial("ab\r\nc")), Err(ErrMode::Backtrack(Error::new(Partial("ab\r\nc"), ErrorKind::CrLf))));
/// assert_eq!(crlf::<_, Error<_>, true>(Partial("")), Err(ErrMode::Incomplete(Needed::new(2))));
/// ```
#[inline(always)]
pub fn crlf<I, E: ParseError<I>, const PARTIAL: bool>(
    input: I,
) -> IResult<I, <I as Input>::Slice, E>
where
    I: InputIsPartial<PARTIAL>,
    I: Input,
    I: Compare<&'static str>,
{
    if PARTIAL {
        streaming::crlf(input)
    } else {
        complete::crlf(input)
    }
}

/// Recognizes a string of any char except '\r\n' or '\n'.
///
/// *Complete version*: Will return an error if there's not enough input data.
///
/// *Partial version*: Will return `Err(winnow::error::ErrMode::Incomplete(_))` if there's not enough input data.
///
/// # Example
///
/// ```
/// # use winnow::{error::ErrMode, error::{Error, ErrorKind}, IResult, error::Needed};
/// # use winnow::character::not_line_ending;
/// fn parser(input: &str) -> IResult<&str, &str> {
///     not_line_ending(input)
/// }
///
/// assert_eq!(parser("ab\r\nc"), Ok(("\r\nc", "ab")));
/// assert_eq!(parser("ab\nc"), Ok(("\nc", "ab")));
/// assert_eq!(parser("abc"), Ok(("", "abc")));
/// assert_eq!(parser(""), Ok(("", "")));
/// assert_eq!(parser("a\rb\nc"), Err(ErrMode::Backtrack(Error { input: "a\rb\nc", kind: ErrorKind::Tag })));
/// assert_eq!(parser("a\rbc"), Err(ErrMode::Backtrack(Error { input: "a\rbc", kind: ErrorKind::Tag })));
/// ```
///
/// ```
/// # use winnow::{error::ErrMode, error::{Error, ErrorKind}, IResult, error::Needed};
/// # use winnow::Partial;
/// # use winnow::character::not_line_ending;
/// assert_eq!(not_line_ending::<_, Error<_>, true>(Partial("ab\r\nc")), Ok((Partial("\r\nc"), "ab")));
/// assert_eq!(not_line_ending::<_, Error<_>, true>(Partial("abc")), Err(ErrMode::Incomplete(Needed::Unknown)));
/// assert_eq!(not_line_ending::<_, Error<_>, true>(Partial("")), Err(ErrMode::Incomplete(Needed::Unknown)));
/// assert_eq!(not_line_ending::<_, Error<_>, true>(Partial("a\rb\nc")), Err(ErrMode::Backtrack(Error::new(Partial("a\rb\nc"), ErrorKind::Tag ))));
/// assert_eq!(not_line_ending::<_, Error<_>, true>(Partial("a\rbc")), Err(ErrMode::Backtrack(Error::new(Partial("a\rbc"), ErrorKind::Tag ))));
/// ```
#[inline(always)]
pub fn not_line_ending<I, E: ParseError<I>, const PARTIAL: bool>(
    input: I,
) -> IResult<I, <I as Input>::Slice, E>
where
    I: InputIsPartial<PARTIAL>,
    I: Input + AsBStr,
    I: Compare<&'static str>,
    <I as Input>::Token: AsChar,
{
    if PARTIAL {
        streaming::not_line_ending(input)
    } else {
        complete::not_line_ending(input)
    }
}

/// Recognizes an end of line (both '\n' and '\r\n').
///
/// *Complete version*: Will return an error if there's not enough input data.
///
/// *Partial version*: Will return `Err(winnow::error::ErrMode::Incomplete(_))` if there's not enough input data.
///
/// # Example
///
/// ```
/// # use winnow::{error::ErrMode, error::{Error, ErrorKind}, IResult, error::Needed};
/// # use winnow::character::line_ending;
/// fn parser(input: &str) -> IResult<&str, &str> {
///     line_ending(input)
/// }
///
/// assert_eq!(parser("\r\nc"), Ok(("c", "\r\n")));
/// assert_eq!(parser("ab\r\nc"), Err(ErrMode::Backtrack(Error::new("ab\r\nc", ErrorKind::CrLf))));
/// assert_eq!(parser(""), Err(ErrMode::Backtrack(Error::new("", ErrorKind::CrLf))));
/// ```
///
/// ```
/// # use winnow::{error::ErrMode, error::ErrorKind, error::Error, IResult, error::Needed};
/// # use winnow::Partial;
/// # use winnow::character::line_ending;
/// assert_eq!(line_ending::<_, Error<_>, true>(Partial("\r\nc")), Ok((Partial("c"), "\r\n")));
/// assert_eq!(line_ending::<_, Error<_>, true>(Partial("ab\r\nc")), Err(ErrMode::Backtrack(Error::new(Partial("ab\r\nc"), ErrorKind::CrLf))));
/// assert_eq!(line_ending::<_, Error<_>, true>(Partial("")), Err(ErrMode::Incomplete(Needed::new(1))));
/// ```
#[inline(always)]
pub fn line_ending<I, E: ParseError<I>, const PARTIAL: bool>(
    input: I,
) -> IResult<I, <I as Input>::Slice, E>
where
    I: InputIsPartial<PARTIAL>,
    I: Input,
    I: Compare<&'static str>,
{
    if PARTIAL {
        streaming::line_ending(input)
    } else {
        complete::line_ending(input)
    }
}

/// Matches a newline character '\n'.
///
/// *Complete version*: Will return an error if there's not enough input data.
///
/// *Partial version*: Will return `Err(winnow::error::ErrMode::Incomplete(_))` if there's not enough input data.
///
/// # Example
///
/// ```
/// # use winnow::{error::ErrMode, error::{Error, ErrorKind}, IResult, error::Needed};
/// # use winnow::character::newline;
/// fn parser(input: &str) -> IResult<&str, char> {
///     newline(input)
/// }
///
/// assert_eq!(parser("\nc"), Ok(("c", '\n')));
/// assert_eq!(parser("\r\nc"), Err(ErrMode::Backtrack(Error::new("\r\nc", ErrorKind::Char))));
/// assert_eq!(parser(""), Err(ErrMode::Backtrack(Error::new("", ErrorKind::Char))));
/// ```
///
/// ```
/// # use winnow::{error::ErrMode, error::ErrorKind, error::Error, IResult, error::Needed};
/// # use winnow::Partial;
/// # use winnow::character::newline;
/// assert_eq!(newline::<_, Error<_>, true>(Partial("\nc")), Ok((Partial("c"), '\n')));
/// assert_eq!(newline::<_, Error<_>, true>(Partial("\r\nc")), Err(ErrMode::Backtrack(Error::new(Partial("\r\nc"), ErrorKind::Char))));
/// assert_eq!(newline::<_, Error<_>, true>(Partial("")), Err(ErrMode::Incomplete(Needed::new(1))));
/// ```
#[inline(always)]
pub fn newline<I, Error: ParseError<I>, const PARTIAL: bool>(input: I) -> IResult<I, char, Error>
where
    I: InputIsPartial<PARTIAL>,
    I: Input,
    <I as Input>::Token: AsChar,
{
    if PARTIAL {
        streaming::newline(input)
    } else {
        complete::newline(input)
    }
}

/// Matches a tab character '\t'.
///
/// *Complete version*: Will return an error if there's not enough input data.
///
/// *Partial version*: Will return `Err(winnow::error::ErrMode::Incomplete(_))` if there's not enough input data.
///
/// # Example
///
/// ```
/// # use winnow::{error::ErrMode, error::{Error, ErrorKind}, IResult, error::Needed};
/// # use winnow::character::tab;
/// fn parser(input: &str) -> IResult<&str, char> {
///     tab(input)
/// }
///
/// assert_eq!(parser("\tc"), Ok(("c", '\t')));
/// assert_eq!(parser("\r\nc"), Err(ErrMode::Backtrack(Error::new("\r\nc", ErrorKind::Char))));
/// assert_eq!(parser(""), Err(ErrMode::Backtrack(Error::new("", ErrorKind::Char))));
/// ```
///
/// ```
/// # use winnow::{error::ErrMode, error::ErrorKind, error::Error, IResult, error::Needed};
/// # use winnow::Partial;
/// # use winnow::character::tab;
/// assert_eq!(tab::<_, Error<_>, true>(Partial("\tc")), Ok((Partial("c"), '\t')));
/// assert_eq!(tab::<_, Error<_>, true>(Partial("\r\nc")), Err(ErrMode::Backtrack(Error::new(Partial("\r\nc"), ErrorKind::Char))));
/// assert_eq!(tab::<_, Error<_>, true>(Partial("")), Err(ErrMode::Incomplete(Needed::new(1))));
/// ```
#[inline(always)]
pub fn tab<I, Error: ParseError<I>, const PARTIAL: bool>(input: I) -> IResult<I, char, Error>
where
    I: InputIsPartial<PARTIAL>,
    I: Input,
    <I as Input>::Token: AsChar,
{
    if PARTIAL {
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
/// *Partial version*: Will return `Err(winnow::error::ErrMode::Incomplete(_))` if there's not enough input data,
/// or if no terminating token is found (a non alphabetic character).
///
/// # Example
///
/// ```
/// # use winnow::{error::ErrMode, error::ErrorKind, error::Error, IResult, error::Needed};
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
/// # use winnow::{error::ErrMode, error::ErrorKind, error::Error, IResult, error::Needed};
/// # use winnow::Partial;
/// # use winnow::character::alpha0;
/// assert_eq!(alpha0::<_, Error<_>, true>(Partial("ab1c")), Ok((Partial("1c"), "ab")));
/// assert_eq!(alpha0::<_, Error<_>, true>(Partial("1c")), Ok((Partial("1c"), "")));
/// assert_eq!(alpha0::<_, Error<_>, true>(Partial("")), Err(ErrMode::Incomplete(Needed::new(1))));
/// ```
#[inline(always)]
pub fn alpha0<I, E: ParseError<I>, const PARTIAL: bool>(
    input: I,
) -> IResult<I, <I as Input>::Slice, E>
where
    I: InputIsPartial<PARTIAL>,
    I: Input,
    <I as Input>::Token: AsChar,
{
    if PARTIAL {
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
/// *Partial version*: Will return `Err(winnow::error::ErrMode::Incomplete(_))` if there's not enough input data,
/// or if no terminating token is found (a non alphabetic character).
///
/// # Example
///
/// ```
/// # use winnow::{error::ErrMode, error::{Error, ErrorKind}, IResult, error::Needed};
/// # use winnow::character::alpha1;
/// fn parser(input: &str) -> IResult<&str, &str> {
///     alpha1(input)
/// }
///
/// assert_eq!(parser("aB1c"), Ok(("1c", "aB")));
/// assert_eq!(parser("1c"), Err(ErrMode::Backtrack(Error::new("1c", ErrorKind::Alpha))));
/// assert_eq!(parser(""), Err(ErrMode::Backtrack(Error::new("", ErrorKind::Alpha))));
/// ```
///
/// ```
/// # use winnow::{error::ErrMode, error::ErrorKind, error::Error, IResult, error::Needed};
/// # use winnow::Partial;
/// # use winnow::character::alpha1;
/// assert_eq!(alpha1::<_, Error<_>, true>(Partial("aB1c")), Ok((Partial("1c"), "aB")));
/// assert_eq!(alpha1::<_, Error<_>, true>(Partial("1c")), Err(ErrMode::Backtrack(Error::new(Partial("1c"), ErrorKind::Alpha))));
/// assert_eq!(alpha1::<_, Error<_>, true>(Partial("")), Err(ErrMode::Incomplete(Needed::new(1))));
/// ```
#[inline(always)]
pub fn alpha1<I, E: ParseError<I>, const PARTIAL: bool>(
    input: I,
) -> IResult<I, <I as Input>::Slice, E>
where
    I: InputIsPartial<PARTIAL>,
    I: Input,
    <I as Input>::Token: AsChar,
{
    if PARTIAL {
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
/// *Partial version*: Will return `Err(winnow::error::ErrMode::Incomplete(_))` if there's not enough input data,
/// or if no terminating token is found (a non digit character).
///
/// # Example
///
/// ```
/// # use winnow::{error::ErrMode, error::ErrorKind, error::Error, IResult, error::Needed};
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
/// # use winnow::{error::ErrMode, error::ErrorKind, error::Error, IResult, error::Needed};
/// # use winnow::Partial;
/// # use winnow::character::digit0;
/// assert_eq!(digit0::<_, Error<_>, true>(Partial("21c")), Ok((Partial("c"), "21")));
/// assert_eq!(digit0::<_, Error<_>, true>(Partial("a21c")), Ok((Partial("a21c"), "")));
/// assert_eq!(digit0::<_, Error<_>, true>(Partial("")), Err(ErrMode::Incomplete(Needed::new(1))));
/// ```
#[inline(always)]
pub fn digit0<I, E: ParseError<I>, const PARTIAL: bool>(
    input: I,
) -> IResult<I, <I as Input>::Slice, E>
where
    I: InputIsPartial<PARTIAL>,
    I: Input,
    <I as Input>::Token: AsChar,
{
    if PARTIAL {
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
/// *Partial version*: Will return `Err(winnow::error::ErrMode::Incomplete(_))` if there's not enough input data,
/// or if no terminating token is found (a non digit character).
///
/// # Example
///
/// ```
/// # use winnow::{error::ErrMode, error::{Error, ErrorKind}, IResult, error::Needed};
/// # use winnow::character::digit1;
/// fn parser(input: &str) -> IResult<&str, &str> {
///     digit1(input)
/// }
///
/// assert_eq!(parser("21c"), Ok(("c", "21")));
/// assert_eq!(parser("c1"), Err(ErrMode::Backtrack(Error::new("c1", ErrorKind::Digit))));
/// assert_eq!(parser(""), Err(ErrMode::Backtrack(Error::new("", ErrorKind::Digit))));
/// ```
///
/// ```
/// # use winnow::{error::ErrMode, error::ErrorKind, error::Error, IResult, error::Needed};
/// # use winnow::Partial;
/// # use winnow::character::digit1;
/// assert_eq!(digit1::<_, Error<_>, true>(Partial("21c")), Ok((Partial("c"), "21")));
/// assert_eq!(digit1::<_, Error<_>, true>(Partial("c1")), Err(ErrMode::Backtrack(Error::new(Partial("c1"), ErrorKind::Digit))));
/// assert_eq!(digit1::<_, Error<_>, true>(Partial("")), Err(ErrMode::Incomplete(Needed::new(1))));
/// ```
///
/// ## Parsing an integer
///
/// You can use `digit1` in combination with [`Parser::map_res`][crate::Parser::map_res] to parse an integer:
///
/// ```
/// # use winnow::{error::ErrMode, error::{Error, ErrorKind}, IResult, error::Needed, Parser};
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
pub fn digit1<I, E: ParseError<I>, const PARTIAL: bool>(
    input: I,
) -> IResult<I, <I as Input>::Slice, E>
where
    I: InputIsPartial<PARTIAL>,
    I: Input,
    <I as Input>::Token: AsChar,
{
    if PARTIAL {
        streaming::digit1(input)
    } else {
        complete::digit1(input)
    }
}

/// Recognizes zero or more ASCII hexadecimal numerical characters: 0-9, A-F, a-f
///
/// *Complete version*: Will return the whole input if no terminating token is found (a non hexadecimal digit character).
///
/// *Partial version*: Will return `Err(winnow::error::ErrMode::Incomplete(_))` if there's not enough input data,
/// or if no terminating token is found (a non hexadecimal digit character).
///
/// # Example
///
/// ```
/// # use winnow::{error::ErrMode, error::ErrorKind, error::Error, IResult, error::Needed};
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
/// # use winnow::{error::ErrMode, error::ErrorKind, error::Error, IResult, error::Needed};
/// # use winnow::Partial;
/// # use winnow::character::hex_digit0;
/// assert_eq!(hex_digit0::<_, Error<_>, true>(Partial("21cZ")), Ok((Partial("Z"), "21c")));
/// assert_eq!(hex_digit0::<_, Error<_>, true>(Partial("Z21c")), Ok((Partial("Z21c"), "")));
/// assert_eq!(hex_digit0::<_, Error<_>, true>(Partial("")), Err(ErrMode::Incomplete(Needed::new(1))));
/// ```
#[inline(always)]
pub fn hex_digit0<I, E: ParseError<I>, const PARTIAL: bool>(
    input: I,
) -> IResult<I, <I as Input>::Slice, E>
where
    I: InputIsPartial<PARTIAL>,
    I: Input,
    <I as Input>::Token: AsChar,
{
    if PARTIAL {
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
/// *Partial version*: Will return `Err(winnow::error::ErrMode::Incomplete(_))` if there's not enough input data,
/// or if no terminating token is found (a non hexadecimal digit character).
///
/// # Example
///
/// ```
/// # use winnow::{error::ErrMode, error::{Error, ErrorKind}, IResult, error::Needed};
/// # use winnow::character::hex_digit1;
/// fn parser(input: &str) -> IResult<&str, &str> {
///     hex_digit1(input)
/// }
///
/// assert_eq!(parser("21cZ"), Ok(("Z", "21c")));
/// assert_eq!(parser("H2"), Err(ErrMode::Backtrack(Error::new("H2", ErrorKind::HexDigit))));
/// assert_eq!(parser(""), Err(ErrMode::Backtrack(Error::new("", ErrorKind::HexDigit))));
/// ```
///
/// ```
/// # use winnow::{error::ErrMode, error::ErrorKind, error::Error, IResult, error::Needed};
/// # use winnow::Partial;
/// # use winnow::character::hex_digit1;
/// assert_eq!(hex_digit1::<_, Error<_>, true>(Partial("21cZ")), Ok((Partial("Z"), "21c")));
/// assert_eq!(hex_digit1::<_, Error<_>, true>(Partial("H2")), Err(ErrMode::Backtrack(Error::new(Partial("H2"), ErrorKind::HexDigit))));
/// assert_eq!(hex_digit1::<_, Error<_>, true>(Partial("")), Err(ErrMode::Incomplete(Needed::new(1))));
/// ```
#[inline(always)]
pub fn hex_digit1<I, E: ParseError<I>, const PARTIAL: bool>(
    input: I,
) -> IResult<I, <I as Input>::Slice, E>
where
    I: InputIsPartial<PARTIAL>,
    I: Input,
    <I as Input>::Token: AsChar,
{
    if PARTIAL {
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
/// *Partial version*: Will return `Err(winnow::error::ErrMode::Incomplete(_))` if there's not enough input data,
/// or if no terminating token is found (a non octal digit character).
///
/// # Example
///
/// ```
/// # use winnow::{error::ErrMode, error::ErrorKind, error::Error, IResult, error::Needed};
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
/// # use winnow::{error::ErrMode, error::ErrorKind, error::Error, IResult, error::Needed};
/// # use winnow::Partial;
/// # use winnow::character::oct_digit0;
/// assert_eq!(oct_digit0::<_, Error<_>, true>(Partial("21cZ")), Ok((Partial("cZ"), "21")));
/// assert_eq!(oct_digit0::<_, Error<_>, true>(Partial("Z21c")), Ok((Partial("Z21c"), "")));
/// assert_eq!(oct_digit0::<_, Error<_>, true>(Partial("")), Err(ErrMode::Incomplete(Needed::new(1))));
/// ```
#[inline(always)]
pub fn oct_digit0<I, E: ParseError<I>, const PARTIAL: bool>(
    input: I,
) -> IResult<I, <I as Input>::Slice, E>
where
    I: InputIsPartial<PARTIAL>,
    I: Input,
    <I as Input>::Token: AsChar,
{
    if PARTIAL {
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
/// *Partial version*: Will return `Err(winnow::error::ErrMode::Incomplete(_))` if there's not enough input data,
/// or if no terminating token is found (a non octal digit character).
///
/// # Example
///
/// ```
/// # use winnow::{error::ErrMode, error::{Error, ErrorKind}, IResult, error::Needed};
/// # use winnow::character::oct_digit1;
/// fn parser(input: &str) -> IResult<&str, &str> {
///     oct_digit1(input)
/// }
///
/// assert_eq!(parser("21cZ"), Ok(("cZ", "21")));
/// assert_eq!(parser("H2"), Err(ErrMode::Backtrack(Error::new("H2", ErrorKind::OctDigit))));
/// assert_eq!(parser(""), Err(ErrMode::Backtrack(Error::new("", ErrorKind::OctDigit))));
/// ```
///
/// ```
/// # use winnow::{error::ErrMode, error::ErrorKind, error::Error, IResult, error::Needed};
/// # use winnow::Partial;
/// # use winnow::character::oct_digit1;
/// assert_eq!(oct_digit1::<_, Error<_>, true>(Partial("21cZ")), Ok((Partial("cZ"), "21")));
/// assert_eq!(oct_digit1::<_, Error<_>, true>(Partial("H2")), Err(ErrMode::Backtrack(Error::new(Partial("H2"), ErrorKind::OctDigit))));
/// assert_eq!(oct_digit1::<_, Error<_>, true>(Partial("")), Err(ErrMode::Incomplete(Needed::new(1))));
/// ```
#[inline(always)]
pub fn oct_digit1<I, E: ParseError<I>, const PARTIAL: bool>(
    input: I,
) -> IResult<I, <I as Input>::Slice, E>
where
    I: InputIsPartial<PARTIAL>,
    I: Input,
    <I as Input>::Token: AsChar,
{
    if PARTIAL {
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
/// *Partial version*: Will return `Err(winnow::error::ErrMode::Incomplete(_))` if there's not enough input data,
/// or if no terminating token is found (a non alphanumerical character).
///
/// # Example
///
/// ```
/// # use winnow::{error::ErrMode, error::ErrorKind, error::Error, IResult, error::Needed};
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
/// # use winnow::{error::ErrMode, error::ErrorKind, error::Error, IResult, error::Needed};
/// # use winnow::Partial;
/// # use winnow::character::alphanumeric0;
/// assert_eq!(alphanumeric0::<_, Error<_>, true>(Partial("21cZ%1")), Ok((Partial("%1"), "21cZ")));
/// assert_eq!(alphanumeric0::<_, Error<_>, true>(Partial("&Z21c")), Ok((Partial("&Z21c"), "")));
/// assert_eq!(alphanumeric0::<_, Error<_>, true>(Partial("")), Err(ErrMode::Incomplete(Needed::new(1))));
/// ```
#[inline(always)]
pub fn alphanumeric0<I, E: ParseError<I>, const PARTIAL: bool>(
    input: I,
) -> IResult<I, <I as Input>::Slice, E>
where
    I: InputIsPartial<PARTIAL>,
    I: Input,
    <I as Input>::Token: AsChar,
{
    if PARTIAL {
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
/// *Partial version*: Will return `Err(winnow::error::ErrMode::Incomplete(_))` if there's not enough input data,
/// or if no terminating token is found (a non alphanumerical character).
///
/// # Example
///
/// ```
/// # use winnow::{error::ErrMode, error::{Error, ErrorKind}, IResult, error::Needed};
/// # use winnow::character::alphanumeric1;
/// fn parser(input: &str) -> IResult<&str, &str> {
///     alphanumeric1(input)
/// }
///
/// assert_eq!(parser("21cZ%1"), Ok(("%1", "21cZ")));
/// assert_eq!(parser("&H2"), Err(ErrMode::Backtrack(Error::new("&H2", ErrorKind::AlphaNumeric))));
/// assert_eq!(parser(""), Err(ErrMode::Backtrack(Error::new("", ErrorKind::AlphaNumeric))));
/// ```
///
/// ```
/// # use winnow::{error::ErrMode, error::ErrorKind, error::Error, IResult, error::Needed};
/// # use winnow::Partial;
/// # use winnow::character::alphanumeric1;
/// assert_eq!(alphanumeric1::<_, Error<_>, true>(Partial("21cZ%1")), Ok((Partial("%1"), "21cZ")));
/// assert_eq!(alphanumeric1::<_, Error<_>, true>(Partial("&H2")), Err(ErrMode::Backtrack(Error::new(Partial("&H2"), ErrorKind::AlphaNumeric))));
/// assert_eq!(alphanumeric1::<_, Error<_>, true>(Partial("")), Err(ErrMode::Incomplete(Needed::new(1))));
/// ```
#[inline(always)]
pub fn alphanumeric1<I, E: ParseError<I>, const PARTIAL: bool>(
    input: I,
) -> IResult<I, <I as Input>::Slice, E>
where
    I: InputIsPartial<PARTIAL>,
    I: Input,
    <I as Input>::Token: AsChar,
{
    if PARTIAL {
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
/// *Partial version*: Will return `Err(winnow::error::ErrMode::Incomplete(_))` if there's not enough input data,
/// or if no terminating token is found (a non space character).
///
/// # Example
///
/// ```
/// # use winnow::{error::ErrMode, error::ErrorKind, error::Error, IResult, error::Needed};
/// # use winnow::Partial;
/// # use winnow::character::space0;
/// assert_eq!(space0::<_, Error<_>, true>(Partial(" \t21c")), Ok((Partial("21c"), " \t")));
/// assert_eq!(space0::<_, Error<_>, true>(Partial("Z21c")), Ok((Partial("Z21c"), "")));
/// assert_eq!(space0::<_, Error<_>, true>(Partial("")), Err(ErrMode::Incomplete(Needed::new(1))));
/// ```
#[inline(always)]
pub fn space0<I, E: ParseError<I>, const PARTIAL: bool>(
    input: I,
) -> IResult<I, <I as Input>::Slice, E>
where
    I: InputIsPartial<PARTIAL>,
    I: Input,
    <I as Input>::Token: AsChar,
{
    if PARTIAL {
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
/// *Partial version*: Will return `Err(winnow::error::ErrMode::Incomplete(_))` if there's not enough input data,
/// or if no terminating token is found (a non space character).
///
/// # Example
///
/// ```
/// # use winnow::{error::ErrMode, error::{Error, ErrorKind}, IResult, error::Needed};
/// # use winnow::character::space1;
/// fn parser(input: &str) -> IResult<&str, &str> {
///     space1(input)
/// }
///
/// assert_eq!(parser(" \t21c"), Ok(("21c", " \t")));
/// assert_eq!(parser("H2"), Err(ErrMode::Backtrack(Error::new("H2", ErrorKind::Space))));
/// assert_eq!(parser(""), Err(ErrMode::Backtrack(Error::new("", ErrorKind::Space))));
/// ```
///
/// ```
/// # use winnow::{error::ErrMode, error::ErrorKind, error::Error, IResult, error::Needed};
/// # use winnow::Partial;
/// # use winnow::character::space1;
/// assert_eq!(space1::<_, Error<_>, true>(Partial(" \t21c")), Ok((Partial("21c"), " \t")));
/// assert_eq!(space1::<_, Error<_>, true>(Partial("H2")), Err(ErrMode::Backtrack(Error::new(Partial("H2"), ErrorKind::Space))));
/// assert_eq!(space1::<_, Error<_>, true>(Partial("")), Err(ErrMode::Incomplete(Needed::new(1))));
/// ```
#[inline(always)]
pub fn space1<I, E: ParseError<I>, const PARTIAL: bool>(
    input: I,
) -> IResult<I, <I as Input>::Slice, E>
where
    I: InputIsPartial<PARTIAL>,
    I: Input,
    <I as Input>::Token: AsChar,
{
    if PARTIAL {
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
/// *Partial version*: Will return `Err(winnow::error::ErrMode::Incomplete(_))` if there's not enough input data,
/// or if no terminating token is found (a non space character).
///
/// # Example
///
/// ```
/// # use winnow::{error::ErrMode, error::ErrorKind, error::Error, IResult, error::Needed};
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
/// # use winnow::{error::ErrMode, error::ErrorKind, error::Error, IResult, error::Needed};
/// # use winnow::Partial;
/// # use winnow::character::multispace0;
/// assert_eq!(multispace0::<_, Error<_>, true>(Partial(" \t\n\r21c")), Ok((Partial("21c"), " \t\n\r")));
/// assert_eq!(multispace0::<_, Error<_>, true>(Partial("Z21c")), Ok((Partial("Z21c"), "")));
/// assert_eq!(multispace0::<_, Error<_>, true>(Partial("")), Err(ErrMode::Incomplete(Needed::new(1))));
/// ```
#[inline(always)]
pub fn multispace0<I, E: ParseError<I>, const PARTIAL: bool>(
    input: I,
) -> IResult<I, <I as Input>::Slice, E>
where
    I: InputIsPartial<PARTIAL>,
    I: Input,
    <I as Input>::Token: AsChar,
{
    if PARTIAL {
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
/// *Partial version*: Will return `Err(winnow::error::ErrMode::Incomplete(_))` if there's not enough input data,
/// or if no terminating token is found (a non space character).
///
/// # Example
///
/// ```
/// # use winnow::{error::ErrMode, error::{Error, ErrorKind}, IResult, error::Needed};
/// # use winnow::character::multispace1;
/// fn parser(input: &str) -> IResult<&str, &str> {
///     multispace1(input)
/// }
///
/// assert_eq!(parser(" \t\n\r21c"), Ok(("21c", " \t\n\r")));
/// assert_eq!(parser("H2"), Err(ErrMode::Backtrack(Error::new("H2", ErrorKind::MultiSpace))));
/// assert_eq!(parser(""), Err(ErrMode::Backtrack(Error::new("", ErrorKind::MultiSpace))));
/// ```
///
/// ```
/// # use winnow::{error::ErrMode, error::ErrorKind, error::Error, IResult, error::Needed};
/// # use winnow::Partial;
/// # use winnow::character::multispace1;
/// assert_eq!(multispace1::<_, Error<_>, true>(Partial(" \t\n\r21c")), Ok((Partial("21c"), " \t\n\r")));
/// assert_eq!(multispace1::<_, Error<_>, true>(Partial("H2")), Err(ErrMode::Backtrack(Error::new(Partial("H2"), ErrorKind::MultiSpace))));
/// assert_eq!(multispace1::<_, Error<_>, true>(Partial("")), Err(ErrMode::Incomplete(Needed::new(1))));
/// ```
#[inline(always)]
pub fn multispace1<I, E: ParseError<I>, const PARTIAL: bool>(
    input: I,
) -> IResult<I, <I as Input>::Slice, E>
where
    I: InputIsPartial<PARTIAL>,
    I: Input,
    <I as Input>::Token: AsChar,
{
    if PARTIAL {
        streaming::multispace1(input)
    } else {
        complete::multispace1(input)
    }
}

/// Decode a decimal unsigned integer
///
/// *Complete version*: can parse until the end of input.
///
/// *Partial version*: Will return `Err(winnow::error::ErrMode::Incomplete(_))` if there's not enough input data.
pub fn dec_uint<I, O, E: ParseError<I>, const PARTIAL: bool>(input: I) -> IResult<I, O, E>
where
    I: InputIsPartial<PARTIAL>,
    I: Input,
    <I as Input>::Token: AsChar + Copy,
    O: Uint,
{
    let i = input.clone();

    if i.eof_offset() == 0 {
        if PARTIAL {
            return Err(ErrMode::Incomplete(Needed::new(1)));
        } else {
            return Err(ErrMode::from_error_kind(input, ErrorKind::Digit));
        }
    }

    let mut value = O::default();
    for (offset, c) in i.iter_offsets() {
        match c.as_char().to_digit(10) {
            Some(d) => match value.checked_mul(10, sealed::SealedMarker).and_then(|v| {
                let d = d as u8;
                v.checked_add(d, sealed::SealedMarker)
            }) {
                None => return Err(ErrMode::from_error_kind(input, ErrorKind::Digit)),
                Some(v) => value = v,
            },
            None => {
                if offset == 0 {
                    return Err(ErrMode::from_error_kind(input, ErrorKind::Digit));
                } else {
                    return Ok((i.next_slice(offset).0, value));
                }
            }
        }
    }

    if PARTIAL {
        Err(ErrMode::Incomplete(Needed::new(1)))
    } else {
        Ok((i.next_slice(i.eof_offset()).0, value))
    }
}

/// Metadata for parsing unsigned integers
pub trait Uint: Default {
    #[doc(hidden)]
    fn checked_mul(self, by: u8, _: sealed::SealedMarker) -> Option<Self>;
    #[doc(hidden)]
    fn checked_add(self, by: u8, _: sealed::SealedMarker) -> Option<Self>;
}

impl Uint for u8 {
    fn checked_mul(self, by: u8, _: sealed::SealedMarker) -> Option<Self> {
        self.checked_mul(by as Self)
    }
    fn checked_add(self, by: u8, _: sealed::SealedMarker) -> Option<Self> {
        self.checked_add(by as Self)
    }
}

impl Uint for u16 {
    fn checked_mul(self, by: u8, _: sealed::SealedMarker) -> Option<Self> {
        self.checked_mul(by as Self)
    }
    fn checked_add(self, by: u8, _: sealed::SealedMarker) -> Option<Self> {
        self.checked_add(by as Self)
    }
}

impl Uint for u32 {
    fn checked_mul(self, by: u8, _: sealed::SealedMarker) -> Option<Self> {
        self.checked_mul(by as Self)
    }
    fn checked_add(self, by: u8, _: sealed::SealedMarker) -> Option<Self> {
        self.checked_add(by as Self)
    }
}

impl Uint for u64 {
    fn checked_mul(self, by: u8, _: sealed::SealedMarker) -> Option<Self> {
        self.checked_mul(by as Self)
    }
    fn checked_add(self, by: u8, _: sealed::SealedMarker) -> Option<Self> {
        self.checked_add(by as Self)
    }
}

impl Uint for u128 {
    fn checked_mul(self, by: u8, _: sealed::SealedMarker) -> Option<Self> {
        self.checked_mul(by as Self)
    }
    fn checked_add(self, by: u8, _: sealed::SealedMarker) -> Option<Self> {
        self.checked_add(by as Self)
    }
}

impl Uint for i8 {
    fn checked_mul(self, by: u8, _: sealed::SealedMarker) -> Option<Self> {
        self.checked_mul(by as Self)
    }
    fn checked_add(self, by: u8, _: sealed::SealedMarker) -> Option<Self> {
        self.checked_add(by as Self)
    }
}

impl Uint for i16 {
    fn checked_mul(self, by: u8, _: sealed::SealedMarker) -> Option<Self> {
        self.checked_mul(by as Self)
    }
    fn checked_add(self, by: u8, _: sealed::SealedMarker) -> Option<Self> {
        self.checked_add(by as Self)
    }
}

impl Uint for i32 {
    fn checked_mul(self, by: u8, _: sealed::SealedMarker) -> Option<Self> {
        self.checked_mul(by as Self)
    }
    fn checked_add(self, by: u8, _: sealed::SealedMarker) -> Option<Self> {
        self.checked_add(by as Self)
    }
}

impl Uint for i64 {
    fn checked_mul(self, by: u8, _: sealed::SealedMarker) -> Option<Self> {
        self.checked_mul(by as Self)
    }
    fn checked_add(self, by: u8, _: sealed::SealedMarker) -> Option<Self> {
        self.checked_add(by as Self)
    }
}

impl Uint for i128 {
    fn checked_mul(self, by: u8, _: sealed::SealedMarker) -> Option<Self> {
        self.checked_mul(by as Self)
    }
    fn checked_add(self, by: u8, _: sealed::SealedMarker) -> Option<Self> {
        self.checked_add(by as Self)
    }
}

/// Decode a decimal signed integer
///
/// *Complete version*: can parse until the end of input.
///
/// *Partial version*: Will return `Err(winnow::error::ErrMode::Incomplete(_))` if there's not enough input data.
pub fn dec_int<I, O, E: ParseError<I>, const PARTIAL: bool>(input: I) -> IResult<I, O, E>
where
    I: InputIsPartial<PARTIAL>,
    I: Input,
    <I as Input>::Token: AsChar + Copy,
    O: Int,
{
    let i = input.clone();

    fn sign(token: impl AsChar) -> bool {
        let token = token.as_char();
        token == '+' || token == '-'
    }
    let (i, sign) = opt(crate::bytes::one_of(sign).map(AsChar::as_char))
        .map(|c| c != Some('-'))
        .parse_next(i)?;

    if i.eof_offset() == 0 {
        if PARTIAL {
            return Err(ErrMode::Incomplete(Needed::new(1)));
        } else {
            return Err(ErrMode::from_error_kind(input, ErrorKind::Digit));
        }
    }

    let mut value = O::default();
    for (offset, c) in i.iter_offsets() {
        match c.as_char().to_digit(10) {
            Some(d) => match value.checked_mul(10, sealed::SealedMarker).and_then(|v| {
                let d = d as u8;
                if sign {
                    v.checked_add(d, sealed::SealedMarker)
                } else {
                    v.checked_sub(d, sealed::SealedMarker)
                }
            }) {
                None => return Err(ErrMode::from_error_kind(input, ErrorKind::Digit)),
                Some(v) => value = v,
            },
            None => {
                if offset == 0 {
                    return Err(ErrMode::from_error_kind(input, ErrorKind::Digit));
                } else {
                    return Ok((i.next_slice(offset).0, value));
                }
            }
        }
    }

    if PARTIAL {
        Err(ErrMode::Incomplete(Needed::new(1)))
    } else {
        Ok((i.next_slice(i.eof_offset()).0, value))
    }
}

/// Metadata for parsing signed integers
pub trait Int: Uint {
    #[doc(hidden)]
    fn checked_sub(self, by: u8, _: sealed::SealedMarker) -> Option<Self>;
}

impl Int for i8 {
    fn checked_sub(self, by: u8, _: sealed::SealedMarker) -> Option<Self> {
        self.checked_sub(by as Self)
    }
}

impl Int for i16 {
    fn checked_sub(self, by: u8, _: sealed::SealedMarker) -> Option<Self> {
        self.checked_sub(by as Self)
    }
}

impl Int for i32 {
    fn checked_sub(self, by: u8, _: sealed::SealedMarker) -> Option<Self> {
        self.checked_sub(by as Self)
    }
}

impl Int for i64 {
    fn checked_sub(self, by: u8, _: sealed::SealedMarker) -> Option<Self> {
        self.checked_sub(by as Self)
    }
}

impl Int for i128 {
    fn checked_sub(self, by: u8, _: sealed::SealedMarker) -> Option<Self> {
        self.checked_sub(by as Self)
    }
}

/// Decode a variable-width hexadecimal integer.
///
/// *Complete version*: Will parse until the end of input if it has fewer characters than the type
/// supports.
///
/// *Partial version*: Will return `Err(winnow::error::ErrMode::Incomplete(_))` if end-of-input
/// is hit before a hard boundary (non-hex character, more characters than supported).
///
/// # Example
///
/// ```rust
/// # use winnow::prelude::*;
/// # use winnow::{error::ErrMode, error::ErrorKind, error::Error};
/// use winnow::character::hex_uint;
///
/// fn parser(s: &[u8]) -> IResult<&[u8], u32> {
///   hex_uint(s)
/// }
///
/// assert_eq!(parser(&b"01AE"[..]), Ok((&b""[..], 0x01AE)));
/// assert_eq!(parser(&b"abc"[..]), Ok((&b""[..], 0x0ABC)));
/// assert_eq!(parser(&b"ggg"[..]), Err(ErrMode::Backtrack(Error::new(&b"ggg"[..], ErrorKind::IsA))));
/// ```
///
/// ```rust
/// # use winnow::prelude::*;
/// # use winnow::{error::ErrMode, error::ErrorKind, error::Error, error::Needed};
/// # use winnow::Partial;
/// use winnow::character::hex_uint;
///
/// fn parser(s: Partial<&[u8]>) -> IResult<Partial<&[u8]>, u32> {
///   hex_uint(s)
/// }
///
/// assert_eq!(parser(Partial(&b"01AE;"[..])), Ok((Partial(&b";"[..]), 0x01AE)));
/// assert_eq!(parser(Partial(&b"abc"[..])), Err(ErrMode::Incomplete(Needed::new(1))));
/// assert_eq!(parser(Partial(&b"ggg"[..])), Err(ErrMode::Backtrack(Error::new(Partial(&b"ggg"[..]), ErrorKind::IsA))));
/// ```
#[inline]
pub fn hex_uint<I, O, E: ParseError<I>, const PARTIAL: bool>(input: I) -> IResult<I, O, E>
where
    I: InputIsPartial<PARTIAL>,
    I: Input,
    O: HexUint,
    <I as Input>::Token: AsChar,
    <I as Input>::Slice: AsBStr,
{
    let invalid_offset = input
        .offset_for(|c| {
            let c = c.as_char();
            !"0123456789abcdefABCDEF".contains(c)
        })
        .unwrap_or_else(|| input.eof_offset());
    let max_nibbles = O::max_nibbles(sealed::SealedMarker);
    let max_offset = input.offset_at(max_nibbles);
    let offset = match max_offset {
        Ok(max_offset) => {
            if max_offset < invalid_offset {
                // Overflow
                return Err(ErrMode::from_error_kind(input, ErrorKind::IsA));
            } else {
                invalid_offset
            }
        }
        Err(_) => {
            if PARTIAL && invalid_offset == input.eof_offset() {
                // Only the next byte is guaranteed required
                return Err(ErrMode::Incomplete(Needed::new(1)));
            } else {
                invalid_offset
            }
        }
    };
    if offset == 0 {
        // Must be at least one digit
        return Err(ErrMode::from_error_kind(input, ErrorKind::IsA));
    }
    let (remaining, parsed) = input.next_slice(offset);

    let mut res = O::default();
    for c in parsed.as_bstr() {
        let nibble = *c as char;
        let nibble = nibble.to_digit(16).unwrap_or(0) as u8;
        let nibble = O::from(nibble);
        res = (res << O::from(4)) + nibble;
    }

    Ok((remaining, res))
}

/// Metadata for parsing hex numbers
pub trait HexUint:
    Default + Shl<Self, Output = Self> + Add<Self, Output = Self> + From<u8>
{
    #[doc(hidden)]
    fn max_nibbles(_: sealed::SealedMarker) -> usize;
}

impl HexUint for u8 {
    #[inline(always)]
    fn max_nibbles(_: sealed::SealedMarker) -> usize {
        2
    }
}

impl HexUint for u16 {
    #[inline(always)]
    fn max_nibbles(_: sealed::SealedMarker) -> usize {
        4
    }
}

impl HexUint for u32 {
    #[inline(always)]
    fn max_nibbles(_: sealed::SealedMarker) -> usize {
        8
    }
}

impl HexUint for u64 {
    #[inline(always)]
    fn max_nibbles(_: sealed::SealedMarker) -> usize {
        16
    }
}

impl HexUint for u128 {
    #[inline(always)]
    fn max_nibbles(_: sealed::SealedMarker) -> usize {
        32
    }
}

/// Recognizes floating point number in text format and returns a f32.
///
/// *Complete version*: Can parse until the end of input.
///
/// *Partial version*: Will return `Err(winnow::error::ErrMode::Incomplete(_))` if there is not enough data.
///
/// # Example
///
/// ```rust
/// # use winnow::prelude::*;
/// # use winnow::{error::ErrMode, error::ErrorKind, error::Error, error::Needed};
/// # use winnow::error::Needed::Size;
/// use winnow::character::float;
///
/// fn parser(s: &str) -> IResult<&str, f64> {
///   float(s)
/// }
///
/// assert_eq!(parser("11e-1"), Ok(("", 1.1)));
/// assert_eq!(parser("123E-02"), Ok(("", 1.23)));
/// assert_eq!(parser("123K-01"), Ok(("K-01", 123.0)));
/// assert_eq!(parser("abc"), Err(ErrMode::Backtrack(Error::new("abc", ErrorKind::Float))));
/// ```
///
/// ```rust
/// # use winnow::prelude::*;
/// # use winnow::{error::ErrMode, error::ErrorKind, error::Error, error::Needed};
/// # use winnow::error::Needed::Size;
/// # use winnow::Partial;
/// use winnow::character::float;
///
/// fn parser(s: Partial<&str>) -> IResult<Partial<&str>, f64> {
///   float(s)
/// }
///
/// assert_eq!(parser(Partial("11e-1 ")), Ok((Partial(" "), 1.1)));
/// assert_eq!(parser(Partial("11e-1")), Err(ErrMode::Incomplete(Needed::new(1))));
/// assert_eq!(parser(Partial("123E-02")), Err(ErrMode::Incomplete(Needed::new(1))));
/// assert_eq!(parser(Partial("123K-01")), Ok((Partial("K-01"), 123.0)));
/// assert_eq!(parser(Partial("abc")), Err(ErrMode::Backtrack(Error::new(Partial("abc"), ErrorKind::Float))));
/// ```
#[inline(always)]
pub fn float<I, O, E: ParseError<I>, const PARTIAL: bool>(input: I) -> IResult<I, O, E>
where
    I: InputIsPartial<PARTIAL>,
    I: Input,
    I: Offset + Compare<&'static str>,
    <I as Input>::Slice: ParseSlice<O>,
    <I as Input>::Token: AsChar + Copy,
    <I as Input>::IterOffsets: Clone,
    I: AsBStr,
{
    let (i, s) = if PARTIAL {
        crate::number::streaming::recognize_float_or_exceptions(input)?
    } else {
        crate::number::complete::recognize_float_or_exceptions(input)?
    };
    match s.parse_slice() {
        Some(f) => Ok((i, f)),
        None => Err(ErrMode::from_error_kind(i, ErrorKind::Float)),
    }
}

/// Matches a byte string with escaped characters.
///
/// * The first argument matches the normal characters (it must not accept the control character)
/// * The second argument is the control character (like `\` in most languages)
/// * The third argument matches the escaped characters
/// # Example
/// ```
/// # use winnow::{error::ErrMode, error::ErrorKind, error::Error, error::Needed, IResult};
/// # use winnow::character::digit1;
/// use winnow::character::escaped;
/// use winnow::bytes::one_of;
///
/// fn esc(s: &str) -> IResult<&str, &str> {
///   escaped(digit1, '\\', one_of(r#""n\"#))(s)
/// }
///
/// assert_eq!(esc("123;"), Ok((";", "123")));
/// assert_eq!(esc(r#"12\"34;"#), Ok((";", r#"12\"34"#)));
/// ```
///
/// ```
/// # use winnow::{error::ErrMode, error::ErrorKind, error::Error, error::Needed, IResult};
/// # use winnow::character::digit1;
/// # use winnow::Partial;
/// use winnow::character::escaped;
/// use winnow::bytes::one_of;
///
/// fn esc(s: Partial<&str>) -> IResult<Partial<&str>, &str> {
///   escaped(digit1, '\\', one_of("\"n\\"))(s)
/// }
///
/// assert_eq!(esc(Partial("123;")), Ok((Partial(";"), "123")));
/// assert_eq!(esc(Partial("12\\\"34;")), Ok((Partial(";"), "12\\\"34")));
/// ```
#[inline(always)]
pub fn escaped<'a, I: 'a, Error, F, G, O1, O2, const PARTIAL: bool>(
    mut normal: F,
    control_char: char,
    mut escapable: G,
) -> impl FnMut(I) -> IResult<I, <I as Input>::Slice, Error>
where
    I: InputIsPartial<PARTIAL>,
    I: Input + Offset,
    <I as Input>::Token: crate::stream::AsChar,
    F: Parser<I, O1, Error>,
    G: Parser<I, O2, Error>,
    Error: ParseError<I>,
{
    move |input: I| {
        if PARTIAL {
            crate::bytes::streaming::escaped_internal(
                input,
                &mut normal,
                control_char,
                &mut escapable,
            )
        } else {
            crate::bytes::complete::escaped_internal(
                input,
                &mut normal,
                control_char,
                &mut escapable,
            )
        }
    }
}

/// Matches a byte string with escaped characters.
///
/// * The first argument matches the normal characters (it must not match the control character)
/// * The second argument is the control character (like `\` in most languages)
/// * The third argument matches the escaped characters and transforms them
///
/// As an example, the chain `abc\tdef` could be `abc    def` (it also consumes the control character)
///
/// ```
/// # use winnow::prelude::*;
/// # use winnow::{error::ErrMode, error::ErrorKind, error::Error, error::Needed};
/// # use std::str::from_utf8;
/// use winnow::bytes::tag;
/// use winnow::character::escaped_transform;
/// use winnow::character::alpha1;
/// use winnow::branch::alt;
/// use winnow::combinator::value;
///
/// fn parser(input: &str) -> IResult<&str, String> {
///   escaped_transform(
///     alpha1,
///     '\\',
///     alt((
///       tag("\\").value("\\"),
///       tag("\"").value("\""),
///       tag("n").value("\n"),
///     ))
///   )(input)
/// }
///
/// assert_eq!(parser("ab\\\"cd"), Ok(("", String::from("ab\"cd"))));
/// assert_eq!(parser("ab\\ncd"), Ok(("", String::from("ab\ncd"))));
/// ```
///
/// ```
/// # use winnow::prelude::*;
/// # use winnow::{error::ErrMode, error::ErrorKind, error::Error, error::Needed};
/// # use std::str::from_utf8;
/// # use winnow::Partial;
/// use winnow::bytes::tag;
/// use winnow::character::escaped_transform;
/// use winnow::character::alpha1;
/// use winnow::branch::alt;
/// use winnow::combinator::value;
///
/// fn parser(input: Partial<&str>) -> IResult<Partial<&str>, String> {
///   escaped_transform(
///     alpha1,
///     '\\',
///     alt((
///       tag("\\").value("\\"),
///       tag("\"").value("\""),
///       tag("n").value("\n"),
///     ))
///   )(input)
/// }
///
/// assert_eq!(parser(Partial("ab\\\"cd\"")), Ok((Partial("\""), String::from("ab\"cd"))));
/// ```
#[cfg(feature = "alloc")]
#[inline(always)]
pub fn escaped_transform<I, Error, F, G, Output, const PARTIAL: bool>(
    mut normal: F,
    control_char: char,
    mut transform: G,
) -> impl FnMut(I) -> IResult<I, Output, Error>
where
    I: InputIsPartial<PARTIAL>,
    I: Input + Offset,
    <I as Input>::Token: crate::stream::AsChar,
    Output: crate::stream::Accumulate<<I as Input>::Slice>,
    F: Parser<I, <I as Input>::Slice, Error>,
    G: Parser<I, <I as Input>::Slice, Error>,
    Error: ParseError<I>,
{
    move |input: I| {
        if PARTIAL {
            crate::bytes::streaming::escaped_transform_internal(
                input,
                &mut normal,
                control_char,
                &mut transform,
            )
        } else {
            crate::bytes::complete::escaped_transform_internal(
                input,
                &mut normal,
                control_char,
                &mut transform,
            )
        }
    }
}

#[inline]
#[doc(hidden)]
#[deprecated(since = "0.1.0", note = "Replaced with `AsChar::is_alpha`")]
pub fn is_alphabetic(chr: u8) -> bool {
    matches!(chr, 0x41..=0x5A | 0x61..=0x7A)
}

#[inline]
#[doc(hidden)]
#[deprecated(since = "0.1.0", note = "Replaced with `AsChar::is_dec_digit`")]
pub fn is_digit(chr: u8) -> bool {
    matches!(chr, 0x30..=0x39)
}

#[inline]
#[doc(hidden)]
#[deprecated(since = "0.1.0", note = "Replaced with `AsChar::is_hex_digit`")]
pub fn is_hex_digit(chr: u8) -> bool {
    matches!(chr, 0x30..=0x39 | 0x41..=0x46 | 0x61..=0x66)
}

#[inline]
#[doc(hidden)]
#[deprecated(since = "0.1.0", note = "Replaced with `AsChar::is_oct_digit`")]
pub fn is_oct_digit(chr: u8) -> bool {
    matches!(chr, 0x30..=0x37)
}

#[inline]
#[doc(hidden)]
#[deprecated(since = "0.1.0", note = "Replaced with `AsChar::is_alphanum`")]
pub fn is_alphanumeric(chr: u8) -> bool {
    #![allow(deprecated)]
    is_alphabetic(chr) || is_digit(chr)
}

#[inline]
#[doc(hidden)]
#[deprecated(since = "0.1.0", note = "Replaced with `AsChar::is_space`")]
pub fn is_space(chr: u8) -> bool {
    chr == b' ' || chr == b'\t'
}

#[inline]
#[doc(hidden)]
#[deprecated(since = "0.1.0", note = "Replaced with `AsChar::is_newline`")]
pub fn is_newline(chr: u8) -> bool {
    chr == b'\n'
}

mod sealed {
    pub struct SealedMarker;
}
