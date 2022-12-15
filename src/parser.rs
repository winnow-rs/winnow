//! Basic types to build the parsers

use self::Needed::*;
use crate::combinator::*;
use crate::error::{self, ErrorKind};
use crate::input::InputIsStreaming;
use crate::lib::std::fmt;
use core::num::NonZeroUsize;

/// Holds the result of parsing functions
///
/// It depends on the input type `I`, the output type `O`, and the error type `E`
/// (by default `(I, nom::ErrorKind)`)
///
/// The `Ok` side is a pair containing the remainder of the input (the part of the data that
/// was not parsed) and the produced value. The `Err` side contains an instance of `nom::Err`.
///
/// Outside of the parsing code, you can use the [`FinishIResult::finish`] method to convert
/// it to a more common result type
pub type IResult<I, O, E = error::Error<I>> = Result<(I, O), Err<E>>;

/// Extension trait to convert a parser's [`IResult`] to a more manageable type
pub trait FinishIResult<I, O, E> {
  /// Converts the parser's [`IResult`] to a type that is more consumable by callers.
  ///
  /// Errors if the parser is not at the [end of input][crate::combinator::eof].  See
  /// [`FinishIResult::finish_err`] if the remaining input is needed.
  ///
  /// # Panic
  ///
  /// If the result is `Err(Err::Incomplete(_))`, this method will panic.
  /// - "complete" parsers: It will not be an issue, `Incomplete` is never used
  /// - "streaming" parsers: `Incomplete` will be returned if there's not enough data
  /// for the parser to decide, and you should gather more data before parsing again.
  /// Once the parser returns either `Ok(_)`, `Err(Err::Error(_))` or `Err(Err::Failure(_))`,
  /// you can get out of the parsing loop and call `finish_err()` on the parser's result
  fn finish(self) -> Result<O, E>;

  /// Converts the parser's [`IResult`] to a type that is more consumable by errors.
  ///
  ///  It keeps the same `Ok` branch, and merges `Err::Error` and `Err::Failure` into the `Err`
  ///  side.
  ///
  /// # Panic
  ///
  /// If the result is `Err(Err::Incomplete(_))`, this method will panic as [`Err::Incomplete`]
  /// should only be set when the input is [`InputIsStreaming<false>`] which this isn't implemented
  /// for.
  fn finish_err(self) -> Result<(I, O), E>;
}

impl<I, O, E> FinishIResult<I, O, E> for IResult<I, O, E>
where
  I: crate::input::InputLength,
  I: crate::input::IntoOutput,
  // Force users to deal with `Incomplete` when `InputIsStreaming<true>`
  I: InputIsStreaming<false>,
  I: Clone,
  E: crate::error::ParseError<I>,
{
  fn finish(self) -> Result<O, E> {
    let (i, o) = self.finish_err()?;
    crate::combinator::eof(i).finish_err()?;
    Ok(o)
  }

  fn finish_err(self) -> Result<(I, O), E> {
    match self {
      Ok(res) => Ok(res),
      Err(Err::Error(e)) | Err(Err::Failure(e)) => Err(e),
      Err(Err::Incomplete(_)) => {
        panic!("`InputIsStreaming<false>` conflicts with `Err(Err::Incomplete(_))`")
      }
    }
  }
}

#[doc(hidden)]
#[deprecated(
  since = "8.0.0",
  note = "Replaced with `FinishIResult` which is available via `nom::prelude`"
)]
pub trait Finish<I, O, E> {
  #[deprecated(
    since = "8.0.0",
    note = "Replaced with `FinishIResult::finish_err` which is available via `nom::prelude`"
  )]
  fn finish(self) -> Result<(I, O), E>;
}

#[allow(deprecated)]
impl<I, O, E> Finish<I, O, E> for IResult<I, O, E> {
  fn finish(self) -> Result<(I, O), E> {
    match self {
      Ok(res) => Ok(res),
      Err(Err::Error(e)) | Err(Err::Failure(e)) => Err(e),
      Err(Err::Incomplete(_)) => {
        panic!("Cannot call `finish()` on `Err(Err::Incomplete(_))`: this result means that the parser does not have enough data to decide, you should gather more data and try to reapply  the parser instead")
      }
    }
  }
}

/// Convert an `Input` into an appropriate `Output` type
pub trait IntoOutputIResult<I, O, E> {
  /// Convert an `Input` into an appropriate `Output` type
  fn into_output(self) -> IResult<I, O, E>;
}

impl<I, E> IntoOutputIResult<I, <I as crate::input::IntoOutput>::Output, E> for IResult<I, I, E>
where
  I: crate::input::IntoOutput,
{
  fn into_output(self) -> IResult<I, <I as crate::input::IntoOutput>::Output, E> {
    self.map(|(i, o)| (i, o.into_output()))
  }
}

/// Contains information on needed data if a parser returned `Incomplete`
///
/// **Note:** This is only possible for `Input` types that implement [`InputIsStreaming<true>`],
/// like [`Streaming`][crate::input::Streaming].
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[cfg_attr(nightly, warn(rustdoc::missing_doc_code_examples))]
pub enum Needed {
  /// Needs more data, but we do not know how much
  Unknown,
  /// Contains the required data size in bytes
  Size(NonZeroUsize),
}

impl Needed {
  /// Creates `Needed` instance, returns `Needed::Unknown` if the argument is zero
  pub fn new(s: usize) -> Self {
    match NonZeroUsize::new(s) {
      Some(sz) => Needed::Size(sz),
      None => Needed::Unknown,
    }
  }

  /// Indicates if we know how many bytes we need
  pub fn is_known(&self) -> bool {
    *self != Unknown
  }

  /// Maps a `Needed` to `Needed` by applying a function to a contained `Size` value.
  #[inline]
  pub fn map<F: Fn(NonZeroUsize) -> usize>(self, f: F) -> Needed {
    match self {
      Unknown => Unknown,
      Size(n) => Needed::new(f(n)),
    }
  }
}

/// The `Err` enum indicates the parser was not successful
///
/// It has three cases:
///
/// * `Incomplete` indicates that more data is needed to decide. The [`Needed`] enum
/// can contain how many additional bytes are necessary. If you are sure your parser
/// is working on full data, you can wrap your parser with the `complete` combinator
/// to transform that case in `Error`
/// * `Error` means some parser did not succeed, but another one might (as an example,
/// when testing different branches of an `alt` combinator)
/// * `Failure` indicates an unrecoverable error. As an example, if you recognize a prefix
/// to decide on the next parser to apply, and that parser fails, you know there's no need
/// to try other parsers, you were already in the right branch, so the data is invalid
///
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(nightly, warn(rustdoc::missing_doc_code_examples))]
pub enum Err<E> {
  /// There was not enough data
  ///
  /// This must only be set when the `Input` is [`InputIsStreaming<true>`], like with
  /// [`Streaming`][crate::input::Streaming]
  ///
  /// Convert this into an `Error` with [`nom::combinator::complete`][crate::combinator::complete]
  Incomplete(Needed),
  /// The parser had an error (recoverable)
  Error(E),
  /// The parser had an unrecoverable error: we got to the right
  /// branch and we know other branches won't work, so backtrack
  /// as fast as possible
  Failure(E),
}

impl<E> Err<E> {
  /// Tests if the result is Incomplete
  pub fn is_incomplete(&self) -> bool {
    if let Err::Incomplete(_) = self {
      true
    } else {
      false
    }
  }

  /// Applies the given function to the inner error
  pub fn map<E2, F>(self, f: F) -> Err<E2>
  where
    F: FnOnce(E) -> E2,
  {
    match self {
      Err::Incomplete(n) => Err::Incomplete(n),
      Err::Failure(t) => Err::Failure(f(t)),
      Err::Error(t) => Err::Error(f(t)),
    }
  }

  /// Automatically converts between errors if the underlying type supports it
  pub fn convert<F>(e: Err<F>) -> Self
  where
    E: From<F>,
  {
    e.map(crate::lib::std::convert::Into::into)
  }
}

impl<T> Err<(T, ErrorKind)> {
  /// Maps `Err<(T, ErrorKind)>` to `Err<(U, ErrorKind)>` with the given `F: T -> U`
  pub fn map_input<U, F>(self, f: F) -> Err<(U, ErrorKind)>
  where
    F: FnOnce(T) -> U,
  {
    match self {
      Err::Incomplete(n) => Err::Incomplete(n),
      Err::Failure((input, k)) => Err::Failure((f(input), k)),
      Err::Error((input, k)) => Err::Error((f(input), k)),
    }
  }
}

impl<T> Err<error::Error<T>> {
  /// Maps `Err<error::Error<T>>` to `Err<error::Error<U>>` with the given `F: T -> U`
  pub fn map_input<U, F>(self, f: F) -> Err<error::Error<U>>
  where
    F: FnOnce(T) -> U,
  {
    match self {
      Err::Incomplete(n) => Err::Incomplete(n),
      Err::Failure(error::Error { input, code }) => Err::Failure(error::Error {
        input: f(input),
        code,
      }),
      Err::Error(error::Error { input, code }) => Err::Error(error::Error {
        input: f(input),
        code,
      }),
    }
  }
}

#[cfg(feature = "alloc")]
use crate::lib::std::{borrow::ToOwned, string::String, vec::Vec};
impl Err<(&[u8], ErrorKind)> {
  /// Obtaining ownership
  #[cfg(feature = "alloc")]
  pub fn to_owned(self) -> Err<(Vec<u8>, ErrorKind)> {
    self.map_input(ToOwned::to_owned)
  }
}

impl Err<(&str, ErrorKind)> {
  /// Obtaining ownership
  #[cfg(feature = "alloc")]
  pub fn to_owned(self) -> Err<(String, ErrorKind)> {
    self.map_input(ToOwned::to_owned)
  }
}

impl Err<error::Error<&[u8]>> {
  /// Obtaining ownership
  #[cfg(feature = "alloc")]
  pub fn to_owned(self) -> Err<error::Error<Vec<u8>>> {
    self.map_input(ToOwned::to_owned)
  }
}

impl Err<error::Error<&str>> {
  /// Obtaining ownership
  #[cfg(feature = "alloc")]
  pub fn to_owned(self) -> Err<error::Error<String>> {
    self.map_input(ToOwned::to_owned)
  }
}

impl<E: Eq> Eq for Err<E> {}

impl<E> fmt::Display for Err<E>
where
  E: fmt::Debug,
{
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Err::Incomplete(Needed::Size(u)) => write!(f, "Parsing requires {} bytes/chars", u),
      Err::Incomplete(Needed::Unknown) => write!(f, "Parsing requires more data"),
      Err::Failure(c) => write!(f, "Parsing Failure: {:?}", c),
      Err::Error(c) => write!(f, "Parsing Error: {:?}", c),
    }
  }
}

#[cfg(feature = "std")]
use std::error::Error;

#[cfg(feature = "std")]
impl<E> Error for Err<E>
where
  E: fmt::Debug,
{
  fn source(&self) -> Option<&(dyn Error + 'static)> {
    None // no underlying error
  }
}

/// All nom parsers implement this trait
pub trait Parser<I, O, E> {
  /// A parser takes in input type, and returns a `Result` containing
  /// either the remaining input and the output value, or an error
  fn parse(&mut self, input: I) -> IResult<I, O, E>;

  /// Treat `&mut Self` as a parser
  ///
  /// This helps when needing to move a `Parser` when all you have is a `&mut Parser`.
  ///
  /// # Example
  ///
  /// Because parsers are `FnMut`, they can be called multiple times.  This prevents moving `f`
  /// into [`length_data`][crate::multi::length_data] and `g` into
  /// [`complete`][crate::combinator::complete]:
  /// ```rust,compile_fail
  /// # use nom::prelude::*;
  /// # use nom::IResult;
  /// # use nom::Parser;
  /// # use nom::error::ParseError;
  /// # use nom::multi::length_data;
  /// # use nom::combinator::complete;
  /// pub fn length_value<'i, O, E: ParseError<&'i [u8]>>(
  ///     mut f: impl Parser<&'i [u8], usize, E>,
  ///     mut g: impl Parser<&'i [u8], O, E>
  /// ) -> impl FnMut(&'i [u8]) -> IResult<&'i [u8], O, E> {
  ///   move |i: &'i [u8]| {
  ///     let (i, data) = length_data(f).parse(i)?;
  ///     let (_, o) = complete(g).parse(data)?;
  ///     Ok((i, o))
  ///   }
  /// }
  /// ```
  ///
  /// By adding `as_mut_parser`, we can make this work:
  /// ```rust
  /// # use nom::prelude::*;
  /// # use nom::IResult;
  /// # use nom::Parser;
  /// # use nom::error::ParseError;
  /// # use nom::multi::length_data;
  /// # use nom::combinator::complete;
  /// pub fn length_value<'i, O, E: ParseError<&'i [u8]>>(
  ///     mut f: impl Parser<&'i [u8], usize, E>,
  ///     mut g: impl Parser<&'i [u8], O, E>
  /// ) -> impl FnMut(&'i [u8]) -> IResult<&'i [u8], O, E> {
  ///   move |i: &'i [u8]| {
  ///     let (i, data) = length_data(f.as_mut_parser()).parse(i)?;
  ///     let (_, o) = complete(g.as_mut_parser()).parse(data)?;
  ///     Ok((i, o))
  ///   }
  /// }
  /// ```
  fn as_mut_parser(&mut self) -> MutParser<Self>
  where
    Self: core::marker::Sized,
  {
    MutParser::new(self)
  }
  /// Returns the provided value if the child parser succeeds.
  ///
  /// # Example
  ///
  /// ```rust
  /// # use nom::{Err,error::ErrorKind, IResult, Parser};
  /// use nom::character::alpha1;
  /// # fn main() {
  ///
  /// let mut parser = alpha1.value(1234);
  ///
  /// assert_eq!(parser.parse("abcd"), Ok(("", 1234)));
  /// assert_eq!(parser.parse("123abcd;"), Err(Err::Error(("123abcd;", ErrorKind::Alpha))));
  /// # }
  /// ```
  fn value<O2>(self, val: O2) -> Value<Self, O, O2>
  where
    Self: core::marker::Sized,
    O2: Clone,
  {
    Value::new(self, val)
  }

  /// If the child parser was successful, return the consumed input as produced value.
  ///
  /// # Example
  ///
  /// ```rust
  /// # use nom::{Err,error::ErrorKind, IResult, Parser};
  /// use nom::character::{char, alpha1};
  /// use nom::sequence::separated_pair;
  /// # fn main() {
  ///
  /// let mut parser = separated_pair(alpha1, char(','), alpha1).recognize();
  ///
  /// assert_eq!(parser.parse("abcd,efgh"), Ok(("", "abcd,efgh")));
  /// assert_eq!(parser.parse("abcd;"),Err(Err::Error((";", ErrorKind::Char))));
  /// # }
  /// ```
  fn recognize(self) -> Recognize<Self, O>
  where
    Self: core::marker::Sized,
  {
    Recognize::new(self)
  }

  /// Maps a function over the result of a parser
  ///
  /// # Example
  ///
  /// ```rust
  /// use nom::{Err,error::ErrorKind, IResult,Parser};
  /// use nom::character::digit1;
  /// # fn main() {
  ///
  /// let mut parser = digit1.map(|s: &str| s.len());
  ///
  /// // the parser will count how many characters were returned by digit1
  /// assert_eq!(parser.parse("123456"), Ok(("", 6)));
  ///
  /// // this will fail if digit1 fails
  /// assert_eq!(parser.parse("abc"), Err(Err::Error(("abc", ErrorKind::Digit))));
  /// # }
  /// ```
  fn map<G, O2>(self, g: G) -> Map<Self, G, O>
  where
    G: Fn(O) -> O2,
    Self: core::marker::Sized,
  {
    Map::new(self, g)
  }

  /// Applies a function returning a `Result` over the result of a parser.
  ///
  /// # Example
  ///
  /// ```rust
  /// # use nom::{Err,error::ErrorKind, IResult, Parser};
  /// use nom::character::digit1;
  /// # fn main() {
  ///
  /// let mut parse = digit1.map_res(|s: &str| s.parse::<u8>());
  ///
  /// // the parser will convert the result of digit1 to a number
  /// assert_eq!(parse.parse("123"), Ok(("", 123)));
  ///
  /// // this will fail if digit1 fails
  /// assert_eq!(parse.parse("abc"), Err(Err::Error(("abc", ErrorKind::Digit))));
  ///
  /// // this will fail if the mapped function fails (a `u8` is too small to hold `123456`)
  /// assert_eq!(parse.parse("123456"), Err(Err::Error(("123456", ErrorKind::MapRes))));
  /// # }
  /// ```
  fn map_res<G, O2, E2>(self, g: G) -> MapRes<Self, G, O>
  where
    Self: core::marker::Sized,
    G: FnMut(O) -> Result<O2, E2>,
  {
    MapRes::new(self, g)
  }

  /// Applies a function returning an `Option` over the result of a parser.
  ///
  /// # Example
  ///
  /// ```rust
  /// # use nom::{Err,error::ErrorKind, IResult, Parser};
  /// use nom::character::digit1;
  /// # fn main() {
  ///
  /// let mut parse = digit1.map_opt(|s: &str| s.parse::<u8>().ok());
  ///
  /// // the parser will convert the result of digit1 to a number
  /// assert_eq!(parse.parse("123"), Ok(("", 123)));
  ///
  /// // this will fail if digit1 fails
  /// assert_eq!(parse.parse("abc"), Err(Err::Error(("abc", ErrorKind::Digit))));
  ///
  /// // this will fail if the mapped function fails (a `u8` is too small to hold `123456`)
  /// assert_eq!(parse.parse("123456"), Err(Err::Error(("123456", ErrorKind::MapOpt))));
  /// # }
  /// ```
  fn map_opt<G, O2>(self, g: G) -> MapOpt<Self, G, O>
  where
    Self: core::marker::Sized,
    G: FnMut(O) -> Option<O2>,
  {
    MapOpt::new(self, g)
  }

  /// Creates a second parser from the output of the first one, then apply over the rest of the input
  ///
  /// # Example
  ///
  /// ```rust
  /// # use nom::{Err,error::ErrorKind, IResult, Parser};
  /// use nom::bytes::take;
  /// use nom::number::u8;
  /// # fn main() {
  ///
  /// let mut length_data = u8.flat_map(take);
  ///
  /// assert_eq!(length_data.parse(&[2, 0, 1, 2][..]), Ok((&[2][..], &[0, 1][..])));
  /// assert_eq!(length_data.parse(&[4, 0, 1, 2][..]), Err(Err::Error((&[0, 1, 2][..], ErrorKind::Eof))));
  /// # }
  /// ```
  fn flat_map<G, H, O2>(self, g: G) -> FlatMap<Self, G, O>
  where
    G: FnMut(O) -> H,
    H: Parser<I, O2, E>,
    Self: core::marker::Sized,
  {
    FlatMap::new(self, g)
  }

  /// Applies a second parser over the output of the first one
  ///
  /// # Example
  ///
  /// ```rust
  /// # use nom::{Err,error::ErrorKind, IResult, Parser};
  /// use nom::character::digit1;
  /// use nom::bytes::take;
  /// # fn main() {
  ///
  /// let mut digits = take(5u8).and_then(digit1);
  ///
  /// assert_eq!(digits.parse("12345"), Ok(("", "12345")));
  /// assert_eq!(digits.parse("123ab"), Ok(("", "123")));
  /// assert_eq!(digits.parse("123"), Err(Err::Error(("123", ErrorKind::Eof))));
  /// # }
  /// ```
  fn and_then<G, O2>(self, g: G) -> AndThen<Self, G, O>
  where
    G: Parser<O, O2, E>,
    Self: core::marker::Sized,
  {
    AndThen::new(self, g)
  }

  /// Returns the result of the child parser if it satisfies a verification function.
  ///
  /// The verification function takes as argument a reference to the output of the
  /// parser.
  ///
  /// # Example
  ///
  /// ```rust
  /// # use nom::{Err,error::ErrorKind, IResult, Parser};
  /// # use nom::character::alpha1;
  /// # fn main() {
  ///
  /// let mut parser = alpha1.verify(|s: &str| s.len() == 4);
  ///
  /// assert_eq!(parser.parse("abcd"), Ok(("", "abcd")));
  /// assert_eq!(parser.parse("abcde"), Err(Err::Error(("abcde", ErrorKind::Verify))));
  /// assert_eq!(parser.parse("123abcd;"),Err(Err::Error(("123abcd;", ErrorKind::Alpha))));
  /// # }
  /// ```
  fn verify<G, O2: ?Sized>(self, second: G) -> Verify<Self, G, O2>
  where
    Self: core::marker::Sized,
    G: Fn(&O2) -> bool,
  {
    Verify::new(self, second)
  }

  /// Transforms [`Incomplete`][crate::Err::Incomplete] into [`Error`][crate::Err::Error]
  ///
  /// # Example
  ///
  /// ```rust
  /// # use nom::{Err,error::ErrorKind, IResult, input::Streaming, Parser};
  /// # use nom::bytes::take;
  /// # fn main() {
  ///
  /// let mut parser = take(5u8).complete();
  ///
  /// assert_eq!(parser.parse(Streaming("abcdefg")), Ok((Streaming("fg"), "abcde")));
  /// assert_eq!(parser.parse(Streaming("abcd")), Err(Err::Error((Streaming("abcd"), ErrorKind::Complete))));
  /// # }
  /// ```
  fn complete(self) -> Complete<Self>
  where
    Self: core::marker::Sized,
  {
    Complete::new(self)
  }

  /// Applies a second parser after the first one, return their results as a tuple
  ///
  /// **WARNING:** Deprecated, replaced with [`nom::sequence::tuple`][crate::sequence::tuple]
  #[deprecated(since = "8.0.0", note = "Replaced with `nom::sequence::tuple")]
  fn and<G, O2>(self, g: G) -> And<Self, G>
  where
    G: Parser<I, O2, E>,
    Self: core::marker::Sized,
  {
    And::new(self, g)
  }

  /// Applies a second parser over the input if the first one failed
  ///
  /// **WARNING:** Deprecated, replaced with [`nom::branch::alt`][crate::branch::alt]
  #[deprecated(since = "8.0.0", note = "Replaced with `nom::branch::alt")]
  fn or<G>(self, g: G) -> Or<Self, G>
  where
    G: Parser<I, O, E>,
    Self: core::marker::Sized,
  {
    Or::new(self, g)
  }

  /// automatically converts the parser's output and error values to another type, as long as they
  /// implement the `From` trait
  ///
  /// # Example
  ///
  /// ```rust
  /// # use nom::IResult;
  /// # use nom::Parser;
  /// use nom::character::alpha1;
  /// # fn main() {
  ///
  ///  fn parser1(i: &str) -> IResult<&str, &str> {
  ///    alpha1(i)
  ///  }
  ///
  ///  let mut parser2 = Parser::into(parser1);
  ///
  /// // the parser converts the &str output of the child parser into a Vec<u8>
  /// let bytes: IResult<&str, Vec<u8>> = parser2.parse("abcd");
  /// assert_eq!(bytes, Ok(("", vec![97, 98, 99, 100])));
  /// # }
  /// ```
  fn into<O2: From<O>, E2: From<E>>(self) -> Into<Self, O, O2, E, E2>
  where
    Self: core::marker::Sized,
  {
    Into::new(self)
  }
}

impl<'a, I, O, E, F> Parser<I, O, E> for F
where
  F: FnMut(I) -> IResult<I, O, E> + 'a,
{
  fn parse(&mut self, i: I) -> IResult<I, O, E> {
    self(i)
  }
}

#[cfg(feature = "alloc")]
use alloc::boxed::Box;

#[cfg(feature = "alloc")]
impl<'a, I, O, E> Parser<I, O, E> for Box<dyn Parser<I, O, E> + 'a> {
  fn parse(&mut self, input: I) -> IResult<I, O, E> {
    (**self).parse(input)
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::error::ErrorKind;

  #[doc(hidden)]
  #[macro_export]
  macro_rules! assert_size (
    ($t:ty, $sz:expr) => (
      assert!(crate::lib::std::mem::size_of::<$t>() <= $sz, "{} <= {} failed", crate::lib::std::mem::size_of::<$t>(), $sz);
    );
  );

  #[test]
  #[cfg(target_pointer_width = "64")]
  fn size_test() {
    assert_size!(IResult<&[u8], &[u8], (&[u8], u32)>, 40);
    assert_size!(IResult<&str, &str, u32>, 40);
    assert_size!(Needed, 8);
    assert_size!(Err<u32>, 16);
    assert_size!(ErrorKind, 1);
  }

  #[test]
  fn err_map_test() {
    let e = Err::Error(1);
    assert_eq!(e.map(|v| v + 1), Err::Error(2));
  }
}
