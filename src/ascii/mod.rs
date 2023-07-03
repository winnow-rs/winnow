//! Character specific parsers and combinators
//!
//! Functions recognizing specific characters

#[cfg(test)]
mod tests;

use crate::lib::std::ops::{Add, Shl};

use crate::combinator::alt;
use crate::token::one_of;

use crate::combinator::cut_err;
use crate::combinator::opt;
use crate::error::ParseError;
use crate::error::{ErrMode, ErrorKind, Needed};
use crate::stream::{AsBStr, AsChar, ParseSlice, Stream, StreamIsPartial};
use crate::stream::{Compare, CompareResult};
use crate::token::take_while;
use crate::trace::trace;
use crate::PResult;
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
/// # use winnow::prelude::*;
/// # use winnow::{error::ErrMode, error::{Error, ErrorKind}};
/// # use winnow::ascii::crlf;
/// fn parser<'s>(input: &mut &'s str) -> PResult<&'s str, Error<&'s str>> {
///     crlf.parse_next(input)
/// }
///
/// assert_eq!(parser.parse_peek("\r\nc"), Ok(("c", "\r\n")));
/// assert_eq!(parser.parse_peek("ab\r\nc"), Err(ErrMode::Backtrack(Error::new("ab\r\nc", ErrorKind::Tag))));
/// assert_eq!(parser.parse_peek(""), Err(ErrMode::Backtrack(Error::new("", ErrorKind::Tag))));
/// ```
///
/// ```
/// # use winnow::prelude::*;
/// # use winnow::{error::ErrMode, error::ErrorKind, error::Error, error::Needed};
/// # use winnow::Partial;
/// # use winnow::ascii::crlf;
/// assert_eq!(crlf::<_, Error<_>>.parse_peek(Partial::new("\r\nc")), Ok((Partial::new("c"), "\r\n")));
/// assert_eq!(crlf::<_, Error<_>>.parse_peek(Partial::new("ab\r\nc")), Err(ErrMode::Backtrack(Error::new(Partial::new("ab\r\nc"), ErrorKind::Tag))));
/// assert_eq!(crlf::<_, Error<_>>.parse_peek(Partial::new("")), Err(ErrMode::Incomplete(Needed::new(2))));
/// ```
#[inline(always)]
pub fn crlf<I, E: ParseError<I>>(input: &mut I) -> PResult<<I as Stream>::Slice, E>
where
    I: StreamIsPartial,
    I: Stream,
    I: Compare<&'static str>,
{
    trace("crlf", "\r\n").parse_next(input)
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
/// # use winnow::prelude::*;
/// # use winnow::{error::ErrMode, error::{Error, ErrorKind}, error::Needed};
/// # use winnow::ascii::not_line_ending;
/// fn parser<'s>(input: &mut &'s str) -> PResult<&'s str, Error<&'s str>> {
///     not_line_ending.parse_next(input)
/// }
///
/// assert_eq!(parser.parse_peek("ab\r\nc"), Ok(("\r\nc", "ab")));
/// assert_eq!(parser.parse_peek("ab\nc"), Ok(("\nc", "ab")));
/// assert_eq!(parser.parse_peek("abc"), Ok(("", "abc")));
/// assert_eq!(parser.parse_peek(""), Ok(("", "")));
/// assert_eq!(parser.parse_peek("a\rb\nc"), Err(ErrMode::Backtrack(Error { input: "\rb\nc", kind: ErrorKind::Tag })));
/// assert_eq!(parser.parse_peek("a\rbc"), Err(ErrMode::Backtrack(Error { input: "\rbc", kind: ErrorKind::Tag })));
/// ```
///
/// ```
/// # use winnow::prelude::*;
/// # use winnow::{error::ErrMode, error::{Error, ErrorKind}, error::Needed};
/// # use winnow::Partial;
/// # use winnow::ascii::not_line_ending;
/// assert_eq!(not_line_ending::<_, Error<_>>.parse_peek(Partial::new("ab\r\nc")), Ok((Partial::new("\r\nc"), "ab")));
/// assert_eq!(not_line_ending::<_, Error<_>>.parse_peek(Partial::new("abc")), Err(ErrMode::Incomplete(Needed::Unknown)));
/// assert_eq!(not_line_ending::<_, Error<_>>.parse_peek(Partial::new("")), Err(ErrMode::Incomplete(Needed::Unknown)));
/// assert_eq!(not_line_ending::<_, Error<_>>.parse_peek(Partial::new("a\rb\nc")), Err(ErrMode::Backtrack(Error::new(Partial::new("\rb\nc"), ErrorKind::Tag ))));
/// assert_eq!(not_line_ending::<_, Error<_>>.parse_peek(Partial::new("a\rbc")), Err(ErrMode::Backtrack(Error::new(Partial::new("\rbc"), ErrorKind::Tag ))));
/// ```
#[inline(always)]
pub fn not_line_ending<I, E: ParseError<I>>(input: &mut I) -> PResult<<I as Stream>::Slice, E>
where
    I: StreamIsPartial,
    I: Stream + AsBStr,
    I: Compare<&'static str>,
    <I as Stream>::Token: AsChar,
{
    trace("not_line_ending", move |input: &mut I| {
        if <I as StreamIsPartial>::is_partial_supported() {
            not_line_ending_::<_, _, true>(input)
        } else {
            not_line_ending_::<_, _, false>(input)
        }
    })
    .parse_next(input)
}

fn not_line_ending_<I, E: ParseError<I>, const PARTIAL: bool>(
    input: &mut I,
) -> PResult<<I as Stream>::Slice, E>
where
    I: StreamIsPartial,
    I: Stream + AsBStr,
    I: Compare<&'static str>,
    <I as Stream>::Token: AsChar,
{
    match input.offset_for(|item| {
        let c = item.as_char();
        c == '\r' || c == '\n'
    }) {
        None if PARTIAL && input.is_partial() => Err(ErrMode::Incomplete(Needed::Unknown)),
        None => Ok(input.finish()),
        Some(offset) => {
            let res = input.next_slice(offset);
            let bytes = input.as_bstr();
            let nth = bytes[0];
            if nth == b'\r' {
                let comp = input.compare("\r\n");
                match comp {
                    //FIXME: calculate the right index
                    CompareResult::Ok => {}
                    CompareResult::Incomplete if PARTIAL && input.is_partial() => {
                        return Err(ErrMode::Incomplete(Needed::Unknown));
                    }
                    CompareResult::Incomplete | CompareResult::Error => {
                        let e: ErrorKind = ErrorKind::Tag;
                        return Err(ErrMode::from_error_kind(input.clone(), e));
                    }
                }
            }
            Ok(res)
        }
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
/// # use winnow::prelude::*;
/// # use winnow::{error::ErrMode, error::{Error, ErrorKind}, error::Needed};
/// # use winnow::ascii::line_ending;
/// fn parser<'s>(input: &mut &'s str) -> PResult<&'s str, Error<&'s str>> {
///     line_ending.parse_next(input)
/// }
///
/// assert_eq!(parser.parse_peek("\r\nc"), Ok(("c", "\r\n")));
/// assert_eq!(parser.parse_peek("ab\r\nc"), Err(ErrMode::Backtrack(Error::new("ab\r\nc", ErrorKind::Tag))));
/// assert_eq!(parser.parse_peek(""), Err(ErrMode::Backtrack(Error::new("", ErrorKind::Tag))));
/// ```
///
/// ```
/// # use winnow::prelude::*;
/// # use winnow::{error::ErrMode, error::ErrorKind, error::Error, error::Needed};
/// # use winnow::Partial;
/// # use winnow::ascii::line_ending;
/// assert_eq!(line_ending::<_, Error<_>>.parse_peek(Partial::new("\r\nc")), Ok((Partial::new("c"), "\r\n")));
/// assert_eq!(line_ending::<_, Error<_>>.parse_peek(Partial::new("ab\r\nc")), Err(ErrMode::Backtrack(Error::new(Partial::new("ab\r\nc"), ErrorKind::Tag))));
/// assert_eq!(line_ending::<_, Error<_>>.parse_peek(Partial::new("")), Err(ErrMode::Incomplete(Needed::new(1))));
/// ```
#[inline(always)]
pub fn line_ending<I, E: ParseError<I>>(input: &mut I) -> PResult<<I as Stream>::Slice, E>
where
    I: StreamIsPartial,
    I: Stream,
    I: Compare<&'static str>,
{
    trace("line_ending", alt(("\n", "\r\n"))).parse_next(input)
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
/// # use winnow::prelude::*;
/// # use winnow::{error::ErrMode, error::{Error, ErrorKind}, error::Needed};
/// # use winnow::ascii::newline;
/// fn parser<'s>(input: &mut &'s str) -> PResult<char, Error<&'s str>> {
///     newline.parse_next(input)
/// }
///
/// assert_eq!(parser.parse_peek("\nc"), Ok(("c", '\n')));
/// assert_eq!(parser.parse_peek("\r\nc"), Err(ErrMode::Backtrack(Error::new("\r\nc", ErrorKind::Verify))));
/// assert_eq!(parser.parse_peek(""), Err(ErrMode::Backtrack(Error::new("", ErrorKind::Token))));
/// ```
///
/// ```
/// # use winnow::prelude::*;
/// # use winnow::{error::ErrMode, error::ErrorKind, error::Error, error::Needed};
/// # use winnow::Partial;
/// # use winnow::ascii::newline;
/// assert_eq!(newline::<_, Error<_>>.parse_peek(Partial::new("\nc")), Ok((Partial::new("c"), '\n')));
/// assert_eq!(newline::<_, Error<_>>.parse_peek(Partial::new("\r\nc")), Err(ErrMode::Backtrack(Error::new(Partial::new("\r\nc"), ErrorKind::Verify))));
/// assert_eq!(newline::<_, Error<_>>.parse_peek(Partial::new("")), Err(ErrMode::Incomplete(Needed::new(1))));
/// ```
#[inline(always)]
pub fn newline<I, Error: ParseError<I>>(input: &mut I) -> PResult<char, Error>
where
    I: StreamIsPartial,
    I: Stream,
    <I as Stream>::Token: AsChar + Copy,
{
    trace("newline", '\n'.map(|c: <I as Stream>::Token| c.as_char())).parse_next(input)
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
/// # use winnow::prelude::*;
/// # use winnow::{error::ErrMode, error::{Error, ErrorKind}, error::Needed};
/// # use winnow::ascii::tab;
/// fn parser<'s>(input: &mut &'s str) -> PResult<char, Error<&'s str>> {
///     tab.parse_next(input)
/// }
///
/// assert_eq!(parser.parse_peek("\tc"), Ok(("c", '\t')));
/// assert_eq!(parser.parse_peek("\r\nc"), Err(ErrMode::Backtrack(Error::new("\r\nc", ErrorKind::Verify))));
/// assert_eq!(parser.parse_peek(""), Err(ErrMode::Backtrack(Error::new("", ErrorKind::Token))));
/// ```
///
/// ```
/// # use winnow::prelude::*;
/// # use winnow::{error::ErrMode, error::ErrorKind, error::Error, error::Needed};
/// # use winnow::Partial;
/// # use winnow::ascii::tab;
/// assert_eq!(tab::<_, Error<_>>.parse_peek(Partial::new("\tc")), Ok((Partial::new("c"), '\t')));
/// assert_eq!(tab::<_, Error<_>>.parse_peek(Partial::new("\r\nc")), Err(ErrMode::Backtrack(Error::new(Partial::new("\r\nc"), ErrorKind::Verify))));
/// assert_eq!(tab::<_, Error<_>>.parse_peek(Partial::new("")), Err(ErrMode::Incomplete(Needed::new(1))));
/// ```
#[inline(always)]
pub fn tab<I, Error: ParseError<I>>(input: &mut I) -> PResult<char, Error>
where
    I: StreamIsPartial,
    I: Stream,
    <I as Stream>::Token: AsChar + Copy,
{
    trace("tab", '\t'.map(|c: <I as Stream>::Token| c.as_char())).parse_next(input)
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
/// # use winnow::prelude::*;
/// # use winnow::{error::ErrMode, error::ErrorKind, error::Error, error::Needed};
/// # use winnow::ascii::alpha0;
/// fn parser<'s>(input: &mut &'s str) -> PResult<&'s str, Error<&'s str>> {
///     alpha0.parse_next(input)
/// }
///
/// assert_eq!(parser.parse_peek("ab1c"), Ok(("1c", "ab")));
/// assert_eq!(parser.parse_peek("1c"), Ok(("1c", "")));
/// assert_eq!(parser.parse_peek(""), Ok(("", "")));
/// ```
///
/// ```
/// # use winnow::prelude::*;
/// # use winnow::{error::ErrMode, error::ErrorKind, error::Error, error::Needed};
/// # use winnow::Partial;
/// # use winnow::ascii::alpha0;
/// assert_eq!(alpha0::<_, Error<_>>.parse_peek(Partial::new("ab1c")), Ok((Partial::new("1c"), "ab")));
/// assert_eq!(alpha0::<_, Error<_>>.parse_peek(Partial::new("1c")), Ok((Partial::new("1c"), "")));
/// assert_eq!(alpha0::<_, Error<_>>.parse_peek(Partial::new("")), Err(ErrMode::Incomplete(Needed::new(1))));
/// ```
#[inline(always)]
pub fn alpha0<I, E: ParseError<I>>(input: &mut I) -> PResult<<I as Stream>::Slice, E>
where
    I: StreamIsPartial,
    I: Stream,
    <I as Stream>::Token: AsChar,
{
    trace(
        "alpha0",
        take_while(0.., |c: <I as Stream>::Token| c.is_alpha()),
    )
    .parse_next(input)
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
/// # use winnow::prelude::*;
/// # use winnow::{error::ErrMode, error::{Error, ErrorKind}, error::Needed};
/// # use winnow::ascii::alpha1;
/// fn parser<'s>(input: &mut &'s str) -> PResult<&'s str, Error<&'s str>> {
///     alpha1.parse_next(input)
/// }
///
/// assert_eq!(parser.parse_peek("aB1c"), Ok(("1c", "aB")));
/// assert_eq!(parser.parse_peek("1c"), Err(ErrMode::Backtrack(Error::new("1c", ErrorKind::Slice))));
/// assert_eq!(parser.parse_peek(""), Err(ErrMode::Backtrack(Error::new("", ErrorKind::Slice))));
/// ```
///
/// ```
/// # use winnow::prelude::*;
/// # use winnow::{error::ErrMode, error::ErrorKind, error::Error, error::Needed};
/// # use winnow::Partial;
/// # use winnow::ascii::alpha1;
/// assert_eq!(alpha1::<_, Error<_>>.parse_peek(Partial::new("aB1c")), Ok((Partial::new("1c"), "aB")));
/// assert_eq!(alpha1::<_, Error<_>>.parse_peek(Partial::new("1c")), Err(ErrMode::Backtrack(Error::new(Partial::new("1c"), ErrorKind::Slice))));
/// assert_eq!(alpha1::<_, Error<_>>.parse_peek(Partial::new("")), Err(ErrMode::Incomplete(Needed::new(1))));
/// ```
#[inline(always)]
pub fn alpha1<I, E: ParseError<I>>(input: &mut I) -> PResult<<I as Stream>::Slice, E>
where
    I: StreamIsPartial,
    I: Stream,
    <I as Stream>::Token: AsChar,
{
    trace(
        "alpha1",
        take_while(1.., |c: <I as Stream>::Token| c.is_alpha()),
    )
    .parse_next(input)
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
/// # use winnow::prelude::*;
/// # use winnow::{error::ErrMode, error::ErrorKind, error::Error, error::Needed};
/// # use winnow::ascii::digit0;
/// fn parser<'s>(input: &mut &'s str) -> PResult<&'s str, Error<&'s str>> {
///     digit0.parse_next(input)
/// }
///
/// assert_eq!(parser.parse_peek("21c"), Ok(("c", "21")));
/// assert_eq!(parser.parse_peek("21"), Ok(("", "21")));
/// assert_eq!(parser.parse_peek("a21c"), Ok(("a21c", "")));
/// assert_eq!(parser.parse_peek(""), Ok(("", "")));
/// ```
///
/// ```
/// # use winnow::prelude::*;
/// # use winnow::{error::ErrMode, error::ErrorKind, error::Error, error::Needed};
/// # use winnow::Partial;
/// # use winnow::ascii::digit0;
/// assert_eq!(digit0::<_, Error<_>>.parse_peek(Partial::new("21c")), Ok((Partial::new("c"), "21")));
/// assert_eq!(digit0::<_, Error<_>>.parse_peek(Partial::new("a21c")), Ok((Partial::new("a21c"), "")));
/// assert_eq!(digit0::<_, Error<_>>.parse_peek(Partial::new("")), Err(ErrMode::Incomplete(Needed::new(1))));
/// ```
#[inline(always)]
pub fn digit0<I, E: ParseError<I>>(input: &mut I) -> PResult<<I as Stream>::Slice, E>
where
    I: StreamIsPartial,
    I: Stream,
    <I as Stream>::Token: AsChar,
{
    trace(
        "digit0",
        take_while(0.., |c: <I as Stream>::Token| c.is_dec_digit()),
    )
    .parse_next(input)
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
/// # use winnow::prelude::*;
/// # use winnow::{error::ErrMode, error::{Error, ErrorKind}, error::Needed};
/// # use winnow::ascii::digit1;
/// fn parser<'s>(input: &mut &'s str) -> PResult<&'s str, Error<&'s str>> {
///     digit1.parse_next(input)
/// }
///
/// assert_eq!(parser.parse_peek("21c"), Ok(("c", "21")));
/// assert_eq!(parser.parse_peek("c1"), Err(ErrMode::Backtrack(Error::new("c1", ErrorKind::Slice))));
/// assert_eq!(parser.parse_peek(""), Err(ErrMode::Backtrack(Error::new("", ErrorKind::Slice))));
/// ```
///
/// ```
/// # use winnow::prelude::*;
/// # use winnow::{error::ErrMode, error::ErrorKind, error::Error, error::Needed};
/// # use winnow::Partial;
/// # use winnow::ascii::digit1;
/// assert_eq!(digit1::<_, Error<_>>.parse_peek(Partial::new("21c")), Ok((Partial::new("c"), "21")));
/// assert_eq!(digit1::<_, Error<_>>.parse_peek(Partial::new("c1")), Err(ErrMode::Backtrack(Error::new(Partial::new("c1"), ErrorKind::Slice))));
/// assert_eq!(digit1::<_, Error<_>>.parse_peek(Partial::new("")), Err(ErrMode::Incomplete(Needed::new(1))));
/// ```
///
/// ## Parsing an integer
///
/// You can use `digit1` in combination with [`Parser::try_map`][crate::Parser::try_map] to parse an integer:
///
/// ```
/// # use winnow::prelude::*;
/// # use winnow::{error::ErrMode, error::{Error, ErrorKind}, error::Needed, Parser};
/// # use winnow::ascii::digit1;
/// fn parser<'s>(input: &mut &'s str) -> PResult<u32, Error<&'s str>> {
///   digit1.try_map(str::parse).parse_next(input)
/// }
///
/// assert_eq!(parser.parse_peek("416"), Ok(("", 416)));
/// assert_eq!(parser.parse_peek("12b"), Ok(("b", 12)));
/// assert!(parser.parse_peek("b").is_err());
/// ```
#[inline(always)]
pub fn digit1<I, E: ParseError<I>>(input: &mut I) -> PResult<<I as Stream>::Slice, E>
where
    I: StreamIsPartial,
    I: Stream,
    <I as Stream>::Token: AsChar,
{
    trace(
        "digit1",
        take_while(1.., |c: <I as Stream>::Token| c.is_dec_digit()),
    )
    .parse_next(input)
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
/// # use winnow::prelude::*;
/// # use winnow::{error::ErrMode, error::ErrorKind, error::Error, error::Needed};
/// # use winnow::ascii::hex_digit0;
/// fn parser<'s>(input: &mut &'s str) -> PResult<&'s str, Error<&'s str>> {
///     hex_digit0.parse_next(input)
/// }
///
/// assert_eq!(parser.parse_peek("21cZ"), Ok(("Z", "21c")));
/// assert_eq!(parser.parse_peek("Z21c"), Ok(("Z21c", "")));
/// assert_eq!(parser.parse_peek(""), Ok(("", "")));
/// ```
///
/// ```
/// # use winnow::prelude::*;
/// # use winnow::{error::ErrMode, error::ErrorKind, error::Error, error::Needed};
/// # use winnow::Partial;
/// # use winnow::ascii::hex_digit0;
/// assert_eq!(hex_digit0::<_, Error<_>>.parse_peek(Partial::new("21cZ")), Ok((Partial::new("Z"), "21c")));
/// assert_eq!(hex_digit0::<_, Error<_>>.parse_peek(Partial::new("Z21c")), Ok((Partial::new("Z21c"), "")));
/// assert_eq!(hex_digit0::<_, Error<_>>.parse_peek(Partial::new("")), Err(ErrMode::Incomplete(Needed::new(1))));
/// ```
#[inline(always)]
pub fn hex_digit0<I, E: ParseError<I>>(input: &mut I) -> PResult<<I as Stream>::Slice, E>
where
    I: StreamIsPartial,
    I: Stream,
    <I as Stream>::Token: AsChar,
{
    trace(
        "hex_digit0",
        take_while(0.., |c: <I as Stream>::Token| c.is_hex_digit()),
    )
    .parse_next(input)
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
/// # use winnow::prelude::*;
/// # use winnow::{error::ErrMode, error::{Error, ErrorKind}, error::Needed};
/// # use winnow::ascii::hex_digit1;
/// fn parser<'s>(input: &mut &'s str) -> PResult<&'s str, Error<&'s str>> {
///     hex_digit1.parse_next(input)
/// }
///
/// assert_eq!(parser.parse_peek("21cZ"), Ok(("Z", "21c")));
/// assert_eq!(parser.parse_peek("H2"), Err(ErrMode::Backtrack(Error::new("H2", ErrorKind::Slice))));
/// assert_eq!(parser.parse_peek(""), Err(ErrMode::Backtrack(Error::new("", ErrorKind::Slice))));
/// ```
///
/// ```
/// # use winnow::prelude::*;
/// # use winnow::{error::ErrMode, error::ErrorKind, error::Error, error::Needed};
/// # use winnow::Partial;
/// # use winnow::ascii::hex_digit1;
/// assert_eq!(hex_digit1::<_, Error<_>>.parse_peek(Partial::new("21cZ")), Ok((Partial::new("Z"), "21c")));
/// assert_eq!(hex_digit1::<_, Error<_>>.parse_peek(Partial::new("H2")), Err(ErrMode::Backtrack(Error::new(Partial::new("H2"), ErrorKind::Slice))));
/// assert_eq!(hex_digit1::<_, Error<_>>.parse_peek(Partial::new("")), Err(ErrMode::Incomplete(Needed::new(1))));
/// ```
#[inline(always)]
pub fn hex_digit1<I, E: ParseError<I>>(input: &mut I) -> PResult<<I as Stream>::Slice, E>
where
    I: StreamIsPartial,
    I: Stream,
    <I as Stream>::Token: AsChar,
{
    trace(
        "hex_digit1",
        take_while(1.., |c: <I as Stream>::Token| c.is_hex_digit()),
    )
    .parse_next(input)
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
/// # use winnow::prelude::*;
/// # use winnow::{error::ErrMode, error::ErrorKind, error::Error, error::Needed};
/// # use winnow::ascii::oct_digit0;
/// fn parser<'s>(input: &mut &'s str) -> PResult<&'s str, Error<&'s str>> {
///     oct_digit0.parse_next(input)
/// }
///
/// assert_eq!(parser.parse_peek("21cZ"), Ok(("cZ", "21")));
/// assert_eq!(parser.parse_peek("Z21c"), Ok(("Z21c", "")));
/// assert_eq!(parser.parse_peek(""), Ok(("", "")));
/// ```
///
/// ```
/// # use winnow::prelude::*;
/// # use winnow::{error::ErrMode, error::ErrorKind, error::Error, error::Needed};
/// # use winnow::Partial;
/// # use winnow::ascii::oct_digit0;
/// assert_eq!(oct_digit0::<_, Error<_>>.parse_peek(Partial::new("21cZ")), Ok((Partial::new("cZ"), "21")));
/// assert_eq!(oct_digit0::<_, Error<_>>.parse_peek(Partial::new("Z21c")), Ok((Partial::new("Z21c"), "")));
/// assert_eq!(oct_digit0::<_, Error<_>>.parse_peek(Partial::new("")), Err(ErrMode::Incomplete(Needed::new(1))));
/// ```
#[inline(always)]
pub fn oct_digit0<I, E: ParseError<I>>(input: &mut I) -> PResult<<I as Stream>::Slice, E>
where
    I: StreamIsPartial,
    I: Stream,
    <I as Stream>::Token: AsChar,
{
    trace(
        "oct_digit0",
        take_while(0.., |c: <I as Stream>::Token| c.is_oct_digit()),
    )
    .parse_next(input)
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
/// # use winnow::prelude::*;
/// # use winnow::{error::ErrMode, error::{Error, ErrorKind}, error::Needed};
/// # use winnow::ascii::oct_digit1;
/// fn parser<'s>(input: &mut &'s str) -> PResult<&'s str, Error<&'s str>> {
///     oct_digit1.parse_next(input)
/// }
///
/// assert_eq!(parser.parse_peek("21cZ"), Ok(("cZ", "21")));
/// assert_eq!(parser.parse_peek("H2"), Err(ErrMode::Backtrack(Error::new("H2", ErrorKind::Slice))));
/// assert_eq!(parser.parse_peek(""), Err(ErrMode::Backtrack(Error::new("", ErrorKind::Slice))));
/// ```
///
/// ```
/// # use winnow::prelude::*;
/// # use winnow::{error::ErrMode, error::ErrorKind, error::Error, error::Needed};
/// # use winnow::Partial;
/// # use winnow::ascii::oct_digit1;
/// assert_eq!(oct_digit1::<_, Error<_>>.parse_peek(Partial::new("21cZ")), Ok((Partial::new("cZ"), "21")));
/// assert_eq!(oct_digit1::<_, Error<_>>.parse_peek(Partial::new("H2")), Err(ErrMode::Backtrack(Error::new(Partial::new("H2"), ErrorKind::Slice))));
/// assert_eq!(oct_digit1::<_, Error<_>>.parse_peek(Partial::new("")), Err(ErrMode::Incomplete(Needed::new(1))));
/// ```
#[inline(always)]
pub fn oct_digit1<I, E: ParseError<I>>(input: &mut I) -> PResult<<I as Stream>::Slice, E>
where
    I: StreamIsPartial,
    I: Stream,
    <I as Stream>::Token: AsChar,
{
    trace(
        "oct_digit0",
        take_while(1.., |c: <I as Stream>::Token| c.is_oct_digit()),
    )
    .parse_next(input)
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
/// # use winnow::prelude::*;
/// # use winnow::{error::ErrMode, error::ErrorKind, error::Error, error::Needed};
/// # use winnow::ascii::alphanumeric0;
/// fn parser<'s>(input: &mut &'s str) -> PResult<&'s str, Error<&'s str>> {
///     alphanumeric0.parse_next(input)
/// }
///
/// assert_eq!(parser.parse_peek("21cZ%1"), Ok(("%1", "21cZ")));
/// assert_eq!(parser.parse_peek("&Z21c"), Ok(("&Z21c", "")));
/// assert_eq!(parser.parse_peek(""), Ok(("", "")));
/// ```
///
/// ```
/// # use winnow::prelude::*;
/// # use winnow::{error::ErrMode, error::ErrorKind, error::Error, error::Needed};
/// # use winnow::Partial;
/// # use winnow::ascii::alphanumeric0;
/// assert_eq!(alphanumeric0::<_, Error<_>>.parse_peek(Partial::new("21cZ%1")), Ok((Partial::new("%1"), "21cZ")));
/// assert_eq!(alphanumeric0::<_, Error<_>>.parse_peek(Partial::new("&Z21c")), Ok((Partial::new("&Z21c"), "")));
/// assert_eq!(alphanumeric0::<_, Error<_>>.parse_peek(Partial::new("")), Err(ErrMode::Incomplete(Needed::new(1))));
/// ```
#[inline(always)]
pub fn alphanumeric0<I, E: ParseError<I>>(input: &mut I) -> PResult<<I as Stream>::Slice, E>
where
    I: StreamIsPartial,
    I: Stream,
    <I as Stream>::Token: AsChar,
{
    trace(
        "alphanumeric0",
        take_while(0.., |c: <I as Stream>::Token| c.is_alphanum()),
    )
    .parse_next(input)
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
/// # use winnow::prelude::*;
/// # use winnow::{error::ErrMode, error::{Error, ErrorKind}, error::Needed};
/// # use winnow::ascii::alphanumeric1;
/// fn parser<'s>(input: &mut &'s str) -> PResult<&'s str, Error<&'s str>> {
///     alphanumeric1.parse_next(input)
/// }
///
/// assert_eq!(parser.parse_peek("21cZ%1"), Ok(("%1", "21cZ")));
/// assert_eq!(parser.parse_peek("&H2"), Err(ErrMode::Backtrack(Error::new("&H2", ErrorKind::Slice))));
/// assert_eq!(parser.parse_peek(""), Err(ErrMode::Backtrack(Error::new("", ErrorKind::Slice))));
/// ```
///
/// ```
/// # use winnow::prelude::*;
/// # use winnow::{error::ErrMode, error::ErrorKind, error::Error, error::Needed};
/// # use winnow::Partial;
/// # use winnow::ascii::alphanumeric1;
/// assert_eq!(alphanumeric1::<_, Error<_>>.parse_peek(Partial::new("21cZ%1")), Ok((Partial::new("%1"), "21cZ")));
/// assert_eq!(alphanumeric1::<_, Error<_>>.parse_peek(Partial::new("&H2")), Err(ErrMode::Backtrack(Error::new(Partial::new("&H2"), ErrorKind::Slice))));
/// assert_eq!(alphanumeric1::<_, Error<_>>.parse_peek(Partial::new("")), Err(ErrMode::Incomplete(Needed::new(1))));
/// ```
#[inline(always)]
pub fn alphanumeric1<I, E: ParseError<I>>(input: &mut I) -> PResult<<I as Stream>::Slice, E>
where
    I: StreamIsPartial,
    I: Stream,
    <I as Stream>::Token: AsChar,
{
    trace(
        "alphanumeric1",
        take_while(1.., |c: <I as Stream>::Token| c.is_alphanum()),
    )
    .parse_next(input)
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
/// # use winnow::prelude::*;
/// # use winnow::{error::ErrMode, error::ErrorKind, error::Error, error::Needed};
/// # use winnow::Partial;
/// # use winnow::ascii::space0;
/// assert_eq!(space0::<_, Error<_>>.parse_peek(Partial::new(" \t21c")), Ok((Partial::new("21c"), " \t")));
/// assert_eq!(space0::<_, Error<_>>.parse_peek(Partial::new("Z21c")), Ok((Partial::new("Z21c"), "")));
/// assert_eq!(space0::<_, Error<_>>.parse_peek(Partial::new("")), Err(ErrMode::Incomplete(Needed::new(1))));
/// ```
#[inline(always)]
pub fn space0<I, E: ParseError<I>>(input: &mut I) -> PResult<<I as Stream>::Slice, E>
where
    I: StreamIsPartial,
    I: Stream,
    <I as Stream>::Token: AsChar + Copy,
{
    trace(
        "space0",
        take_while(0.., |c: <I as Stream>::Token| {
            let ch = c.as_char();
            matches!(ch, ' ' | '\t')
        }),
    )
    .parse_next(input)
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
/// # use winnow::prelude::*;
/// # use winnow::{error::ErrMode, error::ErrorKind, error::Error, error::Needed};
/// # use winnow::ascii::space1;
/// fn parser<'s>(input: &mut &'s str) -> PResult<&'s str, Error<&'s str>> {
///     space1.parse_next(input)
/// }
///
/// assert_eq!(parser.parse_peek(" \t21c"), Ok(("21c", " \t")));
/// assert_eq!(parser.parse_peek("H2"), Err(ErrMode::Backtrack(Error::new("H2", ErrorKind::Slice))));
/// assert_eq!(parser.parse_peek(""), Err(ErrMode::Backtrack(Error::new("", ErrorKind::Slice))));
/// ```
///
/// ```
/// # use winnow::prelude::*;
/// # use winnow::{error::ErrMode, error::ErrorKind, error::Error, error::Needed};
/// # use winnow::Partial;
/// # use winnow::ascii::space1;
/// assert_eq!(space1::<_, Error<_>>.parse_peek(Partial::new(" \t21c")), Ok((Partial::new("21c"), " \t")));
/// assert_eq!(space1::<_, Error<_>>.parse_peek(Partial::new("H2")), Err(ErrMode::Backtrack(Error::new(Partial::new("H2"), ErrorKind::Slice))));
/// assert_eq!(space1::<_, Error<_>>.parse_peek(Partial::new("")), Err(ErrMode::Incomplete(Needed::new(1))));
/// ```
#[inline(always)]
pub fn space1<I, E: ParseError<I>>(input: &mut I) -> PResult<<I as Stream>::Slice, E>
where
    I: StreamIsPartial,
    I: Stream,
    <I as Stream>::Token: AsChar + Copy,
{
    trace(
        "space1",
        take_while(1.., |c: <I as Stream>::Token| {
            let ch = c.as_char();
            matches!(ch, ' ' | '\t')
        }),
    )
    .parse_next(input)
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
/// # use winnow::prelude::*;
/// # use winnow::{error::ErrMode, error::ErrorKind, error::Error, error::Needed};
/// # use winnow::ascii::multispace0;
/// fn parser<'s>(input: &mut &'s str) -> PResult<&'s str, Error<&'s str>> {
///     multispace0.parse_next(input)
/// }
///
/// assert_eq!(parser.parse_peek(" \t\n\r21c"), Ok(("21c", " \t\n\r")));
/// assert_eq!(parser.parse_peek("Z21c"), Ok(("Z21c", "")));
/// assert_eq!(parser.parse_peek(""), Ok(("", "")));
/// ```
///
/// ```
/// # use winnow::prelude::*;
/// # use winnow::{error::ErrMode, error::ErrorKind, error::Error, error::Needed};
/// # use winnow::Partial;
/// # use winnow::ascii::multispace0;
/// assert_eq!(multispace0::<_, Error<_>>.parse_peek(Partial::new(" \t\n\r21c")), Ok((Partial::new("21c"), " \t\n\r")));
/// assert_eq!(multispace0::<_, Error<_>>.parse_peek(Partial::new("Z21c")), Ok((Partial::new("Z21c"), "")));
/// assert_eq!(multispace0::<_, Error<_>>.parse_peek(Partial::new("")), Err(ErrMode::Incomplete(Needed::new(1))));
/// ```
#[inline(always)]
pub fn multispace0<I, E: ParseError<I>>(input: &mut I) -> PResult<<I as Stream>::Slice, E>
where
    I: StreamIsPartial,
    I: Stream,
    <I as Stream>::Token: AsChar + Copy,
{
    trace(
        "multispace0",
        take_while(0.., |c: <I as Stream>::Token| {
            let ch = c.as_char();
            matches!(ch, ' ' | '\t' | '\r' | '\n')
        }),
    )
    .parse_next(input)
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
/// # use winnow::prelude::*;
/// # use winnow::{error::ErrMode, error::{Error, ErrorKind}, error::Needed};
/// # use winnow::ascii::multispace1;
/// fn parser<'s>(input: &mut &'s str) -> PResult<&'s str, Error<&'s str>> {
///     multispace1.parse_next(input)
/// }
///
/// assert_eq!(parser.parse_peek(" \t\n\r21c"), Ok(("21c", " \t\n\r")));
/// assert_eq!(parser.parse_peek("H2"), Err(ErrMode::Backtrack(Error::new("H2", ErrorKind::Slice))));
/// assert_eq!(parser.parse_peek(""), Err(ErrMode::Backtrack(Error::new("", ErrorKind::Slice))));
/// ```
///
/// ```
/// # use winnow::prelude::*;
/// # use winnow::{error::ErrMode, error::ErrorKind, error::Error, error::Needed};
/// # use winnow::Partial;
/// # use winnow::ascii::multispace1;
/// assert_eq!(multispace1::<_, Error<_>>.parse_peek(Partial::new(" \t\n\r21c")), Ok((Partial::new("21c"), " \t\n\r")));
/// assert_eq!(multispace1::<_, Error<_>>.parse_peek(Partial::new("H2")), Err(ErrMode::Backtrack(Error::new(Partial::new("H2"), ErrorKind::Slice))));
/// assert_eq!(multispace1::<_, Error<_>>.parse_peek(Partial::new("")), Err(ErrMode::Incomplete(Needed::new(1))));
/// ```
#[inline(always)]
pub fn multispace1<I, E: ParseError<I>>(input: &mut I) -> PResult<<I as Stream>::Slice, E>
where
    I: StreamIsPartial,
    I: Stream,
    <I as Stream>::Token: AsChar + Copy,
{
    trace(
        "multispace1",
        take_while(1.., |c: <I as Stream>::Token| {
            let ch = c.as_char();
            matches!(ch, ' ' | '\t' | '\r' | '\n')
        }),
    )
    .parse_next(input)
}

/// Decode a decimal unsigned integer
///
/// *Complete version*: can parse until the end of input.
///
/// *Partial version*: Will return `Err(winnow::error::ErrMode::Incomplete(_))` if there's not enough input data.
#[doc(alias = "u8")]
#[doc(alias = "u16")]
#[doc(alias = "u32")]
#[doc(alias = "u64")]
#[doc(alias = "u128")]
pub fn dec_uint<I, O, E: ParseError<I>>(input: &mut I) -> PResult<O, E>
where
    I: StreamIsPartial,
    I: Stream,
    <I as Stream>::Token: AsChar + Copy,
    O: Uint,
{
    trace("dec_uint", move |input: &mut I| {
        if input.eof_offset() == 0 {
            if <I as StreamIsPartial>::is_partial_supported() && input.is_partial() {
                return Err(ErrMode::Incomplete(Needed::new(1)));
            } else {
                return Err(ErrMode::from_error_kind(input.clone(), ErrorKind::Slice));
            }
        }

        let mut value = O::default();
        for (offset, c) in input.iter_offsets() {
            match c.as_char().to_digit(10) {
                Some(d) => match value.checked_mul(10, sealed::SealedMarker).and_then(|v| {
                    let d = d as u8;
                    v.checked_add(d, sealed::SealedMarker)
                }) {
                    None => return Err(ErrMode::from_error_kind(input.clone(), ErrorKind::Verify)),
                    Some(v) => value = v,
                },
                None => {
                    if offset == 0 {
                        return Err(ErrMode::from_error_kind(input.clone(), ErrorKind::Slice));
                    } else {
                        let _ = input.next_slice(offset);
                        return Ok(value);
                    }
                }
            }
        }

        if <I as StreamIsPartial>::is_partial_supported() && input.is_partial() {
            Err(ErrMode::Incomplete(Needed::new(1)))
        } else {
            let _ = input.finish();
            Ok(value)
        }
    })
    .parse_next(input)
}

/// Metadata for parsing unsigned integers, see [`dec_uint`]
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
#[doc(alias = "i8")]
#[doc(alias = "i16")]
#[doc(alias = "i32")]
#[doc(alias = "i64")]
#[doc(alias = "i128")]
pub fn dec_int<I, O, E: ParseError<I>>(input: &mut I) -> PResult<O, E>
where
    I: StreamIsPartial,
    I: Stream,
    <I as Stream>::Token: AsChar + Copy,
    O: Int,
{
    trace("dec_int", move |input: &mut I| {
        fn sign(token: impl AsChar) -> bool {
            let token = token.as_char();
            token == '+' || token == '-'
        }
        let sign = opt(crate::token::one_of(sign).map(AsChar::as_char))
            .map(|c| c != Some('-'))
            .parse_next(input)?;

        if input.eof_offset() == 0 {
            if <I as StreamIsPartial>::is_partial_supported() && input.is_partial() {
                return Err(ErrMode::Incomplete(Needed::new(1)));
            } else {
                return Err(ErrMode::from_error_kind(input.clone(), ErrorKind::Slice));
            }
        }

        let mut value = O::default();
        for (offset, c) in input.iter_offsets() {
            match c.as_char().to_digit(10) {
                Some(d) => match value.checked_mul(10, sealed::SealedMarker).and_then(|v| {
                    let d = d as u8;
                    if sign {
                        v.checked_add(d, sealed::SealedMarker)
                    } else {
                        v.checked_sub(d, sealed::SealedMarker)
                    }
                }) {
                    None => return Err(ErrMode::from_error_kind(input.clone(), ErrorKind::Verify)),
                    Some(v) => value = v,
                },
                None => {
                    if offset == 0 {
                        return Err(ErrMode::from_error_kind(input.clone(), ErrorKind::Slice));
                    } else {
                        let _ = input.next_slice(offset);
                        return Ok(value);
                    }
                }
            }
        }

        if <I as StreamIsPartial>::is_partial_supported() && input.is_partial() {
            Err(ErrMode::Incomplete(Needed::new(1)))
        } else {
            let _ = input.finish();
            Ok(value)
        }
    })
    .parse_next(input)
}

/// Metadata for parsing signed integers, see [`dec_int`]
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
/// use winnow::ascii::hex_uint;
///
/// fn parser<'s>(s: &mut &'s [u8]) -> PResult<u32, Error<&'s [u8]>> {
///   hex_uint(s)
/// }
///
/// assert_eq!(parser.parse_peek(&b"01AE"[..]), Ok((&b""[..], 0x01AE)));
/// assert_eq!(parser.parse_peek(&b"abc"[..]), Ok((&b""[..], 0x0ABC)));
/// assert_eq!(parser.parse_peek(&b"ggg"[..]), Err(ErrMode::Backtrack(Error::new(&b"ggg"[..], ErrorKind::Slice))));
/// ```
///
/// ```rust
/// # use winnow::prelude::*;
/// # use winnow::{error::ErrMode, error::ErrorKind, error::Error, error::Needed};
/// # use winnow::Partial;
/// use winnow::ascii::hex_uint;
///
/// fn parser<'s>(s: &mut Partial<&'s [u8]>) -> PResult<u32, Error<Partial<&'s [u8]>>> {
///   hex_uint(s)
/// }
///
/// assert_eq!(parser.parse_peek(Partial::new(&b"01AE;"[..])), Ok((Partial::new(&b";"[..]), 0x01AE)));
/// assert_eq!(parser.parse_peek(Partial::new(&b"abc"[..])), Err(ErrMode::Incomplete(Needed::new(1))));
/// assert_eq!(parser.parse_peek(Partial::new(&b"ggg"[..])), Err(ErrMode::Backtrack(Error::new(Partial::new(&b"ggg"[..]), ErrorKind::Slice))));
/// ```
#[inline]
pub fn hex_uint<I, O, E: ParseError<I>>(input: &mut I) -> PResult<O, E>
where
    I: StreamIsPartial,
    I: Stream,
    O: HexUint,
    <I as Stream>::Token: AsChar,
    <I as Stream>::Slice: AsBStr,
{
    trace("hex_uint", move |input: &mut I| {
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
                    return Err(ErrMode::from_error_kind(input.clone(), ErrorKind::Verify));
                } else {
                    invalid_offset
                }
            }
            Err(_) => {
                if <I as StreamIsPartial>::is_partial_supported()
                    && input.is_partial()
                    && invalid_offset == input.eof_offset()
                {
                    // Only the next byte is guaranteed required
                    return Err(ErrMode::Incomplete(Needed::new(1)));
                } else {
                    invalid_offset
                }
            }
        };
        if offset == 0 {
            // Must be at least one digit
            return Err(ErrMode::from_error_kind(input.clone(), ErrorKind::Slice));
        }
        let parsed = input.next_slice(offset);

        let mut res = O::default();
        for c in parsed.as_bstr() {
            let nibble = *c as char;
            let nibble = nibble.to_digit(16).unwrap_or(0) as u8;
            let nibble = O::from(nibble);
            res = (res << O::from(4)) + nibble;
        }

        Ok(res)
    })
    .parse_next(input)
}

/// Metadata for parsing hex numbers, see [`hex_uint`]
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

/// Recognizes floating point number in text format and returns a f32 or f64.
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
/// use winnow::ascii::float;
///
/// fn parser<'s>(s: &mut &'s str) -> PResult<f64, Error<&'s str>> {
///   float(s)
/// }
///
/// assert_eq!(parser.parse_peek("11e-1"), Ok(("", 1.1)));
/// assert_eq!(parser.parse_peek("123E-02"), Ok(("", 1.23)));
/// assert_eq!(parser.parse_peek("123K-01"), Ok(("K-01", 123.0)));
/// assert_eq!(parser.parse_peek("abc"), Err(ErrMode::Backtrack(Error::new("abc", ErrorKind::Tag))));
/// ```
///
/// ```rust
/// # use winnow::prelude::*;
/// # use winnow::{error::ErrMode, error::ErrorKind, error::Error, error::Needed};
/// # use winnow::error::Needed::Size;
/// # use winnow::Partial;
/// use winnow::ascii::float;
///
/// fn parser<'s>(s: &mut Partial<&'s str>) -> PResult<f64, Error<Partial<&'s str>>> {
///   float(s)
/// }
///
/// assert_eq!(parser.parse_peek(Partial::new("11e-1 ")), Ok((Partial::new(" "), 1.1)));
/// assert_eq!(parser.parse_peek(Partial::new("11e-1")), Err(ErrMode::Incomplete(Needed::new(1))));
/// assert_eq!(parser.parse_peek(Partial::new("123E-02")), Err(ErrMode::Incomplete(Needed::new(1))));
/// assert_eq!(parser.parse_peek(Partial::new("123K-01")), Ok((Partial::new("K-01"), 123.0)));
/// assert_eq!(parser.parse_peek(Partial::new("abc")), Err(ErrMode::Backtrack(Error::new(Partial::new("abc"), ErrorKind::Tag))));
/// ```
#[inline(always)]
#[doc(alias = "f32")]
#[doc(alias = "double")]
pub fn float<I, O, E: ParseError<I>>(input: &mut I) -> PResult<O, E>
where
    I: StreamIsPartial,
    I: Stream,
    I: Compare<&'static str>,
    <I as Stream>::Slice: ParseSlice<O>,
    <I as Stream>::Token: AsChar + Copy,
    <I as Stream>::IterOffsets: Clone,
    I: AsBStr,
{
    trace("float", move |input: &mut I| {
        let s = recognize_float_or_exceptions(input)?;
        s.parse_slice()
            .ok_or_else(|| ErrMode::from_error_kind(input.clone(), ErrorKind::Verify))
    })
    .parse_next(input)
}

fn recognize_float_or_exceptions<I, E: ParseError<I>>(
    input: &mut I,
) -> PResult<<I as Stream>::Slice, E>
where
    I: StreamIsPartial,
    I: Stream,
    I: Compare<&'static str>,
    <I as Stream>::Token: AsChar + Copy,
    <I as Stream>::IterOffsets: Clone,
    I: AsBStr,
{
    alt((
        recognize_float,
        crate::token::tag_no_case("nan"),
        crate::token::tag_no_case("infinity"),
        crate::token::tag_no_case("inf"),
    ))
    .parse_next(input)
}

fn recognize_float<I, E: ParseError<I>>(input: &mut I) -> PResult<<I as Stream>::Slice, E>
where
    I: StreamIsPartial,
    I: Stream,
    I: Compare<&'static str>,
    <I as Stream>::Token: AsChar + Copy,
    <I as Stream>::IterOffsets: Clone,
    I: AsBStr,
{
    (
        opt(one_of(['+', '-'])),
        alt((
            (digit1, opt(('.', opt(digit1)))).map(|_| ()),
            ('.', digit1).map(|_| ()),
        )),
        opt((one_of(['e', 'E']), opt(one_of(['+', '-'])), cut_err(digit1))),
    )
        .recognize()
        .parse_next(input)
}

/// Matches a byte string with escaped characters.
///
/// * The first argument matches the normal characters (it must not accept the control character)
/// * The second argument is the control character (like `\` in most languages)
/// * The third argument matches the escaped characters
///
/// # Example
///
/// ```rust
/// # use winnow::prelude::*;
/// # use winnow::{error::ErrMode, error::ErrorKind, error::Error, error::Needed, IResult};
/// # use winnow::ascii::digit1;
/// # use winnow::prelude::*;
/// use winnow::ascii::escaped;
/// use winnow::token::one_of;
///
/// fn esc(s: &str) -> IResult<&str, &str> {
///   escaped(digit1, '\\', one_of(['"', 'n', '\\'])).parse_peek(s)
/// }
///
/// assert_eq!(esc("123;"), Ok((";", "123")));
/// assert_eq!(esc(r#"12\"34;"#), Ok((";", r#"12\"34"#)));
/// ```
///
/// ```rust
/// # use winnow::prelude::*;
/// # use winnow::{error::ErrMode, error::ErrorKind, error::Error, error::Needed, IResult};
/// # use winnow::ascii::digit1;
/// # use winnow::prelude::*;
/// # use winnow::Partial;
/// use winnow::ascii::escaped;
/// use winnow::token::one_of;
///
/// fn esc(s: Partial<&str>) -> IResult<Partial<&str>, &str> {
///   escaped(digit1, '\\', one_of(['"', 'n', '\\'])).parse_peek(s)
/// }
///
/// assert_eq!(esc(Partial::new("123;")), Ok((Partial::new(";"), "123")));
/// assert_eq!(esc(Partial::new("12\\\"34;")), Ok((Partial::new(";"), "12\\\"34")));
/// ```
#[inline(always)]
pub fn escaped<'a, I: 'a, Error, F, G, O1, O2>(
    mut normal: F,
    control_char: char,
    mut escapable: G,
) -> impl Parser<I, <I as Stream>::Slice, Error>
where
    I: StreamIsPartial,
    I: Stream,
    <I as Stream>::Token: crate::stream::AsChar,
    F: Parser<I, O1, Error>,
    G: Parser<I, O2, Error>,
    Error: ParseError<I>,
{
    trace("escaped", move |input: &mut I| {
        if <I as StreamIsPartial>::is_partial_supported() && input.is_partial() {
            streaming_escaped_internal(input, &mut normal, control_char, &mut escapable)
        } else {
            complete_escaped_internal(input, &mut normal, control_char, &mut escapable)
        }
    })
}

fn streaming_escaped_internal<I, Error, F, G, O1, O2>(
    input: &mut I,
    normal: &mut F,
    control_char: char,
    escapable: &mut G,
) -> PResult<<I as Stream>::Slice, Error>
where
    I: StreamIsPartial,
    I: Stream,
    <I as Stream>::Token: crate::stream::AsChar,
    F: Parser<I, O1, Error>,
    G: Parser<I, O2, Error>,
    Error: ParseError<I>,
{
    let start = input.checkpoint();

    while input.eof_offset() > 0 {
        let current_len = input.eof_offset();

        match normal.parse_next(input) {
            Ok(_) => {
                if input.eof_offset() == 0 {
                    return Err(ErrMode::Incomplete(Needed::Unknown));
                } else if input.eof_offset() == current_len {
                    let offset = input.offset_from(&start);
                    input.reset(start);
                    return Ok(input.next_slice(offset));
                }
            }
            Err(ErrMode::Backtrack(_)) => {
                if input.peek_token().expect("eof_offset > 0").1.as_char() == control_char {
                    let next = control_char.len_utf8();
                    let _ = input.next_slice(next);
                    let _ = escapable.parse_next(input)?;
                    if input.eof_offset() == 0 {
                        return Err(ErrMode::Incomplete(Needed::Unknown));
                    }
                } else {
                    let offset = input.offset_from(&start);
                    input.reset(start);
                    return Ok(input.next_slice(offset));
                }
            }
            Err(e) => {
                return Err(e);
            }
        }
    }

    Err(ErrMode::Incomplete(Needed::Unknown))
}

fn complete_escaped_internal<'a, I: 'a, Error, F, G, O1, O2>(
    input: &mut I,
    normal: &mut F,
    control_char: char,
    escapable: &mut G,
) -> PResult<<I as Stream>::Slice, Error>
where
    I: StreamIsPartial,
    I: Stream,
    <I as Stream>::Token: crate::stream::AsChar,
    F: Parser<I, O1, Error>,
    G: Parser<I, O2, Error>,
    Error: ParseError<I>,
{
    let start = input.checkpoint();

    while input.eof_offset() > 0 {
        let current_len = input.eof_offset();

        match normal.parse_next(input) {
            Ok(_) => {
                // return if we consumed everything or if the normal parser
                // does not consume anything
                if input.eof_offset() == 0 {
                    input.reset(start);
                    return Ok(input.finish());
                } else if input.eof_offset() == current_len {
                    let offset = input.offset_from(&start);
                    input.reset(start);
                    return Ok(input.next_slice(offset));
                }
            }
            Err(ErrMode::Backtrack(_)) => {
                if input.peek_token().expect("eof_offset > 0").1.as_char() == control_char {
                    let next = control_char.len_utf8();
                    let _ = input.next_slice(next);
                    let _ = escapable.parse_next(input)?;
                    if input.eof_offset() == 0 {
                        input.reset(start);
                        return Ok(input.finish());
                    }
                } else {
                    let offset = input.offset_from(&start);
                    input.reset(start);
                    return Ok(input.next_slice(offset));
                }
            }
            Err(e) => {
                return Err(e);
            }
        }
    }

    Ok(input.finish())
}

/// Matches a byte string with escaped characters.
///
/// * The first argument matches the normal characters (it must not match the control character)
/// * The second argument is the control character (like `\` in most languages)
/// * The third argument matches the escaped characters and transforms them
///
/// As an example, the chain `abc\tdef` could be `abc    def` (it also consumes the control character)
///
/// # Example
///
/// ```rust
/// # use winnow::prelude::*;
/// # use winnow::{error::ErrMode, error::ErrorKind, error::Error, error::Needed};
/// # use std::str::from_utf8;
/// use winnow::token::tag;
/// use winnow::ascii::escaped_transform;
/// use winnow::ascii::alpha1;
/// use winnow::combinator::alt;
///
/// fn parser<'s>(input: &mut &'s str) -> PResult<String, Error<&'s str>> {
///   escaped_transform(
///     alpha1,
///     '\\',
///     alt((
///       "\\".value("\\"),
///       "\"".value("\""),
///       "n".value("\n"),
///     ))
///   ).parse_next(input)
/// }
///
/// assert_eq!(parser.parse_peek("ab\\\"cd"), Ok(("", String::from("ab\"cd"))));
/// assert_eq!(parser.parse_peek("ab\\ncd"), Ok(("", String::from("ab\ncd"))));
/// ```
///
/// ```
/// # use winnow::prelude::*;
/// # use winnow::{error::ErrMode, error::ErrorKind, error::Error, error::Needed};
/// # use std::str::from_utf8;
/// # use winnow::Partial;
/// use winnow::token::tag;
/// use winnow::ascii::escaped_transform;
/// use winnow::ascii::alpha1;
/// use winnow::combinator::alt;
///
/// fn parser<'s>(input: &mut Partial<&'s str>) -> PResult<String, Error<Partial<&'s str>>> {
///   escaped_transform(
///     alpha1,
///     '\\',
///     alt((
///       "\\".value("\\"),
///       "\"".value("\""),
///       "n".value("\n"),
///     ))
///   ).parse_next(input)
/// }
///
/// assert_eq!(parser.parse_peek(Partial::new("ab\\\"cd\"")), Ok((Partial::new("\""), String::from("ab\"cd"))));
/// ```
#[inline(always)]
pub fn escaped_transform<I, Error, F, G, Output>(
    mut normal: F,
    control_char: char,
    mut transform: G,
) -> impl Parser<I, Output, Error>
where
    I: StreamIsPartial,
    I: Stream,
    <I as Stream>::Token: crate::stream::AsChar,
    Output: crate::stream::Accumulate<<I as Stream>::Slice>,
    F: Parser<I, <I as Stream>::Slice, Error>,
    G: Parser<I, <I as Stream>::Slice, Error>,
    Error: ParseError<I>,
{
    trace("escaped_transform", move |input: &mut I| {
        if <I as StreamIsPartial>::is_partial_supported() && input.is_partial() {
            streaming_escaped_transform_internal(input, &mut normal, control_char, &mut transform)
        } else {
            complete_escaped_transform_internal(input, &mut normal, control_char, &mut transform)
        }
    })
}

fn streaming_escaped_transform_internal<I, Error, F, G, Output>(
    input: &mut I,
    normal: &mut F,
    control_char: char,
    transform: &mut G,
) -> PResult<Output, Error>
where
    I: StreamIsPartial,
    I: Stream,
    <I as Stream>::Token: crate::stream::AsChar,
    Output: crate::stream::Accumulate<<I as Stream>::Slice>,
    F: Parser<I, <I as Stream>::Slice, Error>,
    G: Parser<I, <I as Stream>::Slice, Error>,
    Error: ParseError<I>,
{
    let mut res = Output::initial(Some(input.eof_offset()));

    while input.eof_offset() > 0 {
        let current_len = input.eof_offset();
        match normal.parse_next(input) {
            Ok(o) => {
                res.accumulate(o);
                if input.eof_offset() == 0 {
                    return Err(ErrMode::Incomplete(Needed::Unknown));
                } else if input.eof_offset() == current_len {
                    return Ok(res);
                }
            }
            Err(ErrMode::Backtrack(_)) => {
                if input.peek_token().expect("eof_offset > 0").1.as_char() == control_char {
                    let next = control_char.len_utf8();
                    let _ = input.next_slice(next);
                    let o = transform.parse_next(input)?;
                    res.accumulate(o);
                    if input.eof_offset() == 0 {
                        return Err(ErrMode::Incomplete(Needed::Unknown));
                    }
                } else {
                    return Ok(res);
                }
            }
            Err(e) => return Err(e),
        }
    }
    Err(ErrMode::Incomplete(Needed::Unknown))
}

fn complete_escaped_transform_internal<I, Error, F, G, Output>(
    input: &mut I,
    normal: &mut F,
    control_char: char,
    transform: &mut G,
) -> PResult<Output, Error>
where
    I: StreamIsPartial,
    I: Stream,
    <I as Stream>::Token: crate::stream::AsChar,
    Output: crate::stream::Accumulate<<I as Stream>::Slice>,
    F: Parser<I, <I as Stream>::Slice, Error>,
    G: Parser<I, <I as Stream>::Slice, Error>,
    Error: ParseError<I>,
{
    let mut res = Output::initial(Some(input.eof_offset()));

    while input.eof_offset() > 0 {
        let current_len = input.eof_offset();

        match normal.parse_next(input) {
            Ok(o) => {
                res.accumulate(o);
                if input.eof_offset() == 0 {
                    return Ok(res);
                } else if input.eof_offset() == current_len {
                    return Ok(res);
                }
            }
            Err(ErrMode::Backtrack(_)) => {
                if input.peek_token().expect("eof_offset > 0").1.as_char() == control_char {
                    let next = control_char.len_utf8();
                    let _ = input.next_slice(next);
                    let o = transform.parse_next(input)?;
                    res.accumulate(o);
                    if input.eof_offset() == 0 {
                        return Ok(res);
                    }
                } else {
                    return Ok(res);
                }
            }
            Err(e) => return Err(e),
        }
    }
    Ok(res)
}

mod sealed {
    pub struct SealedMarker;
}
