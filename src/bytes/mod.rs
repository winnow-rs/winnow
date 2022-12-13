//! Parsers recognizing bytes streams

pub mod complete;
pub mod streaming;
#[cfg(test)]
mod tests;

use crate::error::ParseError;
use crate::input::{
  Compare, FindSubstring, FindToken, InputIsStreaming, InputIter, InputLength, InputTake,
  InputTakeAtPosition, IntoOutput, Slice, ToUsize,
};
use crate::lib::std::ops::RangeFrom;
use crate::{IResult, Parser};

/// Recognizes a pattern
///
/// The input data will be compared to the tag combinator's argument and will return the part of
/// the input that matches the argument
///
/// It will return `Err(Err::Error((_, ErrorKind::Tag)))` if the input doesn't match the pattern
/// # Example
/// ```rust
/// # use nom::{Err, error::{Error, ErrorKind}, Needed, IResult};
/// use nom::bytes::tag;
///
/// fn parser(s: &str) -> IResult<&str, &str> {
///   tag("Hello")(s)
/// }
///
/// assert_eq!(parser("Hello, World!"), Ok((", World!", "Hello")));
/// assert_eq!(parser("Something"), Err(Err::Error(Error::new("Something", ErrorKind::Tag))));
/// assert_eq!(parser(""), Err(Err::Error(Error::new("", ErrorKind::Tag))));
/// ```
///
/// ```rust
/// # use nom::{Err, error::{Error, ErrorKind}, Needed, IResult};
/// # use nom::input::Streaming;
/// use nom::bytes::tag;
///
/// fn parser(s: Streaming<&str>) -> IResult<Streaming<&str>, &str> {
///   tag("Hello")(s)
/// }
///
/// assert_eq!(parser(Streaming("Hello, World!")), Ok((Streaming(", World!"), "Hello")));
/// assert_eq!(parser(Streaming("Something")), Err(Err::Error(Error::new(Streaming("Something"), ErrorKind::Tag))));
/// assert_eq!(parser(Streaming("S")), Err(Err::Error(Error::new(Streaming("S"), ErrorKind::Tag))));
/// assert_eq!(parser(Streaming("H")), Err(Err::Incomplete(Needed::new(4))));
/// ```
#[inline(always)]
pub fn tag<T, Input, Error: ParseError<Input>, const STREAMING: bool>(
  tag: T,
) -> impl Fn(Input) -> IResult<Input, <Input as IntoOutput>::Output, Error>
where
  Input: InputTake + InputLength + Compare<T> + InputIsStreaming<STREAMING>,
  Input: IntoOutput,
  T: InputLength + Clone,
{
  move |i: Input| {
    let t = tag.clone();
    if STREAMING {
      streaming::tag_internal(i, t)
    } else {
      complete::tag_internal(i, t)
    }
  }
}

/// Recognizes a case insensitive pattern.
///
/// The input data will be compared to the tag combinator's argument and will return the part of
/// the input that matches the argument with no regard to case.
///
/// It will return `Err(Err::Error((_, ErrorKind::Tag)))` if the input doesn't match the pattern.
/// # Example
/// ```rust
/// # use nom::{Err, error::{Error, ErrorKind}, Needed, IResult};
/// use nom::bytes::tag_no_case;
///
/// fn parser(s: &str) -> IResult<&str, &str> {
///   tag_no_case("hello")(s)
/// }
///
/// assert_eq!(parser("Hello, World!"), Ok((", World!", "Hello")));
/// assert_eq!(parser("hello, World!"), Ok((", World!", "hello")));
/// assert_eq!(parser("HeLlO, World!"), Ok((", World!", "HeLlO")));
/// assert_eq!(parser("Something"), Err(Err::Error(Error::new("Something", ErrorKind::Tag))));
/// assert_eq!(parser(""), Err(Err::Error(Error::new("", ErrorKind::Tag))));
/// ```
///
/// ```rust
/// # use nom::{Err, error::{Error, ErrorKind}, Needed, IResult};
/// # use nom::input::Streaming;
/// use nom::bytes::tag_no_case;
///
/// fn parser(s: Streaming<&str>) -> IResult<Streaming<&str>, &str> {
///   tag_no_case("hello")(s)
/// }
///
/// assert_eq!(parser(Streaming("Hello, World!")), Ok((Streaming(", World!"), "Hello")));
/// assert_eq!(parser(Streaming("hello, World!")), Ok((Streaming(", World!"), "hello")));
/// assert_eq!(parser(Streaming("HeLlO, World!")), Ok((Streaming(", World!"), "HeLlO")));
/// assert_eq!(parser(Streaming("Something")), Err(Err::Error(Error::new(Streaming("Something"), ErrorKind::Tag))));
/// assert_eq!(parser(Streaming("")), Err(Err::Incomplete(Needed::new(5))));
/// ```
#[inline(always)]
pub fn tag_no_case<T, Input, Error: ParseError<Input>, const STREAMING: bool>(
  tag: T,
) -> impl Fn(Input) -> IResult<Input, <Input as IntoOutput>::Output, Error>
where
  Input: InputTake + InputLength + Compare<T> + InputIsStreaming<STREAMING>,
  Input: IntoOutput,
  T: InputLength + Clone,
{
  move |i: Input| {
    let t = tag.clone();
    if STREAMING {
      streaming::tag_no_case_internal(i, t)
    } else {
      complete::tag_no_case_internal(i, t)
    }
  }
}

/// Parse till certain characters are met.
///
/// The parser will return the longest slice till one of the characters of the combinator's argument are met.
///
/// It doesn't consume the matched character.
///
/// *Complete version*: It will return a `Err::Error(("", ErrorKind::IsNot))` if the pattern wasn't met.
///
/// *Streaming version*: It will return a `Err::Incomplete(Needed::new(1))` if the pattern wasn't met.
///
/// # Example
/// ```rust
/// # use nom::{Err, error::{Error, ErrorKind}, Needed, IResult};
/// use nom::bytes::is_not;
///
/// fn not_space(s: &str) -> IResult<&str, &str> {
///   is_not(" \t\r\n")(s)
/// }
///
/// assert_eq!(not_space("Hello, World!"), Ok((" World!", "Hello,")));
/// assert_eq!(not_space("Sometimes\t"), Ok(("\t", "Sometimes")));
/// assert_eq!(not_space("Nospace"), Ok(("", "Nospace")));
/// assert_eq!(not_space(""), Err(Err::Error(Error::new("", ErrorKind::IsNot))));
/// ```
///
/// ```rust
/// # use nom::{Err, error::ErrorKind, Needed, IResult};
/// # use nom::input::Streaming;
/// use nom::bytes::is_not;
///
/// fn not_space(s: Streaming<&str>) -> IResult<Streaming<&str>, &str> {
///   is_not(" \t\r\n")(s)
/// }
///
/// assert_eq!(not_space(Streaming("Hello, World!")), Ok((Streaming(" World!"), "Hello,")));
/// assert_eq!(not_space(Streaming("Sometimes\t")), Ok((Streaming("\t"), "Sometimes")));
/// assert_eq!(not_space(Streaming("Nospace")), Err(Err::Incomplete(Needed::new(1))));
/// assert_eq!(not_space(Streaming("")), Err(Err::Incomplete(Needed::new(1))));
/// ```
#[inline(always)]
pub fn is_not<T, Input, Error: ParseError<Input>, const STREAMING: bool>(
  arr: T,
) -> impl Fn(Input) -> IResult<Input, <Input as IntoOutput>::Output, Error>
where
  Input: InputTakeAtPosition + InputIsStreaming<STREAMING>,
  Input: IntoOutput,
  T: FindToken<<Input as InputTakeAtPosition>::Item>,
  Input: InputTakeAtPosition,
  T: FindToken<<Input as InputTakeAtPosition>::Item>,
{
  move |i: Input| {
    if STREAMING {
      streaming::is_not_internal(i, &arr)
    } else {
      complete::is_not_internal(i, &arr)
    }
  }
}

/// Returns the longest slice of the matches the pattern.
///
/// The parser will return the longest slice consisting of the characters in provided in the
/// combinator's argument.
///
/// *Complete version*: It will return a `Err(Err::Error((_, ErrorKind::IsA)))` if the pattern wasn't met.
///
/// *Streaming version*: will return a `Err::Incomplete(Needed::new(1))` if the pattern wasn't met
/// or if the pattern reaches the end of the input.
///
/// # Example
/// ```rust
/// # use nom::{Err, error::{Error, ErrorKind}, Needed, IResult};
/// use nom::bytes::is_a;
///
/// fn hex(s: &str) -> IResult<&str, &str> {
///   is_a("1234567890ABCDEF")(s)
/// }
///
/// assert_eq!(hex("123 and voila"), Ok((" and voila", "123")));
/// assert_eq!(hex("DEADBEEF and others"), Ok((" and others", "DEADBEEF")));
/// assert_eq!(hex("BADBABEsomething"), Ok(("something", "BADBABE")));
/// assert_eq!(hex("D15EA5E"), Ok(("", "D15EA5E")));
/// assert_eq!(hex(""), Err(Err::Error(Error::new("", ErrorKind::IsA))));
/// ```
///
/// ```rust
/// # use nom::{Err, error::ErrorKind, Needed, IResult};
/// # use nom::input::Streaming;
/// use nom::bytes::is_a;
///
/// fn hex(s: Streaming<&str>) -> IResult<Streaming<&str>, &str> {
///   is_a("1234567890ABCDEF")(s)
/// }
///
/// assert_eq!(hex(Streaming("123 and voila")), Ok((Streaming(" and voila"), "123")));
/// assert_eq!(hex(Streaming("DEADBEEF and others")), Ok((Streaming(" and others"), "DEADBEEF")));
/// assert_eq!(hex(Streaming("BADBABEsomething")), Ok((Streaming("something"), "BADBABE")));
/// assert_eq!(hex(Streaming("D15EA5E")), Err(Err::Incomplete(Needed::new(1))));
/// assert_eq!(hex(Streaming("")), Err(Err::Incomplete(Needed::new(1))));
/// ```
#[inline(always)]
pub fn is_a<T, Input, Error: ParseError<Input>, const STREAMING: bool>(
  arr: T,
) -> impl Fn(Input) -> IResult<Input, <Input as IntoOutput>::Output, Error>
where
  Input: InputTakeAtPosition + InputIsStreaming<STREAMING>,
  Input: IntoOutput,
  T: FindToken<<Input as InputTakeAtPosition>::Item>,
  Input: InputTakeAtPosition,
  T: FindToken<<Input as InputTakeAtPosition>::Item>,
{
  move |i: Input| {
    if STREAMING {
      streaming::is_a_internal(i, &arr)
    } else {
      complete::is_a_internal(i, &arr)
    }
  }
}

/// Returns the longest input slice (if any) that matches the predicate.
///
/// The parser will return the longest slice that matches the given predicate *(a function that
/// takes the input and returns a bool)*.
///
/// *Streaming version*: will return a `Err::Incomplete(Needed::new(1))` if the pattern reaches the end of the input.
/// # Example
/// ```rust
/// # use nom::{Err, error::ErrorKind, Needed, IResult};
/// use nom::bytes::take_while;
/// use nom::input::AsChar;
///
/// fn alpha(s: &[u8]) -> IResult<&[u8], &[u8]> {
///   take_while(AsChar::is_alpha)(s)
/// }
///
/// assert_eq!(alpha(b"latin123"), Ok((&b"123"[..], &b"latin"[..])));
/// assert_eq!(alpha(b"12345"), Ok((&b"12345"[..], &b""[..])));
/// assert_eq!(alpha(b"latin"), Ok((&b""[..], &b"latin"[..])));
/// assert_eq!(alpha(b""), Ok((&b""[..], &b""[..])));
/// ```
///
/// ```rust
/// # use nom::{Err, error::ErrorKind, Needed, IResult};
/// # use nom::input::Streaming;
/// use nom::bytes::take_while;
/// use nom::input::AsChar;
///
/// fn alpha(s: Streaming<&[u8]>) -> IResult<Streaming<&[u8]>, &[u8]> {
///   take_while(AsChar::is_alpha)(s)
/// }
///
/// assert_eq!(alpha(Streaming(b"latin123")), Ok((Streaming(&b"123"[..]), &b"latin"[..])));
/// assert_eq!(alpha(Streaming(b"12345")), Ok((Streaming(&b"12345"[..]), &b""[..])));
/// assert_eq!(alpha(Streaming(b"latin")), Err(Err::Incomplete(Needed::new(1))));
/// assert_eq!(alpha(Streaming(b"")), Err(Err::Incomplete(Needed::new(1))));
/// ```
#[inline(always)]
pub fn take_while<F, Input, Error: ParseError<Input>, const STREAMING: bool>(
  cond: F,
) -> impl Fn(Input) -> IResult<Input, <Input as IntoOutput>::Output, Error>
where
  Input: InputTakeAtPosition + InputIsStreaming<STREAMING>,
  Input: IntoOutput,
  F: Fn(<Input as InputTakeAtPosition>::Item) -> bool,
  Input: InputTakeAtPosition,
  F: Fn(<Input as InputTakeAtPosition>::Item) -> bool,
{
  move |i: Input| {
    if STREAMING {
      streaming::take_while_internal(i, &cond)
    } else {
      complete::take_while_internal(i, &cond)
    }
  }
}

/// Returns the longest (at least 1) input slice that matches the predicate.
///
/// The parser will return the longest slice that matches the given predicate *(a function that
/// takes the input and returns a bool)*.
///
/// It will return an `Err(Err::Error((_, ErrorKind::TakeWhile1)))` if the pattern wasn't met.
///
/// *Streaming version* will return a `Err::Incomplete(Needed::new(1))` or if the pattern reaches the end of the input.
///
/// # Example
/// ```rust
/// # use nom::{Err, error::{Error, ErrorKind}, Needed, IResult};
/// use nom::bytes::take_while1;
/// use nom::input::AsChar;
///
/// fn alpha(s: &[u8]) -> IResult<&[u8], &[u8]> {
///   take_while1(AsChar::is_alpha)(s)
/// }
///
/// assert_eq!(alpha(b"latin123"), Ok((&b"123"[..], &b"latin"[..])));
/// assert_eq!(alpha(b"latin"), Ok((&b""[..], &b"latin"[..])));
/// assert_eq!(alpha(b"12345"), Err(Err::Error(Error::new(&b"12345"[..], ErrorKind::TakeWhile1))));
/// ```
///
/// ```rust
/// # use nom::{Err, error::{Error, ErrorKind}, Needed, IResult};
/// # use nom::input::Streaming;
/// use nom::bytes::take_while1;
/// use nom::input::AsChar;
///
/// fn alpha(s: Streaming<&[u8]>) -> IResult<Streaming<&[u8]>, &[u8]> {
///   take_while1(AsChar::is_alpha)(s)
/// }
///
/// assert_eq!(alpha(Streaming(b"latin123")), Ok((Streaming(&b"123"[..]), &b"latin"[..])));
/// assert_eq!(alpha(Streaming(b"latin")), Err(Err::Incomplete(Needed::new(1))));
/// assert_eq!(alpha(Streaming(b"12345")), Err(Err::Error(Error::new(Streaming(&b"12345"[..]), ErrorKind::TakeWhile1))));
/// ```
#[inline(always)]
pub fn take_while1<F, Input, Error: ParseError<Input>, const STREAMING: bool>(
  cond: F,
) -> impl Fn(Input) -> IResult<Input, <Input as IntoOutput>::Output, Error>
where
  Input: InputTakeAtPosition + InputIsStreaming<STREAMING>,
  Input: IntoOutput,
  F: Fn(<Input as InputTakeAtPosition>::Item) -> bool,
{
  move |i: Input| {
    if STREAMING {
      streaming::take_while1_internal(i, &cond)
    } else {
      complete::take_while1_internal(i, &cond)
    }
  }
}

/// Returns the longest (m <= len <= n) input slice  that matches the predicate.
///
/// The parser will return the longest slice that matches the given predicate *(a function that
/// takes the input and returns a bool)*.
///
/// It will return an `Err::Error((_, ErrorKind::TakeWhileMN))` if the pattern wasn't met or is out
/// of range (m <= len <= n).
///
/// *Streaming version* will return a `Err::Incomplete(Needed::new(1))`  if the pattern reaches the end of the input or is too short.
///
/// # Example
/// ```rust
/// # use nom::{Err, error::{Error, ErrorKind}, Needed, IResult};
/// use nom::bytes::take_while_m_n;
/// use nom::input::AsChar;
///
/// fn short_alpha(s: &[u8]) -> IResult<&[u8], &[u8]> {
///   take_while_m_n(3, 6, AsChar::is_alpha)(s)
/// }
///
/// assert_eq!(short_alpha(b"latin123"), Ok((&b"123"[..], &b"latin"[..])));
/// assert_eq!(short_alpha(b"lengthy"), Ok((&b"y"[..], &b"length"[..])));
/// assert_eq!(short_alpha(b"latin"), Ok((&b""[..], &b"latin"[..])));
/// assert_eq!(short_alpha(b"ed"), Err(Err::Error(Error::new(&b"ed"[..], ErrorKind::TakeWhileMN))));
/// assert_eq!(short_alpha(b"12345"), Err(Err::Error(Error::new(&b"12345"[..], ErrorKind::TakeWhileMN))));
/// ```
///
/// ```rust
/// # use nom::{Err, error::{Error, ErrorKind}, Needed, IResult};
/// # use nom::input::Streaming;
/// use nom::bytes::take_while_m_n;
/// use nom::input::AsChar;
///
/// fn short_alpha(s: Streaming<&[u8]>) -> IResult<Streaming<&[u8]>, &[u8]> {
///   take_while_m_n(3, 6, AsChar::is_alpha)(s)
/// }
///
/// assert_eq!(short_alpha(Streaming(b"latin123")), Ok((Streaming(&b"123"[..]), &b"latin"[..])));
/// assert_eq!(short_alpha(Streaming(b"lengthy")), Ok((Streaming(&b"y"[..]), &b"length"[..])));
/// assert_eq!(short_alpha(Streaming(b"latin")), Err(Err::Incomplete(Needed::new(1))));
/// assert_eq!(short_alpha(Streaming(b"ed")), Err(Err::Incomplete(Needed::new(1))));
/// assert_eq!(short_alpha(Streaming(b"12345")), Err(Err::Error(Error::new(Streaming(&b"12345"[..]), ErrorKind::TakeWhileMN))));
/// ```
#[inline(always)]
pub fn take_while_m_n<F, Input, Error: ParseError<Input>, const STREAMING: bool>(
  m: usize,
  n: usize,
  cond: F,
) -> impl Fn(Input) -> IResult<Input, <Input as IntoOutput>::Output, Error>
where
  Input:
    InputTake + InputIter + InputLength + Slice<RangeFrom<usize>> + InputIsStreaming<STREAMING>,
  Input: IntoOutput,
  F: Fn(<Input as InputIter>::Item) -> bool,
{
  move |i: Input| {
    if STREAMING {
      streaming::take_while_m_n_internal(i, m, n, &cond)
    } else {
      complete::take_while_m_n_internal(i, m, n, &cond)
    }
  }
}

/// Returns the longest input slice (if any) till a predicate is met.
///
/// The parser will return the longest slice till the given predicate *(a function that
/// takes the input and returns a bool)*.
///
/// *Streaming version* will return a `Err::Incomplete(Needed::new(1))` if the match reaches the
/// end of input or if there was not match.
///
/// # Example
/// ```rust
/// # use nom::{Err, error::ErrorKind, Needed, IResult};
/// use nom::bytes::take_till;
///
/// fn till_colon(s: &str) -> IResult<&str, &str> {
///   take_till(|c| c == ':')(s)
/// }
///
/// assert_eq!(till_colon("latin:123"), Ok((":123", "latin")));
/// assert_eq!(till_colon(":empty matched"), Ok((":empty matched", ""))); //allowed
/// assert_eq!(till_colon("12345"), Ok(("", "12345")));
/// assert_eq!(till_colon(""), Ok(("", "")));
/// ```
///
/// ```rust
/// # use nom::{Err, error::ErrorKind, Needed, IResult};
/// # use nom::input::Streaming;
/// use nom::bytes::take_till;
///
/// fn till_colon(s: Streaming<&str>) -> IResult<Streaming<&str>, &str> {
///   take_till(|c| c == ':')(s)
/// }
///
/// assert_eq!(till_colon(Streaming("latin:123")), Ok((Streaming(":123"), "latin")));
/// assert_eq!(till_colon(Streaming(":empty matched")), Ok((Streaming(":empty matched"), ""))); //allowed
/// assert_eq!(till_colon(Streaming("12345")), Err(Err::Incomplete(Needed::new(1))));
/// assert_eq!(till_colon(Streaming("")), Err(Err::Incomplete(Needed::new(1))));
/// ```
#[inline(always)]
pub fn take_till<F, Input, Error: ParseError<Input>, const STREAMING: bool>(
  cond: F,
) -> impl Fn(Input) -> IResult<Input, <Input as IntoOutput>::Output, Error>
where
  Input: InputTakeAtPosition + InputIsStreaming<STREAMING>,
  Input: IntoOutput,
  F: Fn(<Input as InputTakeAtPosition>::Item) -> bool,
{
  move |i: Input| {
    if STREAMING {
      streaming::take_till_internal(i, &cond)
    } else {
      complete::take_till_internal(i, &cond)
    }
  }
}

/// Returns the longest (at least 1) input slice till a predicate is met.
///
/// The parser will return the longest slice till the given predicate *(a function that
/// takes the input and returns a bool)*.
///
/// It will return `Err(Err::Error((_, ErrorKind::TakeTill1)))` if the input is empty or the
/// predicate matches the first input.
///
/// *Streaming version* will return a `Err::Incomplete(Needed::new(1))` if the match reaches the
/// end of input or if there was not match.
///
/// # Example
/// ```rust
/// # use nom::{Err, error::{Error, ErrorKind}, Needed, IResult};
/// use nom::bytes::take_till1;
///
/// fn till_colon(s: &str) -> IResult<&str, &str> {
///   take_till1(|c| c == ':')(s)
/// }
///
/// assert_eq!(till_colon("latin:123"), Ok((":123", "latin")));
/// assert_eq!(till_colon(":empty matched"), Err(Err::Error(Error::new(":empty matched", ErrorKind::TakeTill1))));
/// assert_eq!(till_colon("12345"), Ok(("", "12345")));
/// assert_eq!(till_colon(""), Err(Err::Error(Error::new("", ErrorKind::TakeTill1))));
/// ```
///
/// ```rust
/// # use nom::{Err, error::{Error, ErrorKind}, Needed, IResult};
/// # use nom::input::Streaming;
/// use nom::bytes::take_till1;
///
/// fn till_colon(s: Streaming<&str>) -> IResult<Streaming<&str>, &str> {
///   take_till1(|c| c == ':')(s)
/// }
///
/// assert_eq!(till_colon(Streaming("latin:123")), Ok((Streaming(":123"), "latin")));
/// assert_eq!(till_colon(Streaming(":empty matched")), Err(Err::Error(Error::new(Streaming(":empty matched"), ErrorKind::TakeTill1))));
/// assert_eq!(till_colon(Streaming("12345")), Err(Err::Incomplete(Needed::new(1))));
/// assert_eq!(till_colon(Streaming("")), Err(Err::Incomplete(Needed::new(1))));
/// ```
#[inline(always)]
pub fn take_till1<F, Input, Error: ParseError<Input>, const STREAMING: bool>(
  cond: F,
) -> impl Fn(Input) -> IResult<Input, <Input as IntoOutput>::Output, Error>
where
  Input: InputTakeAtPosition + InputIsStreaming<STREAMING>,
  Input: IntoOutput,
  F: Fn(<Input as InputTakeAtPosition>::Item) -> bool,
{
  move |i: Input| {
    if STREAMING {
      streaming::take_till1_internal(i, &cond)
    } else {
      complete::take_till1_internal(i, &cond)
    }
  }
}

/// Returns an input slice containing the first N input elements (Input[..N]).
///
/// *Complete version*: It will return `Err(Err::Error((_, ErrorKind::Eof)))` if the input is shorter than the argument.
///
/// *Streaming version*: if the input has less than N elements, `take` will
/// return a `Err::Incomplete(Needed::new(M))` where M is the number of
/// additional bytes the parser would need to succeed.
/// It is well defined for `&[u8]` as the number of elements is the byte size,
/// but for types like `&str`, we cannot know how many bytes correspond for
/// the next few chars, so the result will be `Err::Incomplete(Needed::Unknown)`
///
/// # Example
/// ```rust
/// # use nom::{Err, error::{Error, ErrorKind}, Needed, IResult};
/// use nom::bytes::take;
///
/// fn take6(s: &str) -> IResult<&str, &str> {
///   take(6usize)(s)
/// }
///
/// assert_eq!(take6("1234567"), Ok(("7", "123456")));
/// assert_eq!(take6("things"), Ok(("", "things")));
/// assert_eq!(take6("short"), Err(Err::Error(Error::new("short", ErrorKind::Eof))));
/// assert_eq!(take6(""), Err(Err::Error(Error::new("", ErrorKind::Eof))));
/// ```
///
/// The units that are taken will depend on the input type. For example, for a
/// `&str` it will take a number of `char`'s, whereas for a `&[u8]` it will
/// take that many `u8`'s:
///
/// ```rust
/// use nom::error::Error;
/// use nom::bytes::take;
///
/// assert_eq!(take::<_, _, Error<_>, false>(1usize)("💙"), Ok(("", "💙")));
/// assert_eq!(take::<_, _, Error<_>, false>(1usize)("💙".as_bytes()), Ok((b"\x9F\x92\x99".as_ref(), b"\xF0".as_ref())));
/// ```
///
/// ```rust
/// # use nom::{Err, error::ErrorKind, Needed, IResult};
/// # use nom::input::Streaming;
/// use nom::bytes::take;
///
/// fn take6(s: Streaming<&str>) -> IResult<Streaming<&str>, &str> {
///   take(6usize)(s)
/// }
///
/// assert_eq!(take6(Streaming("1234567")), Ok((Streaming("7"), "123456")));
/// assert_eq!(take6(Streaming("things")), Ok((Streaming(""), "things")));
/// // `Unknown` as we don't know the number of bytes that `count` corresponds to
/// assert_eq!(take6(Streaming("short")), Err(Err::Incomplete(Needed::Unknown)));
/// ```
#[inline(always)]
pub fn take<C, Input, Error: ParseError<Input>, const STREAMING: bool>(
  count: C,
) -> impl Fn(Input) -> IResult<Input, <Input as IntoOutput>::Output, Error>
where
  Input: InputIter + InputLength + InputTake + InputIsStreaming<STREAMING>,
  Input: IntoOutput,
  C: ToUsize,
{
  let c = count.to_usize();
  move |i: Input| {
    if STREAMING {
      streaming::take_internal(i, c)
    } else {
      complete::take_internal(i, c)
    }
  }
}

/// Returns the input slice up to the first occurrence of the pattern.
///
/// It doesn't consume the pattern.
///
/// *Complete version*: It will return `Err(Err::Error((_, ErrorKind::TakeUntil)))`
/// if the pattern wasn't met.
///
/// *Streaming version*: will return a `Err::Incomplete(Needed::new(N))` if the input doesn't
/// contain the pattern or if the input is smaller than the pattern.
/// # Example
/// ```rust
/// # use nom::{Err, error::{Error, ErrorKind}, Needed, IResult};
/// use nom::bytes::take_until;
///
/// fn until_eof(s: &str) -> IResult<&str, &str> {
///   take_until("eof")(s)
/// }
///
/// assert_eq!(until_eof("hello, worldeof"), Ok(("eof", "hello, world")));
/// assert_eq!(until_eof("hello, world"), Err(Err::Error(Error::new("hello, world", ErrorKind::TakeUntil))));
/// assert_eq!(until_eof(""), Err(Err::Error(Error::new("", ErrorKind::TakeUntil))));
/// assert_eq!(until_eof("1eof2eof"), Ok(("eof2eof", "1")));
/// ```
///
/// ```rust
/// # use nom::{Err, error::ErrorKind, Needed, IResult};
/// # use nom::input::Streaming;
/// use nom::bytes::take_until;
///
/// fn until_eof(s: Streaming<&str>) -> IResult<Streaming<&str>, &str> {
///   take_until("eof")(s)
/// }
///
/// assert_eq!(until_eof(Streaming("hello, worldeof")), Ok((Streaming("eof"), "hello, world")));
/// assert_eq!(until_eof(Streaming("hello, world")), Err(Err::Incomplete(Needed::Unknown)));
/// assert_eq!(until_eof(Streaming("hello, worldeo")), Err(Err::Incomplete(Needed::Unknown)));
/// assert_eq!(until_eof(Streaming("1eof2eof")), Ok((Streaming("eof2eof"), "1")));
/// ```
#[inline(always)]
pub fn take_until<T, Input, Error: ParseError<Input>, const STREAMING: bool>(
  tag: T,
) -> impl Fn(Input) -> IResult<Input, <Input as IntoOutput>::Output, Error>
where
  Input: InputTake + InputLength + FindSubstring<T> + InputIsStreaming<STREAMING>,
  Input: IntoOutput,
  T: InputLength + Clone,
{
  move |i: Input| {
    if STREAMING {
      streaming::take_until_internal(i, tag.clone())
    } else {
      complete::take_until_internal(i, tag.clone())
    }
  }
}

/// Returns the non empty input slice up to the first occurrence of the pattern.
///
/// It doesn't consume the pattern.
///
/// *Complete version*: It will return `Err(Err::Error((_, ErrorKind::TakeUntil)))`
/// if the pattern wasn't met.
///
/// *Streaming version*: will return a `Err::Incomplete(Needed::new(N))` if the input doesn't
/// contain the pattern or if the input is smaller than the pattern.
///
/// # Example
/// ```rust
/// # use nom::{Err, error::{Error, ErrorKind}, Needed, IResult};
/// use nom::bytes::take_until1;
///
/// fn until_eof(s: &str) -> IResult<&str, &str> {
///   take_until1("eof")(s)
/// }
///
/// assert_eq!(until_eof("hello, worldeof"), Ok(("eof", "hello, world")));
/// assert_eq!(until_eof("hello, world"), Err(Err::Error(Error::new("hello, world", ErrorKind::TakeUntil))));
/// assert_eq!(until_eof(""), Err(Err::Error(Error::new("", ErrorKind::TakeUntil))));
/// assert_eq!(until_eof("1eof2eof"), Ok(("eof2eof", "1")));
/// assert_eq!(until_eof("eof"), Err(Err::Error(Error::new("eof", ErrorKind::TakeUntil))));
/// ```
///
/// ```rust
/// # use nom::{Err, error::{Error, ErrorKind}, Needed, IResult};
/// # use nom::input::Streaming;
/// use nom::bytes::take_until1;
///
/// fn until_eof(s: Streaming<&str>) -> IResult<Streaming<&str>, &str> {
///   take_until1("eof")(s)
/// }
///
/// assert_eq!(until_eof(Streaming("hello, worldeof")), Ok((Streaming("eof"), "hello, world")));
/// assert_eq!(until_eof(Streaming("hello, world")), Err(Err::Incomplete(Needed::Unknown)));
/// assert_eq!(until_eof(Streaming("hello, worldeo")), Err(Err::Incomplete(Needed::Unknown)));
/// assert_eq!(until_eof(Streaming("1eof2eof")), Ok((Streaming("eof2eof"), "1")));
/// assert_eq!(until_eof(Streaming("eof")),  Err(Err::Error(Error::new(Streaming("eof"), ErrorKind::TakeUntil))));
/// ```
#[inline(always)]
pub fn take_until1<T, Input, Error: ParseError<Input>, const STREAMING: bool>(
  tag: T,
) -> impl Fn(Input) -> IResult<Input, <Input as IntoOutput>::Output, Error>
where
  Input: InputTake + InputLength + FindSubstring<T> + InputIsStreaming<STREAMING>,
  Input: IntoOutput,
  T: InputLength + Clone,
{
  move |i: Input| {
    if STREAMING {
      streaming::take_until1_internal(i, tag.clone())
    } else {
      complete::take_until1_internal(i, tag.clone())
    }
  }
}

/// Matches a byte string with escaped characters.
///
/// * The first argument matches the normal characters (it must not accept the control character)
/// * The second argument is the control character (like `\` in most languages)
/// * The third argument matches the escaped characters
/// # Example
/// ```
/// # use nom::{Err, error::ErrorKind, Needed, IResult};
/// # use nom::character::digit1;
/// use nom::bytes::escaped;
/// use nom::character::one_of;
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
/// # use nom::{Err, error::ErrorKind, Needed, IResult};
/// # use nom::character::digit1;
/// # use nom::input::Streaming;
/// use nom::bytes::escaped;
/// use nom::character::one_of;
///
/// fn esc(s: Streaming<&str>) -> IResult<Streaming<&str>, &str> {
///   escaped(digit1, '\\', one_of("\"n\\"))(s)
/// }
///
/// assert_eq!(esc(Streaming("123;")), Ok((Streaming(";"), "123")));
/// assert_eq!(esc(Streaming("12\\\"34;")), Ok((Streaming(";"), "12\\\"34")));
/// ```
#[inline(always)]
pub fn escaped<'a, Input: 'a, Error, F, G, O1, O2, const STREAMING: bool>(
  mut normal: F,
  control_char: char,
  mut escapable: G,
) -> impl FnMut(Input) -> IResult<Input, <Input as IntoOutput>::Output, Error>
where
  Input: Clone
    + crate::input::Offset
    + InputLength
    + InputTake
    + InputTakeAtPosition
    + Slice<RangeFrom<usize>>
    + InputIter
    + InputIsStreaming<STREAMING>,
  Input: IntoOutput,
  <Input as InputIter>::Item: crate::input::AsChar,
  F: Parser<Input, O1, Error>,
  G: Parser<Input, O2, Error>,
  Error: ParseError<Input>,
{
  move |input: Input| {
    if STREAMING {
      streaming::escaped_internal(input, &mut normal, control_char, &mut escapable)
    } else {
      complete::escaped_internal(input, &mut normal, control_char, &mut escapable)
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
/// # use nom::{Err, error::ErrorKind, Needed, IResult};
/// # use std::str::from_utf8;
/// use nom::bytes::{escaped_transform, tag};
/// use nom::character::alpha1;
/// use nom::branch::alt;
/// use nom::combinator::value;
///
/// fn parser(input: &str) -> IResult<&str, String> {
///   escaped_transform(
///     alpha1,
///     '\\',
///     alt((
///       value("\\", tag("\\")),
///       value("\"", tag("\"")),
///       value("\n", tag("n")),
///     ))
///   )(input)
/// }
///
/// assert_eq!(parser("ab\\\"cd"), Ok(("", String::from("ab\"cd"))));
/// assert_eq!(parser("ab\\ncd"), Ok(("", String::from("ab\ncd"))));
/// ```
///
/// ```
/// # use nom::{Err, error::ErrorKind, Needed, IResult};
/// # use std::str::from_utf8;
/// # use nom::input::Streaming;
/// use nom::bytes::{escaped_transform, tag};
/// use nom::character::alpha1;
/// use nom::branch::alt;
/// use nom::combinator::value;
///
/// fn parser(input: Streaming<&str>) -> IResult<Streaming<&str>, String> {
///   escaped_transform(
///     alpha1,
///     '\\',
///     alt((
///       value("\\", tag("\\")),
///       value("\"", tag("\"")),
///       value("\n", tag("n")),
///     ))
///   )(input)
/// }
///
/// assert_eq!(parser(Streaming("ab\\\"cd\"")), Ok((Streaming("\""), String::from("ab\"cd"))));
/// ```
#[cfg(feature = "alloc")]
#[inline(always)]
pub fn escaped_transform<Input, Error, F, G, O1, O2, ExtendItem, Output, const STREAMING: bool>(
  mut normal: F,
  control_char: char,
  mut transform: G,
) -> impl FnMut(Input) -> IResult<Input, Output, Error>
where
  Input: Clone
    + crate::input::Offset
    + InputLength
    + InputTake
    + InputTakeAtPosition
    + Slice<RangeFrom<usize>>
    + InputIter
    + InputIsStreaming<STREAMING>,
  Input: IntoOutput,
  Input: crate::input::ExtendInto<Item = ExtendItem, Extender = Output>,
  O1: crate::input::ExtendInto<Item = ExtendItem, Extender = Output>,
  O2: crate::input::ExtendInto<Item = ExtendItem, Extender = Output>,
  <Input as InputIter>::Item: crate::input::AsChar,
  F: Parser<Input, O1, Error>,
  G: Parser<Input, O2, Error>,
  Error: ParseError<Input>,
{
  move |input: Input| {
    if STREAMING {
      streaming::escaped_transform_internal(input, &mut normal, control_char, &mut transform)
    } else {
      complete::escaped_transform_internal(input, &mut normal, control_char, &mut transform)
    }
  }
}
