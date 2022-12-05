//! Character specific parsers and combinators
//!
//! Functions recognizing specific characters

pub mod complete;
pub mod streaming;
#[cfg(test)]
mod tests;

use crate::error::ParseError;
use crate::input::Compare;
use crate::input::{
  AsChar, FindToken, InputIsStreaming, InputIter, InputLength, InputTake, InputTakeAtPosition,
  IntoOutput, Slice,
};
use crate::lib::std::ops::{Range, RangeFrom, RangeTo};
use crate::IResult;

/// Recognizes one character.
///
/// *Complete version*: Will return an error if there's not enough input data.
///
/// *Streaming version*: Will return `Err(nom::Err::Incomplete(_))` if there's not enough input data.
///
/// # Example
///
/// ```
/// # use nom::{Err, error::{ErrorKind, Error}, IResult};
/// # use nom::character::char;
/// fn parser(i: &str) -> IResult<&str, char> {
///     char('a')(i)
/// }
/// assert_eq!(parser("abc"), Ok(("bc", 'a')));
/// assert_eq!(parser(" abc"), Err(Err::Error(Error::new(" abc", ErrorKind::Char))));
/// assert_eq!(parser("bc"), Err(Err::Error(Error::new("bc", ErrorKind::Char))));
/// assert_eq!(parser(""), Err(Err::Error(Error::new("", ErrorKind::Char))));
/// ```
///
/// ```
/// # use nom::{Err, error::{ErrorKind, Error}, Needed, IResult};
/// # use nom::input::Streaming;
/// # use nom::character::char;
/// fn parser(i: Streaming<&str>) -> IResult<Streaming<&str>, char> {
///     char('a')(i)
/// }
/// assert_eq!(parser(Streaming("abc")), Ok((Streaming("bc"), 'a')));
/// assert_eq!(parser(Streaming("bc")), Err(Err::Error(Error::new(Streaming("bc"), ErrorKind::Char))));
/// assert_eq!(parser(Streaming("")), Err(Err::Incomplete(Needed::new(1))));
/// ```
#[inline(always)]
pub fn char<I, Error: ParseError<I>, const STREAMING: bool>(
  c: char,
) -> impl Fn(I) -> IResult<I, char, Error>
where
  I: Slice<RangeFrom<usize>> + InputIter + InputLength + InputIsStreaming<STREAMING>,
  <I as InputIter>::Item: AsChar,
{
  move |i: I| {
    if STREAMING {
      streaming::char_internal(i, c)
    } else {
      complete::char_internal(i, c)
    }
  }
}

/// Recognizes one character and checks that it satisfies a predicate
///
/// *Complete version*: Will return an error if there's not enough input data.
///
/// *Streaming version*: Will return `Err(nom::Err::Incomplete(_))` if there's not enough input data.
///
/// # Example
///
/// ```
/// # use nom::{Err, error::{ErrorKind, Error}, Needed, IResult};
/// # use nom::character::satisfy;
/// fn parser(i: &str) -> IResult<&str, char> {
///     satisfy(|c| c == 'a' || c == 'b')(i)
/// }
/// assert_eq!(parser("abc"), Ok(("bc", 'a')));
/// assert_eq!(parser("cd"), Err(Err::Error(Error::new("cd", ErrorKind::Satisfy))));
/// assert_eq!(parser(""), Err(Err::Error(Error::new("", ErrorKind::Satisfy))));
/// ```
///
/// ```
/// # use nom::{Err, error::{ErrorKind, Error}, Needed, IResult};
/// # use nom::input::Streaming;
/// # use nom::character::satisfy;
/// fn parser(i: Streaming<&str>) -> IResult<Streaming<&str>, char> {
///     satisfy(|c| c == 'a' || c == 'b')(i)
/// }
/// assert_eq!(parser(Streaming("abc")), Ok((Streaming("bc"), 'a')));
/// assert_eq!(parser(Streaming("cd")), Err(Err::Error(Error::new(Streaming("cd"), ErrorKind::Satisfy))));
/// assert_eq!(parser(Streaming("")), Err(Err::Incomplete(Needed::Unknown)));
/// ```
#[inline(always)]
pub fn satisfy<F, I, Error: ParseError<I>, const STREAMING: bool>(
  cond: F,
) -> impl Fn(I) -> IResult<I, char, Error>
where
  I: Slice<RangeFrom<usize>> + InputIter + InputIsStreaming<STREAMING>,
  <I as InputIter>::Item: AsChar,
  F: Fn(char) -> bool,
{
  move |i: I| {
    if STREAMING {
      streaming::satisfy_internal(i, &cond)
    } else {
      complete::satisfy_internal(i, &cond)
    }
  }
}

/// Recognizes one of the provided characters.
///
/// *Complete version*: Will return an error if there's not enough input data.
///
/// *Streaming version*: Will return `Err(nom::Err::Incomplete(_))` if there's not enough input data.
///
/// # Example
///
/// ```
/// # use nom::{Err, error::ErrorKind};
/// # use nom::character::one_of;
/// assert_eq!(one_of::<_, _, (&str, ErrorKind), false>("abc")("b"), Ok(("", 'b')));
/// assert_eq!(one_of::<_, _, (&str, ErrorKind), false>("a")("bc"), Err(Err::Error(("bc", ErrorKind::OneOf))));
/// assert_eq!(one_of::<_, _, (&str, ErrorKind), false>("a")(""), Err(Err::Error(("", ErrorKind::OneOf))));
/// ```
///
/// ```
/// # use nom::{Err, error::ErrorKind, Needed};
/// # use nom::input::Streaming;
/// # use nom::character::one_of;
/// assert_eq!(one_of::<_, _, (_, ErrorKind), true>("abc")(Streaming("b")), Ok((Streaming(""), 'b')));
/// assert_eq!(one_of::<_, _, (_, ErrorKind), true>("a")(Streaming("bc")), Err(Err::Error((Streaming("bc"), ErrorKind::OneOf))));
/// assert_eq!(one_of::<_, _, (_, ErrorKind), true>("a")(Streaming("")), Err(Err::Incomplete(Needed::new(1))));
/// ```
#[inline(always)]
pub fn one_of<I, T, Error: ParseError<I>, const STREAMING: bool>(
  list: T,
) -> impl Fn(I) -> IResult<I, char, Error>
where
  I: Slice<RangeFrom<usize>> + InputIter + InputIsStreaming<STREAMING>,
  <I as InputIter>::Item: AsChar + Copy,
  T: FindToken<<I as InputIter>::Item>,
{
  move |i: I| {
    if STREAMING {
      streaming::one_of_internal(i, &list)
    } else {
      complete::one_of_internal(i, &list)
    }
  }
}

/// Recognizes a character that is not in the provided characters.
///
/// *Complete version*: Will return an error if there's not enough input data.
///
/// *Streaming version*: Will return `Err(nom::Err::Incomplete(_))` if there's not enough input data.
///
/// # Example
///
/// ```
/// # use nom::{Err, error::ErrorKind};
/// # use nom::character::none_of;
/// assert_eq!(none_of::<_, _, (&str, ErrorKind), false>("abc")("z"), Ok(("", 'z')));
/// assert_eq!(none_of::<_, _, (&str, ErrorKind), false>("ab")("a"), Err(Err::Error(("a", ErrorKind::NoneOf))));
/// assert_eq!(none_of::<_, _, (&str, ErrorKind), false>("a")(""), Err(Err::Error(("", ErrorKind::NoneOf))));
/// ```
///
/// ```
/// # use nom::{Err, error::ErrorKind, Needed};
/// # use nom::input::Streaming;
/// # use nom::character::none_of;
/// assert_eq!(none_of::<_, _, (_, ErrorKind), true>("abc")(Streaming("z")), Ok((Streaming(""), 'z')));
/// assert_eq!(none_of::<_, _, (_, ErrorKind), true>("ab")(Streaming("a")), Err(Err::Error((Streaming("a"), ErrorKind::NoneOf))));
/// assert_eq!(none_of::<_, _, (_, ErrorKind), true>("a")(Streaming("")), Err(Err::Incomplete(Needed::new(1))));
/// ```
#[inline(always)]
pub fn none_of<I, T, Error: ParseError<I>, const STREAMING: bool>(
  list: T,
) -> impl Fn(I) -> IResult<I, char, Error>
where
  I: Slice<RangeFrom<usize>> + InputIter + InputIsStreaming<STREAMING>,
  <I as InputIter>::Item: AsChar + Copy,
  T: FindToken<<I as InputIter>::Item>,
{
  move |i: I| {
    if STREAMING {
      streaming::none_of_internal(i, &list)
    } else {
      complete::none_of_internal(i, &list)
    }
  }
}

/// Recognizes the string "\r\n".
///
/// *Complete version*: Will return an error if there's not enough input data.
///
/// *Streaming version*: Will return `Err(nom::Err::Incomplete(_))` if there's not enough input data.
///
/// # Example
///
/// ```
/// # use nom::{Err, error::{Error, ErrorKind}, IResult};
/// # use nom::character::crlf;
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
/// # use nom::{Err, error::ErrorKind, IResult, Needed};
/// # use nom::input::Streaming;
/// # use nom::character::crlf;
/// assert_eq!(crlf::<_, (_, ErrorKind), true>(Streaming("\r\nc")), Ok((Streaming("c"), "\r\n")));
/// assert_eq!(crlf::<_, (_, ErrorKind), true>(Streaming("ab\r\nc")), Err(Err::Error((Streaming("ab\r\nc"), ErrorKind::CrLf))));
/// assert_eq!(crlf::<_, (_, ErrorKind), true>(Streaming("")), Err(Err::Incomplete(Needed::new(2))));
/// ```
#[inline(always)]
pub fn crlf<T, E: ParseError<T>, const STREAMING: bool>(
  input: T,
) -> IResult<T, <T as IntoOutput>::Output, E>
where
  T: Slice<Range<usize>> + Slice<RangeFrom<usize>> + Slice<RangeTo<usize>>,
  T: InputIter + InputIsStreaming<STREAMING>,
  T: IntoOutput,
  T: Compare<&'static str>,
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
/// *Streaming version*: Will return `Err(nom::Err::Incomplete(_))` if there's not enough input data.
///
/// # Example
///
/// ```
/// # use nom::{Err, error::{Error, ErrorKind}, IResult, Needed};
/// # use nom::character::not_line_ending;
/// fn parser(input: &str) -> IResult<&str, &str> {
///     not_line_ending(input)
/// }
///
/// assert_eq!(parser("ab\r\nc"), Ok(("\r\nc", "ab")));
/// assert_eq!(parser("ab\nc"), Ok(("\nc", "ab")));
/// assert_eq!(parser("abc"), Ok(("", "abc")));
/// assert_eq!(parser(""), Ok(("", "")));
/// assert_eq!(parser("a\rb\nc"), Err(Err::Error(Error { input: "a\rb\nc", code: ErrorKind::Tag })));
/// assert_eq!(parser("a\rbc"), Err(Err::Error(Error { input: "a\rbc", code: ErrorKind::Tag })));
/// ```
///
/// ```
/// # use nom::{Err, error::{Error, ErrorKind}, IResult, Needed};
/// # use nom::input::Streaming;
/// # use nom::character::not_line_ending;
/// assert_eq!(not_line_ending::<_, (_, ErrorKind), true>(Streaming("ab\r\nc")), Ok((Streaming("\r\nc"), "ab")));
/// assert_eq!(not_line_ending::<_, (_, ErrorKind), true>(Streaming("abc")), Err(Err::Incomplete(Needed::Unknown)));
/// assert_eq!(not_line_ending::<_, (_, ErrorKind), true>(Streaming("")), Err(Err::Incomplete(Needed::Unknown)));
/// assert_eq!(not_line_ending::<_, (_, ErrorKind), true>(Streaming("a\rb\nc")), Err(Err::Error((Streaming("a\rb\nc"), ErrorKind::Tag ))));
/// assert_eq!(not_line_ending::<_, (_, ErrorKind), true>(Streaming("a\rbc")), Err(Err::Error((Streaming("a\rbc"), ErrorKind::Tag ))));
/// ```
#[inline(always)]
pub fn not_line_ending<T, E: ParseError<T>, const STREAMING: bool>(
  input: T,
) -> IResult<T, <T as IntoOutput>::Output, E>
where
  T: Slice<Range<usize>> + Slice<RangeFrom<usize>> + Slice<RangeTo<usize>>,
  T: InputIter + InputLength + InputIsStreaming<STREAMING>,
  T: IntoOutput,
  T: Compare<&'static str>,
  <T as InputIter>::Item: AsChar,
  <T as InputIter>::Item: AsChar,
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
/// *Streaming version*: Will return `Err(nom::Err::Incomplete(_))` if there's not enough input data.
///
/// # Example
///
/// ```
/// # use nom::{Err, error::{Error, ErrorKind}, IResult, Needed};
/// # use nom::character::line_ending;
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
/// # use nom::{Err, error::ErrorKind, IResult, Needed};
/// # use nom::input::Streaming;
/// # use nom::character::line_ending;
/// assert_eq!(line_ending::<_, (_, ErrorKind), true>(Streaming("\r\nc")), Ok((Streaming("c"), "\r\n")));
/// assert_eq!(line_ending::<_, (_, ErrorKind), true>(Streaming("ab\r\nc")), Err(Err::Error((Streaming("ab\r\nc"), ErrorKind::CrLf))));
/// assert_eq!(line_ending::<_, (_, ErrorKind), true>(Streaming("")), Err(Err::Incomplete(Needed::new(1))));
/// ```
#[inline(always)]
pub fn line_ending<T, E: ParseError<T>, const STREAMING: bool>(
  input: T,
) -> IResult<T, <T as IntoOutput>::Output, E>
where
  T: Slice<Range<usize>> + Slice<RangeFrom<usize>> + Slice<RangeTo<usize>>,
  T: InputIter + InputLength + InputIsStreaming<STREAMING>,
  T: IntoOutput,
  T: Compare<&'static str>,
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
/// *Streaming version*: Will return `Err(nom::Err::Incomplete(_))` if there's not enough input data.
///
/// # Example
///
/// ```
/// # use nom::{Err, error::{Error, ErrorKind}, IResult, Needed};
/// # use nom::character::newline;
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
/// # use nom::{Err, error::ErrorKind, IResult, Needed};
/// # use nom::input::Streaming;
/// # use nom::character::newline;
/// assert_eq!(newline::<_, (_, ErrorKind), true>(Streaming("\nc")), Ok((Streaming("c"), '\n')));
/// assert_eq!(newline::<_, (_, ErrorKind), true>(Streaming("\r\nc")), Err(Err::Error((Streaming("\r\nc"), ErrorKind::Char))));
/// assert_eq!(newline::<_, (_, ErrorKind), true>(Streaming("")), Err(Err::Incomplete(Needed::new(1))));
/// ```
#[inline(always)]
pub fn newline<I, Error: ParseError<I>, const STREAMING: bool>(input: I) -> IResult<I, char, Error>
where
  I: Slice<RangeFrom<usize>> + InputIter + InputLength + InputIsStreaming<STREAMING>,
  <I as InputIter>::Item: AsChar,
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
/// *Streaming version*: Will return `Err(nom::Err::Incomplete(_))` if there's not enough input data.
///
/// # Example
///
/// ```
/// # use nom::{Err, error::{Error, ErrorKind}, IResult, Needed};
/// # use nom::character::tab;
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
/// # use nom::{Err, error::ErrorKind, IResult, Needed};
/// # use nom::input::Streaming;
/// # use nom::character::tab;
/// assert_eq!(tab::<_, (_, ErrorKind), true>(Streaming("\tc")), Ok((Streaming("c"), '\t')));
/// assert_eq!(tab::<_, (_, ErrorKind), true>(Streaming("\r\nc")), Err(Err::Error((Streaming("\r\nc"), ErrorKind::Char))));
/// assert_eq!(tab::<_, (_, ErrorKind), true>(Streaming("")), Err(Err::Incomplete(Needed::new(1))));
/// ```
#[inline(always)]
pub fn tab<I, Error: ParseError<I>, const STREAMING: bool>(input: I) -> IResult<I, char, Error>
where
  I: Slice<RangeFrom<usize>> + InputIter + InputLength + InputIsStreaming<STREAMING>,
  <I as InputIter>::Item: AsChar,
{
  if STREAMING {
    streaming::tab(input)
  } else {
    complete::tab(input)
  }
}

/// Matches one byte as a character. Note that the input type will
/// accept a `str`, but not a `&[u8]`, unlike many other nom parsers.
///
/// *Complete version*: Will return an error if there's not enough input data.
///
/// *Streaming version*: Will return `Err(nom::Err::Incomplete(_))` if there's not enough input data.
///
/// # Example
///
/// ```
/// # use nom::{character::anychar, Err, error::{Error, ErrorKind}, IResult};
/// fn parser(input: &str) -> IResult<&str, char> {
///     anychar(input)
/// }
///
/// assert_eq!(parser("abc"), Ok(("bc",'a')));
/// assert_eq!(parser(""), Err(Err::Error(Error::new("", ErrorKind::Eof))));
/// ```
///
/// ```
/// # use nom::{character::anychar, Err, error::ErrorKind, IResult, Needed};
/// # use nom::input::Streaming;
/// assert_eq!(anychar::<_, (_, ErrorKind), true>(Streaming("abc")), Ok((Streaming("bc"),'a')));
/// assert_eq!(anychar::<_, (_, ErrorKind), true>(Streaming("")), Err(Err::Incomplete(Needed::new(1))));
/// ```
#[inline(always)]
pub fn anychar<T, E: ParseError<T>, const STREAMING: bool>(input: T) -> IResult<T, char, E>
where
  T: InputIter + InputLength + Slice<RangeFrom<usize>> + InputIsStreaming<STREAMING>,
  <T as InputIter>::Item: AsChar,
{
  if STREAMING {
    streaming::anychar(input)
  } else {
    complete::anychar(input)
  }
}

/// Recognizes zero or more lowercase and uppercase ASCII alphabetic characters: a-z, A-Z
///
/// *Complete version*: Will return the whole input if no terminating token is found (a non
/// alphabetic character).
///
/// *Streaming version*: Will return `Err(nom::Err::Incomplete(_))` if there's not enough input data,
/// or if no terminating token is found (a non alphabetic character).
///
/// # Example
///
/// ```
/// # use nom::{Err, error::ErrorKind, IResult, Needed};
/// # use nom::character::alpha0;
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
/// # use nom::{Err, error::ErrorKind, IResult, Needed};
/// # use nom::input::Streaming;
/// # use nom::character::alpha0;
/// assert_eq!(alpha0::<_, (_, ErrorKind), true>(Streaming("ab1c")), Ok((Streaming("1c"), "ab")));
/// assert_eq!(alpha0::<_, (_, ErrorKind), true>(Streaming("1c")), Ok((Streaming("1c"), "")));
/// assert_eq!(alpha0::<_, (_, ErrorKind), true>(Streaming("")), Err(Err::Incomplete(Needed::new(1))));
/// ```
#[inline(always)]
pub fn alpha0<T, E: ParseError<T>, const STREAMING: bool>(
  input: T,
) -> IResult<T, <T as IntoOutput>::Output, E>
where
  T: InputTakeAtPosition + InputIsStreaming<STREAMING>,
  T: IntoOutput,
  <T as InputTakeAtPosition>::Item: AsChar,
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
/// *Streaming version*: Will return `Err(nom::Err::Incomplete(_))` if there's not enough input data,
/// or if no terminating token is found (a non alphabetic character).
///
/// # Example
///
/// ```
/// # use nom::{Err, error::{Error, ErrorKind}, IResult, Needed};
/// # use nom::character::alpha1;
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
/// # use nom::{Err, error::ErrorKind, IResult, Needed};
/// # use nom::input::Streaming;
/// # use nom::character::alpha1;
/// assert_eq!(alpha1::<_, (_, ErrorKind), true>(Streaming("aB1c")), Ok((Streaming("1c"), "aB")));
/// assert_eq!(alpha1::<_, (_, ErrorKind), true>(Streaming("1c")), Err(Err::Error((Streaming("1c"), ErrorKind::Alpha))));
/// assert_eq!(alpha1::<_, (_, ErrorKind), true>(Streaming("")), Err(Err::Incomplete(Needed::new(1))));
/// ```
#[inline(always)]
pub fn alpha1<T, E: ParseError<T>, const STREAMING: bool>(
  input: T,
) -> IResult<T, <T as IntoOutput>::Output, E>
where
  T: InputTakeAtPosition + InputIsStreaming<STREAMING>,
  T: IntoOutput,
  <T as InputTakeAtPosition>::Item: AsChar,
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
/// *Streaming version*: Will return `Err(nom::Err::Incomplete(_))` if there's not enough input data,
/// or if no terminating token is found (a non digit character).
///
/// # Example
///
/// ```
/// # use nom::{Err, error::ErrorKind, IResult, Needed};
/// # use nom::character::digit0;
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
/// # use nom::{Err, error::ErrorKind, IResult, Needed};
/// # use nom::input::Streaming;
/// # use nom::character::digit0;
/// assert_eq!(digit0::<_, (_, ErrorKind), true>(Streaming("21c")), Ok((Streaming("c"), "21")));
/// assert_eq!(digit0::<_, (_, ErrorKind), true>(Streaming("a21c")), Ok((Streaming("a21c"), "")));
/// assert_eq!(digit0::<_, (_, ErrorKind), true>(Streaming("")), Err(Err::Incomplete(Needed::new(1))));
/// ```
#[inline(always)]
pub fn digit0<T, E: ParseError<T>, const STREAMING: bool>(
  input: T,
) -> IResult<T, <T as IntoOutput>::Output, E>
where
  T: InputTakeAtPosition + InputIsStreaming<STREAMING>,
  T: IntoOutput,
  <T as InputTakeAtPosition>::Item: AsChar,
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
/// *Streaming version*: Will return `Err(nom::Err::Incomplete(_))` if there's not enough input data,
/// or if no terminating token is found (a non digit character).
///
/// # Example
///
/// ```
/// # use nom::{Err, error::{Error, ErrorKind}, IResult, Needed};
/// # use nom::character::digit1;
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
/// # use nom::{Err, error::ErrorKind, IResult, Needed};
/// # use nom::input::Streaming;
/// # use nom::character::digit1;
/// assert_eq!(digit1::<_, (_, ErrorKind), true>(Streaming("21c")), Ok((Streaming("c"), "21")));
/// assert_eq!(digit1::<_, (_, ErrorKind), true>(Streaming("c1")), Err(Err::Error((Streaming("c1"), ErrorKind::Digit))));
/// assert_eq!(digit1::<_, (_, ErrorKind), true>(Streaming("")), Err(Err::Incomplete(Needed::new(1))));
/// ```
///
/// ## Parsing an integer
///
/// You can use `digit1` in combination with [`map_res`] to parse an integer:
///
/// ```
/// # use nom::{Err, error::{Error, ErrorKind}, IResult, Needed};
/// # use nom::combinator::map_res;
/// # use nom::character::digit1;
/// fn parser(input: &str) -> IResult<&str, u32> {
///   map_res(digit1, str::parse)(input)
/// }
///
/// assert_eq!(parser("416"), Ok(("", 416)));
/// assert_eq!(parser("12b"), Ok(("b", 12)));
/// assert!(parser("b").is_err());
/// ```
///
/// [`map_res`]: crate::combinator::map_res
#[inline(always)]
pub fn digit1<T, E: ParseError<T>, const STREAMING: bool>(
  input: T,
) -> IResult<T, <T as IntoOutput>::Output, E>
where
  T: InputTakeAtPosition + InputIsStreaming<STREAMING>,
  T: IntoOutput,
  <T as InputTakeAtPosition>::Item: AsChar,
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
/// *Streaming version*: Will return `Err(nom::Err::Incomplete(_))` if there's not enough input data,
/// or if no terminating token is found (a non hexadecimal digit character).
///
/// # Example
///
/// ```
/// # use nom::{Err, error::ErrorKind, IResult, Needed};
/// # use nom::character::hex_digit0;
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
/// # use nom::{Err, error::ErrorKind, IResult, Needed};
/// # use nom::input::Streaming;
/// # use nom::character::hex_digit0;
/// assert_eq!(hex_digit0::<_, (_, ErrorKind), true>(Streaming("21cZ")), Ok((Streaming("Z"), "21c")));
/// assert_eq!(hex_digit0::<_, (_, ErrorKind), true>(Streaming("Z21c")), Ok((Streaming("Z21c"), "")));
/// assert_eq!(hex_digit0::<_, (_, ErrorKind), true>(Streaming("")), Err(Err::Incomplete(Needed::new(1))));
/// ```
#[inline(always)]
pub fn hex_digit0<T, E: ParseError<T>, const STREAMING: bool>(
  input: T,
) -> IResult<T, <T as IntoOutput>::Output, E>
where
  T: InputTakeAtPosition + InputIsStreaming<STREAMING>,
  T: IntoOutput,
  <T as InputTakeAtPosition>::Item: AsChar,
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
/// *Streaming version*: Will return `Err(nom::Err::Incomplete(_))` if there's not enough input data,
/// or if no terminating token is found (a non hexadecimal digit character).
///
/// # Example
///
/// ```
/// # use nom::{Err, error::{Error, ErrorKind}, IResult, Needed};
/// # use nom::character::hex_digit1;
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
/// # use nom::{Err, error::ErrorKind, IResult, Needed};
/// # use nom::input::Streaming;
/// # use nom::character::hex_digit1;
/// assert_eq!(hex_digit1::<_, (_, ErrorKind), true>(Streaming("21cZ")), Ok((Streaming("Z"), "21c")));
/// assert_eq!(hex_digit1::<_, (_, ErrorKind), true>(Streaming("H2")), Err(Err::Error((Streaming("H2"), ErrorKind::HexDigit))));
/// assert_eq!(hex_digit1::<_, (_, ErrorKind), true>(Streaming("")), Err(Err::Incomplete(Needed::new(1))));
/// ```
#[inline(always)]
pub fn hex_digit1<T, E: ParseError<T>, const STREAMING: bool>(
  input: T,
) -> IResult<T, <T as IntoOutput>::Output, E>
where
  T: InputTakeAtPosition + InputIsStreaming<STREAMING>,
  T: IntoOutput,
  <T as InputTakeAtPosition>::Item: AsChar,
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
/// *Streaming version*: Will return `Err(nom::Err::Incomplete(_))` if there's not enough input data,
/// or if no terminating token is found (a non octal digit character).
///
/// # Example
///
/// ```
/// # use nom::{Err, error::ErrorKind, IResult, Needed};
/// # use nom::character::oct_digit0;
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
/// # use nom::{Err, error::ErrorKind, IResult, Needed};
/// # use nom::input::Streaming;
/// # use nom::character::oct_digit0;
/// assert_eq!(oct_digit0::<_, (_, ErrorKind), true>(Streaming("21cZ")), Ok((Streaming("cZ"), "21")));
/// assert_eq!(oct_digit0::<_, (_, ErrorKind), true>(Streaming("Z21c")), Ok((Streaming("Z21c"), "")));
/// assert_eq!(oct_digit0::<_, (_, ErrorKind), true>(Streaming("")), Err(Err::Incomplete(Needed::new(1))));
/// ```
#[inline(always)]
pub fn oct_digit0<T, E: ParseError<T>, const STREAMING: bool>(
  input: T,
) -> IResult<T, <T as IntoOutput>::Output, E>
where
  T: InputTakeAtPosition + InputIsStreaming<STREAMING>,
  T: IntoOutput,
  <T as InputTakeAtPosition>::Item: AsChar,
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
/// *Streaming version*: Will return `Err(nom::Err::Incomplete(_))` if there's not enough input data,
/// or if no terminating token is found (a non octal digit character).
///
/// # Example
///
/// ```
/// # use nom::{Err, error::{Error, ErrorKind}, IResult, Needed};
/// # use nom::character::oct_digit1;
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
/// # use nom::{Err, error::ErrorKind, IResult, Needed};
/// # use nom::input::Streaming;
/// # use nom::character::oct_digit1;
/// assert_eq!(oct_digit1::<_, (_, ErrorKind), true>(Streaming("21cZ")), Ok((Streaming("cZ"), "21")));
/// assert_eq!(oct_digit1::<_, (_, ErrorKind), true>(Streaming("H2")), Err(Err::Error((Streaming("H2"), ErrorKind::OctDigit))));
/// assert_eq!(oct_digit1::<_, (_, ErrorKind), true>(Streaming("")), Err(Err::Incomplete(Needed::new(1))));
/// ```
#[inline(always)]
pub fn oct_digit1<T, E: ParseError<T>, const STREAMING: bool>(
  input: T,
) -> IResult<T, <T as IntoOutput>::Output, E>
where
  T: InputTakeAtPosition + InputIsStreaming<STREAMING>,
  T: IntoOutput,
  <T as InputTakeAtPosition>::Item: AsChar,
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
/// *Streaming version*: Will return `Err(nom::Err::Incomplete(_))` if there's not enough input data,
/// or if no terminating token is found (a non alphanumerical character).
///
/// # Example
///
/// ```
/// # use nom::{Err, error::ErrorKind, IResult, Needed};
/// # use nom::character::alphanumeric0;
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
/// # use nom::{Err, error::ErrorKind, IResult, Needed};
/// # use nom::input::Streaming;
/// # use nom::character::alphanumeric0;
/// assert_eq!(alphanumeric0::<_, (_, ErrorKind), true>(Streaming("21cZ%1")), Ok((Streaming("%1"), "21cZ")));
/// assert_eq!(alphanumeric0::<_, (_, ErrorKind), true>(Streaming("&Z21c")), Ok((Streaming("&Z21c"), "")));
/// assert_eq!(alphanumeric0::<_, (_, ErrorKind), true>(Streaming("")), Err(Err::Incomplete(Needed::new(1))));
/// ```
#[inline(always)]
pub fn alphanumeric0<T, E: ParseError<T>, const STREAMING: bool>(
  input: T,
) -> IResult<T, <T as IntoOutput>::Output, E>
where
  T: InputTakeAtPosition + InputIsStreaming<STREAMING>,
  T: IntoOutput,
  <T as InputTakeAtPosition>::Item: AsChar,
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
/// *Streaming version*: Will return `Err(nom::Err::Incomplete(_))` if there's not enough input data,
/// or if no terminating token is found (a non alphanumerical character).
///
/// # Example
///
/// ```
/// # use nom::{Err, error::{Error, ErrorKind}, IResult, Needed};
/// # use nom::character::alphanumeric1;
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
/// # use nom::{Err, error::ErrorKind, IResult, Needed};
/// # use nom::input::Streaming;
/// # use nom::character::alphanumeric1;
/// assert_eq!(alphanumeric1::<_, (_, ErrorKind), true>(Streaming("21cZ%1")), Ok((Streaming("%1"), "21cZ")));
/// assert_eq!(alphanumeric1::<_, (_, ErrorKind), true>(Streaming("&H2")), Err(Err::Error((Streaming("&H2"), ErrorKind::AlphaNumeric))));
/// assert_eq!(alphanumeric1::<_, (_, ErrorKind), true>(Streaming("")), Err(Err::Incomplete(Needed::new(1))));
/// ```
#[inline(always)]
pub fn alphanumeric1<T, E: ParseError<T>, const STREAMING: bool>(
  input: T,
) -> IResult<T, <T as IntoOutput>::Output, E>
where
  T: InputTakeAtPosition + InputIsStreaming<STREAMING>,
  T: IntoOutput,
  <T as InputTakeAtPosition>::Item: AsChar,
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
/// *Streaming version*: Will return `Err(nom::Err::Incomplete(_))` if there's not enough input data,
/// or if no terminating token is found (a non space character).
///
/// # Example
///
/// ```
/// # use nom::{Err, error::ErrorKind, IResult, Needed};
/// # use nom::input::Streaming;
/// # use nom::character::space0;
/// assert_eq!(space0::<_, (_, ErrorKind), true>(Streaming(" \t21c")), Ok((Streaming("21c"), " \t")));
/// assert_eq!(space0::<_, (_, ErrorKind), true>(Streaming("Z21c")), Ok((Streaming("Z21c"), "")));
/// assert_eq!(space0::<_, (_, ErrorKind), true>(Streaming("")), Err(Err::Incomplete(Needed::new(1))));
/// ```
#[inline(always)]
pub fn space0<T, E: ParseError<T>, const STREAMING: bool>(
  input: T,
) -> IResult<T, <T as IntoOutput>::Output, E>
where
  T: InputTakeAtPosition + InputIsStreaming<STREAMING>,
  T: IntoOutput,
  <T as InputTakeAtPosition>::Item: AsChar + Clone,
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
/// *Streaming version*: Will return `Err(nom::Err::Incomplete(_))` if there's not enough input data,
/// or if no terminating token is found (a non space character).
///
/// # Example
///
/// ```
/// # use nom::{Err, error::{Error, ErrorKind}, IResult, Needed};
/// # use nom::character::space1;
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
/// # use nom::{Err, error::ErrorKind, IResult, Needed};
/// # use nom::input::Streaming;
/// # use nom::character::space1;
/// assert_eq!(space1::<_, (_, ErrorKind), true>(Streaming(" \t21c")), Ok((Streaming("21c"), " \t")));
/// assert_eq!(space1::<_, (_, ErrorKind), true>(Streaming("H2")), Err(Err::Error((Streaming("H2"), ErrorKind::Space))));
/// assert_eq!(space1::<_, (_, ErrorKind), true>(Streaming("")), Err(Err::Incomplete(Needed::new(1))));
/// ```
#[inline(always)]
pub fn space1<T, E: ParseError<T>, const STREAMING: bool>(
  input: T,
) -> IResult<T, <T as IntoOutput>::Output, E>
where
  T: InputTakeAtPosition + InputIsStreaming<STREAMING>,
  T: IntoOutput,
  <T as InputTakeAtPosition>::Item: AsChar + Clone,
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
/// *Streaming version*: Will return `Err(nom::Err::Incomplete(_))` if there's not enough input data,
/// or if no terminating token is found (a non space character).
///
/// # Example
///
/// ```
/// # use nom::{Err, error::ErrorKind, IResult, Needed};
/// # use nom::character::multispace0;
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
/// # use nom::{Err, error::ErrorKind, IResult, Needed};
/// # use nom::input::Streaming;
/// # use nom::character::multispace0;
/// assert_eq!(multispace0::<_, (_, ErrorKind), true>(Streaming(" \t\n\r21c")), Ok((Streaming("21c"), " \t\n\r")));
/// assert_eq!(multispace0::<_, (_, ErrorKind), true>(Streaming("Z21c")), Ok((Streaming("Z21c"), "")));
/// assert_eq!(multispace0::<_, (_, ErrorKind), true>(Streaming("")), Err(Err::Incomplete(Needed::new(1))));
/// ```
#[inline(always)]
pub fn multispace0<T, E: ParseError<T>, const STREAMING: bool>(
  input: T,
) -> IResult<T, <T as IntoOutput>::Output, E>
where
  T: InputTakeAtPosition + InputIsStreaming<STREAMING>,
  T: IntoOutput,
  <T as InputTakeAtPosition>::Item: AsChar + Clone,
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
/// *Streaming version*: Will return `Err(nom::Err::Incomplete(_))` if there's not enough input data,
/// or if no terminating token is found (a non space character).
///
/// # Example
///
/// ```
/// # use nom::{Err, error::{Error, ErrorKind}, IResult, Needed};
/// # use nom::character::multispace1;
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
/// # use nom::{Err, error::ErrorKind, IResult, Needed};
/// # use nom::input::Streaming;
/// # use nom::character::multispace1;
/// assert_eq!(multispace1::<_, (_, ErrorKind), true>(Streaming(" \t\n\r21c")), Ok((Streaming("21c"), " \t\n\r")));
/// assert_eq!(multispace1::<_, (_, ErrorKind), true>(Streaming("H2")), Err(Err::Error((Streaming("H2"), ErrorKind::MultiSpace))));
/// assert_eq!(multispace1::<_, (_, ErrorKind), true>(Streaming("")), Err(Err::Incomplete(Needed::new(1))));
/// ```
#[inline(always)]
pub fn multispace1<T, E: ParseError<T>, const STREAMING: bool>(
  input: T,
) -> IResult<T, <T as IntoOutput>::Output, E>
where
  T: InputTakeAtPosition + InputIsStreaming<STREAMING>,
  T: IntoOutput,
  <T as InputTakeAtPosition>::Item: AsChar + Clone,
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
        /// *Streaming version*: Will return `Err(nom::Err::Incomplete(_))` if there's not enough input data.
        #[inline(always)]
        pub fn $t<T, E: ParseError<T>, const STREAMING: bool>(input: T) -> IResult<T, $t, E>
            where
            T: InputIter + Slice<RangeFrom<usize>> + InputLength + InputTake + Clone + InputIsStreaming<STREAMING>,
            T: IntoOutput,
            <T as InputIter>::Item: AsChar,
            T: for <'a> Compare<&'a[u8]>,
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
        /// *Streaming version*: Will return `Err(nom::Err::Incomplete(_))` if there's not enough input data.
        #[inline(always)]
        pub fn $t<T, E: ParseError<T>, const STREAMING: bool>(input: T) -> IResult<T, $t, E>
            where
            T: InputIter + Slice<RangeFrom<usize>> + InputLength + InputIsStreaming<STREAMING>,
            T: IntoOutput,
            <T as InputIter>::Item: AsChar,
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

#[inline]
#[doc(hidden)]
#[deprecated(since = "8.0.0", note = "Replaced with `AsChar::is_alpha`")]
pub fn is_alphabetic(chr: u8) -> bool {
  (chr >= 0x41 && chr <= 0x5A) || (chr >= 0x61 && chr <= 0x7A)
}

#[inline]
#[doc(hidden)]
#[deprecated(since = "8.0.0", note = "Replaced with `AsChar::is_dec_digit`")]
pub fn is_digit(chr: u8) -> bool {
  chr >= 0x30 && chr <= 0x39
}

#[inline]
#[doc(hidden)]
#[deprecated(since = "8.0.0", note = "Replaced with `AsChar::is_hex_digit`")]
pub fn is_hex_digit(chr: u8) -> bool {
  (chr >= 0x30 && chr <= 0x39) || (chr >= 0x41 && chr <= 0x46) || (chr >= 0x61 && chr <= 0x66)
}

#[inline]
#[doc(hidden)]
#[deprecated(since = "8.0.0", note = "Replaced with `AsChar::is_oct_digit`")]
pub fn is_oct_digit(chr: u8) -> bool {
  chr >= 0x30 && chr <= 0x37
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
