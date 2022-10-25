//! Parsers recognizing bytes streams, streaming version

use crate::error::ErrorKind;
use crate::error::ParseError;
use crate::internal::{Err, IResult, Needed, Parser};
use crate::lib::std::ops::RangeFrom;
use crate::lib::std::result::Result::*;
use crate::traits::{
  Compare, CompareResult, FindSubstring, FindToken, InputIter, InputLength, InputTake,
  InputTakeAtPositionPartial, Slice, ToUsize,
};

/// Recognizes a pattern
///
/// The input data will be compared to the tag combinator's argument and will return the part of
/// the input that matches the argument.
/// # Example
/// ```rust
/// # use nom::{Err, error::{Error, ErrorKind}, Needed, IResult};
/// use nom::bytes::streaming::tag;
///
/// fn parser(s: &str) -> IResult<&str, &str> {
///   tag("Hello").parse(s)
/// }
///
/// assert_eq!(parser("Hello, World!"), Ok((", World!", "Hello")));
/// assert_eq!(parser("Something"), Err(Err::Error(Error::new("Something", ErrorKind::Tag))));
/// assert_eq!(parser("S"), Err(Err::Error(Error::new("S", ErrorKind::Tag))));
/// assert_eq!(parser("H"), Err(Err::Incomplete(Needed::new(4))));
/// ```
pub fn tag<T, I, E>(tag: T) -> Tag<T, I, E> {
  Tag {
    tag,
    i: Default::default(),
    e: Default::default(),
  }
}

/// Implementation of [`tag`]
pub struct Tag<T, I, E> {
  tag: T,
  i: core::marker::PhantomData<I>,
  e: core::marker::PhantomData<E>,
}

impl<T, I, E> Tag<T, I, E>
where
  T: InputLength + Clone,
  I: InputTake + InputLength + Compare<T>,
  E: ParseError<I>,
{
  /// See [`Parser::parse`]
  pub fn parse(&mut self, input: I) -> IResult<I, I, E> {
    let tag_len = self.tag.input_len();
    let t = self.tag.clone();
    let res: IResult<_, _, E> = match input.compare(t) {
      CompareResult::Ok => Ok(input.take_split(tag_len)),
      CompareResult::Incomplete => Err(Err::Incomplete(Needed::new(tag_len - input.input_len()))),
      CompareResult::Error => {
        let e: ErrorKind = ErrorKind::Tag;
        Err(Err::Error(E::from_error_kind(input, e)))
      }
    };
    res
  }
}

impl<T, I, E> Parser<I, I, E> for Tag<T, I, E>
where
  T: InputLength + Clone,
  I: InputTake + InputLength + Compare<T>,
  E: ParseError<I>,
{
  fn parse(&mut self, input: I) -> IResult<I, I, E> {
    self.parse(input)
  }
}

/// Recognizes a case insensitive pattern.
///
/// The input data will be compared to the tag combinator's argument and will return the part of
/// the input that matches the argument with no regard to case.
/// # Example
/// ```rust
/// # use nom::{Err, error::{Error, ErrorKind}, Needed, IResult};
/// use nom::bytes::streaming::tag_no_case;
///
/// fn parser(s: &str) -> IResult<&str, &str> {
///   tag_no_case("hello").parse(s)
/// }
///
/// assert_eq!(parser("Hello, World!"), Ok((", World!", "Hello")));
/// assert_eq!(parser("hello, World!"), Ok((", World!", "hello")));
/// assert_eq!(parser("HeLlO, World!"), Ok((", World!", "HeLlO")));
/// assert_eq!(parser("Something"), Err(Err::Error(Error::new("Something", ErrorKind::Tag))));
/// assert_eq!(parser(""), Err(Err::Incomplete(Needed::new(5))));
/// ```
pub fn tag_no_case<T, I, E>(tag: T) -> TagNoCase<T, I, E> {
  TagNoCase {
    tag,
    i: Default::default(),
    e: Default::default(),
  }
}

/// Implementation of [`tag_no_case`]
pub struct TagNoCase<T, I, E> {
  tag: T,
  i: core::marker::PhantomData<I>,
  e: core::marker::PhantomData<E>,
}

impl<T, I, E> TagNoCase<T, I, E>
where
  T: InputLength + Clone,
  I: InputTake + InputLength + Compare<T>,
  E: ParseError<I>,
{
  /// See [`Parser::parse`]
  pub fn parse(&mut self, input: I) -> IResult<I, I, E> {
    let tag_len = self.tag.input_len();
    let t = self.tag.clone();
    let res: IResult<_, _, E> = match input.compare_no_case(t) {
      CompareResult::Ok => Ok(input.take_split(tag_len)),
      CompareResult::Incomplete => Err(Err::Incomplete(Needed::new(tag_len - input.input_len()))),
      CompareResult::Error => {
        let e: ErrorKind = ErrorKind::Tag;
        Err(Err::Error(E::from_error_kind(input, e)))
      }
    };
    res
  }
}

impl<T, I, E> Parser<I, I, E> for TagNoCase<T, I, E>
where
  T: InputLength + Clone,
  I: InputTake + InputLength + Compare<T>,
  E: ParseError<I>,
{
  fn parse(&mut self, input: I) -> IResult<I, I, E> {
    self.parse(input)
  }
}

/// Parse till certain characters are met.
///
/// The parser will return the longest slice till one of the characters of the combinator's argument are met.
///
/// It doesn't consume the matched character.
///
/// It will return a `Err::Incomplete(Needed::new(1))` if the pattern wasn't met.
/// # Example
/// ```rust
/// # use nom::{Err, error::ErrorKind, Needed, IResult};
/// use nom::bytes::streaming::is_not;
///
/// fn not_space(s: &str) -> IResult<&str, &str> {
///   is_not(" \t\r\n").parse(s)
/// }
///
/// assert_eq!(not_space("Hello, World!"), Ok((" World!", "Hello,")));
/// assert_eq!(not_space("Sometimes\t"), Ok(("\t", "Sometimes")));
/// assert_eq!(not_space("Nospace"), Err(Err::Incomplete(Needed::new(1))));
/// assert_eq!(not_space(""), Err(Err::Incomplete(Needed::new(1))));
/// ```
pub fn is_not<T, I, E>(arr: T) -> IsNot<T, I, E> {
  IsNot {
    arr,
    i: Default::default(),
    e: Default::default(),
  }
}

/// Implementation of [`is_not`]
pub struct IsNot<T, I, E> {
  arr: T,
  i: core::marker::PhantomData<I>,
  e: core::marker::PhantomData<E>,
}

impl<T, I, E> IsNot<T, I, E>
where
  T: FindToken<<I as InputTakeAtPositionPartial>::Item>,
  I: InputTakeAtPositionPartial,
  E: ParseError<I>,
{
  /// See [`Parser::parse`]
  pub fn parse(&mut self, input: I) -> IResult<I, I, E> {
    let e: ErrorKind = ErrorKind::IsNot;
    input.split_at_position1_partial(|c| self.arr.find_token(c), e)
  }
}

impl<T, I, E> Parser<I, I, E> for IsNot<T, I, E>
where
  T: FindToken<<I as InputTakeAtPositionPartial>::Item>,
  I: InputTakeAtPositionPartial,
  E: ParseError<I>,
{
  fn parse(&mut self, input: I) -> IResult<I, I, E> {
    self.parse(input)
  }
}

/// Returns the longest slice of the matches the pattern.
///
/// The parser will return the longest slice consisting of the characters in provided in the
/// combinator's argument.
///
/// # Streaming specific
/// *Streaming version* will return a `Err::Incomplete(Needed::new(1))` if the pattern wasn't met
/// or if the pattern reaches the end of the input.
///
/// # Example
/// ```rust
/// # use nom::{Err, error::ErrorKind, Needed, IResult};
/// use nom::bytes::streaming::is_a;
///
/// fn hex(s: &str) -> IResult<&str, &str> {
///   is_a("1234567890ABCDEF").parse(s)
/// }
///
/// assert_eq!(hex("123 and voila"), Ok((" and voila", "123")));
/// assert_eq!(hex("DEADBEEF and others"), Ok((" and others", "DEADBEEF")));
/// assert_eq!(hex("BADBABEsomething"), Ok(("something", "BADBABE")));
/// assert_eq!(hex("D15EA5E"), Err(Err::Incomplete(Needed::new(1))));
/// assert_eq!(hex(""), Err(Err::Incomplete(Needed::new(1))));
/// ```
pub fn is_a<T, I, E>(arr: T) -> IsA<T, I, E> {
  IsA {
    arr,
    i: Default::default(),
    e: Default::default(),
  }
}

/// Implementation of [`is_a`]
pub struct IsA<T, I, E> {
  arr: T,
  i: core::marker::PhantomData<I>,
  e: core::marker::PhantomData<E>,
}

impl<T, I, E> IsA<T, I, E>
where
  T: FindToken<<I as InputTakeAtPositionPartial>::Item>,
  I: InputTakeAtPositionPartial,
  E: ParseError<I>,
{
  /// See [`Parser::parse`]
  pub fn parse(&mut self, input: I) -> IResult<I, I, E> {
    let e: ErrorKind = ErrorKind::IsA;
    input.split_at_position1_partial(|c| !self.arr.find_token(c), e)
  }
}

impl<T, I, E> Parser<I, I, E> for IsA<T, I, E>
where
  T: FindToken<<I as InputTakeAtPositionPartial>::Item>,
  I: InputTakeAtPositionPartial,
  E: ParseError<I>,
{
  fn parse(&mut self, input: I) -> IResult<I, I, E> {
    self.parse(input)
  }
}

/// Returns the longest input slice (if any) that matches the predicate.
///
/// The parser will return the longest slice that matches the given predicate *(a function that
/// takes the input and returns a bool)*.
///
/// # Streaming Specific
/// *Streaming version* will return a `Err::Incomplete(Needed::new(1))` if the pattern reaches the end of the input.
///
/// # Example
/// ```rust
/// # use nom::{Err, error::ErrorKind, Needed, IResult};
/// use nom::bytes::streaming::take_while;
/// use nom::AsChar;
///
/// fn alpha(s: &[u8]) -> IResult<&[u8], &[u8]> {
///   take_while(AsChar::is_alpha).parse(s)
/// }
///
/// assert_eq!(alpha(b"latin123"), Ok((&b"123"[..], &b"latin"[..])));
/// assert_eq!(alpha(b"12345"), Ok((&b"12345"[..], &b""[..])));
/// assert_eq!(alpha(b"latin"), Err(Err::Incomplete(Needed::new(1))));
/// assert_eq!(alpha(b""), Err(Err::Incomplete(Needed::new(1))));
/// ```
pub fn take_while<F, I, E>(pred: F) -> TakeWhile<F, I, E> {
  TakeWhile {
    pred,
    i: Default::default(),
    e: Default::default(),
  }
}

/// Implementation of [`take_while`]
pub struct TakeWhile<F, I, E> {
  pred: F,
  i: core::marker::PhantomData<I>,
  e: core::marker::PhantomData<E>,
}

impl<F, I, E> TakeWhile<F, I, E>
where
  F: Fn(<I as InputTakeAtPositionPartial>::Item) -> bool,
  I: InputTakeAtPositionPartial,
  E: ParseError<I>,
{
  /// See [`Parser::parse`]
  pub fn parse(&mut self, input: I) -> IResult<I, I, E> {
    input.split_at_position_partial(|c| !(self.pred)(c))
  }
}

impl<F, I, E> Parser<I, I, E> for TakeWhile<F, I, E>
where
  F: Fn(<I as InputTakeAtPositionPartial>::Item) -> bool,
  I: InputTakeAtPositionPartial,
  E: ParseError<I>,
{
  fn parse(&mut self, input: I) -> IResult<I, I, E> {
    self.parse(input)
  }
}

/// Returns the longest (at least 1) input slice that matches the predicate.
///
/// The parser will return the longest slice that matches the given predicate *(a function that
/// takes the input and returns a bool)*.
///
/// It will return an `Err(Err::Error((_, ErrorKind::TakeWhile1)))` if the pattern wasn't met.
///
/// # Streaming Specific
/// *Streaming version* will return a `Err::Incomplete(Needed::new(1))` or if the pattern reaches the end of the input.
///
/// # Example
/// ```rust
/// # use nom::{Err, error::{Error, ErrorKind}, Needed, IResult};
/// use nom::bytes::streaming::take_while1;
/// use nom::AsChar;
///
/// fn alpha(s: &[u8]) -> IResult<&[u8], &[u8]> {
///   take_while1(AsChar::is_alpha).parse(s)
/// }
///
/// assert_eq!(alpha(b"latin123"), Ok((&b"123"[..], &b"latin"[..])));
/// assert_eq!(alpha(b"latin"), Err(Err::Incomplete(Needed::new(1))));
/// assert_eq!(alpha(b"12345"), Err(Err::Error(Error::new(&b"12345"[..], ErrorKind::TakeWhile1))));
/// ```
pub fn take_while1<F, I, E>(pred: F) -> TakeWhile1<F, I, E> {
  TakeWhile1 {
    pred,
    i: Default::default(),
    e: Default::default(),
  }
}

/// Implementation of [`take_while1`]
pub struct TakeWhile1<F, I, E> {
  pred: F,
  i: core::marker::PhantomData<I>,
  e: core::marker::PhantomData<E>,
}

impl<F, I, E> TakeWhile1<F, I, E>
where
  F: Fn(<I as InputTakeAtPositionPartial>::Item) -> bool,
  I: InputTakeAtPositionPartial,
  E: ParseError<I>,
{
  /// See [`Parser::parse`]
  pub fn parse(&mut self, input: I) -> IResult<I, I, E> {
    let e: ErrorKind = ErrorKind::TakeWhile1;
    input.split_at_position1_partial(|c| !(self.pred)(c), e)
  }
}

impl<F, I, E> Parser<I, I, E> for TakeWhile1<F, I, E>
where
  F: Fn(<I as InputTakeAtPositionPartial>::Item) -> bool,
  I: InputTakeAtPositionPartial,
  E: ParseError<I>,
{
  fn parse(&mut self, input: I) -> IResult<I, I, E> {
    self.parse(input)
  }
}

/// Returns the longest (m <= len <= n) input slice  that matches the predicate.
///
/// The parser will return the longest slice that matches the given predicate *(a function that
/// takes the input and returns a bool)*.
///
/// It will return an `Err::Error((_, ErrorKind::TakeWhileMN))` if the pattern wasn't met.
///
/// # Streaming Specific
/// *Streaming version* will return a `Err::Incomplete(Needed::new(1))`  if the pattern reaches the end of the input or is too short.
///
/// # Example
/// ```rust
/// # use nom::{Err, error::{Error, ErrorKind}, Needed, IResult};
/// use nom::bytes::streaming::take_while_m_n;
/// use nom::AsChar;
///
/// fn short_alpha(s: &[u8]) -> IResult<&[u8], &[u8]> {
///   take_while_m_n(3, 6, AsChar::is_alpha).parse(s)
/// }
///
/// assert_eq!(short_alpha(b"latin123"), Ok((&b"123"[..], &b"latin"[..])));
/// assert_eq!(short_alpha(b"lengthy"), Ok((&b"y"[..], &b"length"[..])));
/// assert_eq!(short_alpha(b"latin"), Err(Err::Incomplete(Needed::new(1))));
/// assert_eq!(short_alpha(b"ed"), Err(Err::Incomplete(Needed::new(1))));
/// assert_eq!(short_alpha(b"12345"), Err(Err::Error(Error::new(&b"12345"[..], ErrorKind::TakeWhileMN))));
/// ```
pub fn take_while_m_n<F, I, E>(m: usize, n: usize, pred: F) -> TakeWhileMN<F, I, E> {
  TakeWhileMN {
    m,
    n,
    pred,
    i: Default::default(),
    e: Default::default(),
  }
}

/// Implementation of [`take_while_m_n`]
pub struct TakeWhileMN<F, I, E> {
  m: usize,
  n: usize,
  pred: F,
  i: core::marker::PhantomData<I>,
  e: core::marker::PhantomData<E>,
}

impl<F, I, E> TakeWhileMN<F, I, E>
where
  F: Fn(<I as InputIter>::Item) -> bool,
  I: InputTake + InputIter + InputLength + Slice<RangeFrom<usize>>,
  E: ParseError<I>,
{
  /// See [`Parser::parse`]
  pub fn parse(&mut self, input: I) -> IResult<I, I, E> {
    match input.position(|c| !(self.pred)(c)) {
      Some(idx) => {
        if idx >= self.m {
          if idx <= self.n {
            let res: IResult<_, _, E> = if let Ok(index) = input.slice_index(idx) {
              Ok(input.take_split(index))
            } else {
              Err(Err::Error(E::from_error_kind(
                input,
                ErrorKind::TakeWhileMN,
              )))
            };
            res
          } else {
            let res: IResult<_, _, E> = if let Ok(index) = input.slice_index(self.n) {
              Ok(input.take_split(index))
            } else {
              Err(Err::Error(E::from_error_kind(
                input,
                ErrorKind::TakeWhileMN,
              )))
            };
            res
          }
        } else {
          let e = ErrorKind::TakeWhileMN;
          Err(Err::Error(E::from_error_kind(input, e)))
        }
      }
      None => {
        let len = input.input_len();
        if len >= self.n {
          match input.slice_index(self.n) {
            Ok(index) => Ok(input.take_split(index)),
            Err(_needed) => Err(Err::Error(E::from_error_kind(
              input,
              ErrorKind::TakeWhileMN,
            ))),
          }
        } else {
          let needed = if self.m > len { self.m - len } else { 1 };
          Err(Err::Incomplete(Needed::new(needed)))
        }
      }
    }
  }
}

impl<F, I, E> Parser<I, I, E> for TakeWhileMN<F, I, E>
where
  F: Fn(<I as InputIter>::Item) -> bool,
  I: InputTake + InputIter + InputLength + Slice<RangeFrom<usize>>,
  E: ParseError<I>,
{
  fn parse(&mut self, input: I) -> IResult<I, I, E> {
    self.parse(input)
  }
}

/// Returns the longest input slice (if any) till a predicate is met.
///
/// The parser will return the longest slice till the given predicate *(a function that
/// takes the input and returns a bool)*.
///
/// # Streaming Specific
/// *Streaming version* will return a `Err::Incomplete(Needed::new(1))` if the match reaches the
/// end of input or if there was not match.
///
/// # Example
/// ```rust
/// # use nom::{Err, error::ErrorKind, Needed, IResult};
/// use nom::bytes::streaming::take_till;
///
/// fn till_colon(s: &str) -> IResult<&str, &str> {
///   take_till(|c| c == ':').parse(s)
/// }
///
/// assert_eq!(till_colon("latin:123"), Ok((":123", "latin")));
/// assert_eq!(till_colon(":empty matched"), Ok((":empty matched", ""))); //allowed
/// assert_eq!(till_colon("12345"), Err(Err::Incomplete(Needed::new(1))));
/// assert_eq!(till_colon(""), Err(Err::Incomplete(Needed::new(1))));
/// ```
pub fn take_till<F, I, E>(pred: F) -> TakeTill<F, I, E> {
  TakeTill {
    pred,
    i: Default::default(),
    e: Default::default(),
  }
}

/// Implementation of [`take_till`]
pub struct TakeTill<F, I, E> {
  pred: F,
  i: core::marker::PhantomData<I>,
  e: core::marker::PhantomData<E>,
}

impl<F, I, E> TakeTill<F, I, E>
where
  F: Fn(<I as InputTakeAtPositionPartial>::Item) -> bool,
  I: InputTakeAtPositionPartial,
  E: ParseError<I>,
{
  /// See [`Parser::parse`]
  pub fn parse(&mut self, input: I) -> IResult<I, I, E> {
    input.split_at_position_partial(|c| (self.pred)(c))
  }
}

impl<F, I, E> Parser<I, I, E> for TakeTill<F, I, E>
where
  F: Fn(<I as InputTakeAtPositionPartial>::Item) -> bool,
  I: InputTakeAtPositionPartial,
  E: ParseError<I>,
{
  fn parse(&mut self, input: I) -> IResult<I, I, E> {
    self.parse(input)
  }
}

/// Returns the longest (at least 1) input slice till a predicate is met.
///
/// The parser will return the longest slice till the given predicate *(a function that
/// takes the input and returns a bool)*.
///
/// # Streaming Specific
/// *Streaming version* will return a `Err::Incomplete(Needed::new(1))` if the match reaches the
/// end of input or if there was not match.
///
/// # Example
/// ```rust
/// # use nom::{Err, error::{Error, ErrorKind}, Needed, IResult};
/// use nom::bytes::streaming::take_till1;
///
/// fn till_colon(s: &str) -> IResult<&str, &str> {
///   take_till1(|c| c == ':').parse(s)
/// }
///
/// assert_eq!(till_colon("latin:123"), Ok((":123", "latin")));
/// assert_eq!(till_colon(":empty matched"), Err(Err::Error(Error::new(":empty matched", ErrorKind::TakeTill1))));
/// assert_eq!(till_colon("12345"), Err(Err::Incomplete(Needed::new(1))));
/// assert_eq!(till_colon(""), Err(Err::Incomplete(Needed::new(1))));
/// ```
pub fn take_till1<F, I, E>(pred: F) -> TakeTill1<F, I, E> {
  TakeTill1 {
    pred,
    i: Default::default(),
    e: Default::default(),
  }
}

/// Implementation of [`take_till1`]
pub struct TakeTill1<F, I, E> {
  pred: F,
  i: core::marker::PhantomData<I>,
  e: core::marker::PhantomData<E>,
}

impl<F, I, E> TakeTill1<F, I, E>
where
  F: Fn(<I as InputTakeAtPositionPartial>::Item) -> bool,
  I: InputTakeAtPositionPartial,
  E: ParseError<I>,
{
  /// See [`Parser::parse`]
  pub fn parse(&mut self, input: I) -> IResult<I, I, E> {
    let e: ErrorKind = ErrorKind::TakeTill1;
    input.split_at_position1_partial(|c| (self.pred)(c), e)
  }
}

impl<F, I, E> Parser<I, I, E> for TakeTill1<F, I, E>
where
  F: Fn(<I as InputTakeAtPositionPartial>::Item) -> bool,
  I: InputTakeAtPositionPartial,
  E: ParseError<I>,
{
  fn parse(&mut self, input: I) -> IResult<I, I, E> {
    self.parse(input)
  }
}

/// Returns an input slice containing the first N input elements (Input[..N]).
///
/// # Streaming Specific
/// *Streaming version* if the input has less than N elements, `take` will
/// return a `Err::Incomplete(Needed::new(M))` where M is the number of
/// additional bytes the parser would need to succeed.
/// It is well defined for `&[u8]` as the number of elements is the byte size,
/// but for types like `&str`, we cannot know how many bytes correspond for
/// the next few chars, so the result will be `Err::Incomplete(Needed::Unknown)`
///
/// # Example
/// ```rust
/// # use nom::{Err, error::ErrorKind, Needed, IResult};
/// use nom::bytes::streaming::take;
///
/// fn take6(s: &str) -> IResult<&str, &str> {
///   take(6usize).parse(s)
/// }
///
/// assert_eq!(take6("1234567"), Ok(("7", "123456")));
/// assert_eq!(take6("things"), Ok(("", "things")));
/// assert_eq!(take6("short"), Err(Err::Incomplete(Needed::Unknown)));
/// ```
pub fn take<C, I, E>(count: C) -> Take<C, I, E> {
  Take {
    count,
    i: Default::default(),
    e: Default::default(),
  }
}

/// Implementation of [`take`]
pub struct Take<C, I, E> {
  count: C,
  i: core::marker::PhantomData<I>,
  e: core::marker::PhantomData<E>,
}

impl<C, I, E> Take<C, I, E>
where
  C: ToUsize,
  I: InputIter + InputTake,
  E: ParseError<I>,
{
  /// See [`Parser::parse`]
  pub fn parse(&mut self, input: I) -> IResult<I, I, E> {
    let c = self.count.to_usize();
    match input.slice_index(c) {
      Err(needed) => Err(Err::Incomplete(needed)),
      Ok(index) => Ok(input.take_split(index)),
    }
  }
}

impl<C, I, E> Parser<I, I, E> for Take<C, I, E>
where
  C: ToUsize,
  I: InputIter + InputTake,
  E: ParseError<I>,
{
  fn parse(&mut self, input: I) -> IResult<I, I, E> {
    self.parse(input)
  }
}

/// Returns the input slice up to the first occurrence of the pattern.
///
/// It doesn't consume the pattern.
///
/// # Streaming Specific
/// *Streaming version* will return a `Err::Incomplete(Needed::new(N))` if the input doesn't
/// contain the pattern or if the input is smaller than the pattern.
///
/// # Example
/// ```rust
/// # use nom::{Err, error::ErrorKind, Needed, IResult};
/// use nom::bytes::streaming::take_until;
///
/// fn until_eof(s: &str) -> IResult<&str, &str> {
///   take_until("eof").parse(s)
/// }
///
/// assert_eq!(until_eof("hello, worldeof"), Ok(("eof", "hello, world")));
/// assert_eq!(until_eof("hello, world"), Err(Err::Incomplete(Needed::Unknown)));
/// assert_eq!(until_eof("hello, worldeo"), Err(Err::Incomplete(Needed::Unknown)));
/// assert_eq!(until_eof("1eof2eof"), Ok(("eof2eof", "1")));
/// ```
pub fn take_until<T, I, E>(tag: T) -> TakeUntil<T, I, E> {
  TakeUntil {
    tag,
    i: Default::default(),
    e: Default::default(),
  }
}

/// Implementation of [`take_until`]
pub struct TakeUntil<T, I, E> {
  tag: T,
  i: core::marker::PhantomData<I>,
  e: core::marker::PhantomData<E>,
}

impl<T, I, E> TakeUntil<T, I, E>
where
  T: InputLength + Clone,
  I: InputTake + FindSubstring<T>,
  E: ParseError<I>,
{
  /// See [`Parser::parse`]
  pub fn parse(&mut self, input: I) -> IResult<I, I, E> {
    let t = self.tag.clone();
    let res: IResult<_, _, E> = match input.find_substring(t) {
      None => Err(Err::Incomplete(Needed::Unknown)),
      Some(index) => Ok(input.take_split(index)),
    };
    res
  }
}

impl<T, I, E> Parser<I, I, E> for TakeUntil<T, I, E>
where
  T: InputLength + Clone,
  I: InputTake + FindSubstring<T>,
  E: ParseError<I>,
{
  fn parse(&mut self, input: I) -> IResult<I, I, E> {
    self.parse(input)
  }
}

/// Returns the non empty input slice up to the first occurrence of the pattern.
///
/// It doesn't consume the pattern.
///
/// # Streaming Specific
/// *Streaming version* will return a `Err::Incomplete(Needed::new(N))` if the input doesn't
/// contain the pattern or if the input is smaller than the pattern.
///
/// # Example
/// ```rust
/// # use nom::{Err, error::{Error, ErrorKind}, Needed, IResult};
/// use nom::bytes::streaming::take_until1;
///
/// fn until_eof(s: &str) -> IResult<&str, &str> {
///   take_until1("eof").parse(s)
/// }
///
/// assert_eq!(until_eof("hello, worldeof"), Ok(("eof", "hello, world")));
/// assert_eq!(until_eof("hello, world"), Err(Err::Incomplete(Needed::Unknown)));
/// assert_eq!(until_eof("hello, worldeo"), Err(Err::Incomplete(Needed::Unknown)));
/// assert_eq!(until_eof("1eof2eof"), Ok(("eof2eof", "1")));
/// assert_eq!(until_eof("eof"),  Err(Err::Error(Error::new("eof", ErrorKind::TakeUntil))));
/// ```
pub fn take_until1<T, I, E>(tag: T) -> TakeUntil1<T, I, E> {
  TakeUntil1 {
    tag,
    i: Default::default(),
    e: Default::default(),
  }
}

/// Implementation of [`take_until1`]
pub struct TakeUntil1<T, I, E> {
  tag: T,
  i: core::marker::PhantomData<I>,
  e: core::marker::PhantomData<E>,
}

impl<T, I, E> TakeUntil1<T, I, E>
where
  T: InputLength + Clone,
  I: InputTake + FindSubstring<T>,
  E: ParseError<I>,
{
  /// See [`Parser::parse`]
  pub fn parse(&mut self, input: I) -> IResult<I, I, E> {
    let t = self.tag.clone();
    let res: IResult<_, _, E> = match input.find_substring(t) {
      None => Err(Err::Incomplete(Needed::Unknown)),
      Some(0) => Err(Err::Error(E::from_error_kind(input, ErrorKind::TakeUntil))),
      Some(index) => Ok(input.take_split(index)),
    };
    res
  }
}

impl<T, I, E> Parser<I, I, E> for TakeUntil1<T, I, E>
where
  T: InputLength + Clone,
  I: InputTake + FindSubstring<T>,
  E: ParseError<I>,
{
  fn parse(&mut self, input: I) -> IResult<I, I, E> {
    self.parse(input)
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
/// # use nom::character::complete::digit1;
/// use nom::bytes::streaming::escaped;
/// use nom::character::streaming::one_of;
///
/// fn esc(s: &str) -> IResult<&str, &str> {
///   escaped(digit1, '\\', one_of("\"n\\")).parse(s)
/// }
///
/// assert_eq!(esc("123;"), Ok((";", "123")));
/// assert_eq!(esc("12\\\"34;"), Ok((";", "12\\\"34")));
/// ```
///
pub fn escaped<F, FO, G, GO, I, E>(
  normal: F,
  control_char: char,
  escapable: G,
) -> Escaped<F, FO, G, GO, I, E> {
  Escaped {
    normal,
    control_char,
    escapable,
    fo: Default::default(),
    go: Default::default(),
    i: Default::default(),
    e: Default::default(),
  }
}

/// Implementation of [`escaped`]
pub struct Escaped<F, FO, G, GO, I, E> {
  normal: F,
  control_char: char,
  escapable: G,
  fo: core::marker::PhantomData<FO>,
  go: core::marker::PhantomData<GO>,
  i: core::marker::PhantomData<I>,
  e: core::marker::PhantomData<E>,
}

impl<'i, F, FO, G, GO, I, E> Escaped<F, FO, G, GO, I, E>
where
  I: Clone
    + crate::traits::Offset
    + InputLength
    + InputTake
    + InputTakeAtPositionPartial
    + Slice<RangeFrom<usize>>
    + InputIter
    + 'i,
  <I as InputIter>::Item: crate::traits::AsChar,
  F: Parser<I, FO, E>,
  G: Parser<I, GO, E>,
  E: ParseError<I>,
{
  /// See [`Parser::parse`]
  pub fn parse(&mut self, input: I) -> IResult<I, I, E> {
    use crate::traits::AsChar;
    let mut i = input.clone();

    while i.input_len() > 0 {
      let current_len = i.input_len();

      match self.normal.parse(i.clone()) {
        Ok((i2, _)) => {
          if i2.input_len() == 0 {
            return Err(Err::Incomplete(Needed::Unknown));
          } else if i2.input_len() == current_len {
            let index = input.offset(&i2);
            return Ok(input.take_split(index));
          } else {
            i = i2;
          }
        }
        Err(Err::Error(_)) => {
          // unwrap() should be safe here since index < $i.input_len()
          if i.iter_elements().next().unwrap().as_char() == self.control_char {
            let next = self.control_char.len_utf8();
            if next >= i.input_len() {
              return Err(Err::Incomplete(Needed::new(1)));
            } else {
              match self.escapable.parse(i.slice(next..)) {
                Ok((i2, _)) => {
                  if i2.input_len() == 0 {
                    return Err(Err::Incomplete(Needed::Unknown));
                  } else {
                    i = i2;
                  }
                }
                Err(e) => return Err(e),
              }
            }
          } else {
            let index = input.offset(&i);
            return Ok(input.take_split(index));
          }
        }
        Err(e) => {
          return Err(e);
        }
      }
    }

    Err(Err::Incomplete(Needed::Unknown))
  }
}

impl<'i, F, FO, G, GO, I, E> Parser<I, I, E> for Escaped<F, FO, G, GO, I, E>
where
  I: Clone
    + crate::traits::Offset
    + InputLength
    + InputTake
    + InputTakeAtPositionPartial
    + Slice<RangeFrom<usize>>
    + InputIter
    + 'i,
  <I as InputIter>::Item: crate::traits::AsChar,
  F: Parser<I, FO, E>,
  G: Parser<I, GO, E>,
  E: ParseError<I>,
{
  fn parse(&mut self, input: I) -> IResult<I, I, E> {
    self.parse(input)
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
/// use nom::bytes::streaming::{escaped_transform, tag};
/// use nom::character::streaming::alpha1;
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
///   ).parse(input)
/// }
///
/// assert_eq!(parser("ab\\\"cd\""), Ok(("\"", String::from("ab\"cd"))));
/// ```
#[cfg(feature = "alloc")]
pub fn escaped_transform<F, FO, G, GO, EXT, I, O, E>(
  normal: F,
  control_char: char,
  transform: G,
) -> EscapedTransform<F, FO, G, GO, EXT, I, O, E> {
  EscapedTransform {
    normal,
    control_char,
    transform,
    fo: Default::default(),
    go: Default::default(),
    ext: Default::default(),
    i: Default::default(),
    o: Default::default(),
    e: Default::default(),
  }
}

/// Implementation of [`escaped_transform`]
#[cfg(feature = "alloc")]
pub struct EscapedTransform<F, FO, G, GO, EXT, I, O, E> {
  normal: F,
  control_char: char,
  transform: G,
  fo: core::marker::PhantomData<FO>,
  go: core::marker::PhantomData<GO>,
  ext: core::marker::PhantomData<EXT>,
  i: core::marker::PhantomData<I>,
  o: core::marker::PhantomData<O>,
  e: core::marker::PhantomData<E>,
}

#[cfg(feature = "alloc")]
impl<F, FO, G, GO, EXT, I, O, E> EscapedTransform<F, FO, G, GO, EXT, I, O, E>
where
  I: Clone
    + crate::traits::Offset
    + InputLength
    + InputTake
    + InputTakeAtPositionPartial
    + Slice<RangeFrom<usize>>
    + InputIter
    + crate::traits::ExtendInto<Item = EXT, Extender = O>,
  <I as InputIter>::Item: crate::traits::AsChar,
  F: Parser<I, FO, E>,
  G: Parser<I, GO, E>,
  FO: crate::traits::ExtendInto<Item = EXT, Extender = O>,
  GO: crate::traits::ExtendInto<Item = EXT, Extender = O>,
  E: ParseError<I>,
{
  /// See [`Parser::parse`]
  pub fn parse(&mut self, input: I) -> IResult<I, O, E> {
    use crate::traits::AsChar;

    let mut index = 0;
    let mut res = input.new_builder();

    let i = input.clone();

    while index < i.input_len() {
      let current_len = i.input_len();
      let remainder = i.slice(index..);
      match self.normal.parse(remainder.clone()) {
        Ok((i2, o)) => {
          o.extend_into(&mut res);
          if i2.input_len() == 0 {
            return Err(Err::Incomplete(Needed::Unknown));
          } else if i2.input_len() == current_len {
            return Ok((remainder, res));
          } else {
            index = input.offset(&i2);
          }
        }
        Err(Err::Error(_)) => {
          // unwrap() should be safe here since index < $i.input_len()
          if remainder.iter_elements().next().unwrap().as_char() == self.control_char {
            let next = index + self.control_char.len_utf8();
            let input_len = input.input_len();

            if next >= input_len {
              return Err(Err::Incomplete(Needed::Unknown));
            } else {
              match self.transform.parse(i.slice(next..)) {
                Ok((i2, o)) => {
                  o.extend_into(&mut res);
                  if i2.input_len() == 0 {
                    return Err(Err::Incomplete(Needed::Unknown));
                  } else {
                    index = input.offset(&i2);
                  }
                }
                Err(e) => return Err(e),
              }
            }
          } else {
            return Ok((remainder, res));
          }
        }
        Err(e) => return Err(e),
      }
    }
    Err(Err::Incomplete(Needed::Unknown))
  }
}

#[cfg(feature = "alloc")]
impl<F, FO, G, GO, EXT, I, O, E> Parser<I, O, E> for EscapedTransform<F, FO, G, GO, EXT, I, O, E>
where
  I: Clone
    + crate::traits::Offset
    + InputLength
    + InputTake
    + InputTakeAtPositionPartial
    + Slice<RangeFrom<usize>>
    + InputIter
    + crate::traits::ExtendInto<Item = EXT, Extender = O>,
  <I as InputIter>::Item: crate::traits::AsChar,
  F: Parser<I, FO, E>,
  G: Parser<I, GO, E>,
  FO: crate::traits::ExtendInto<Item = EXT, Extender = O>,
  GO: crate::traits::ExtendInto<Item = EXT, Extender = O>,
  E: ParseError<I>,
{
  fn parse(&mut self, input: I) -> IResult<I, O, E> {
    self.parse(input)
  }
}
