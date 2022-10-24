//! Bit level parsers
//!

pub mod complete;
pub mod streaming;

use crate::error::{ErrorKind, ParseError};
use crate::internal::{Err, IResult, Needed};
use crate::lib::std::ops::RangeFrom;
use crate::traits::{ErrorConvert, Slice};
use crate::Parser;

/// Converts a byte-level input to a bit-level input, for consumption by a parser that uses bits.
///
/// Afterwards, the input is converted back to a byte-level parser, with any remaining bits thrown
/// away.
///
/// # Example
/// ```
/// use nom::bits::{bits, streaming::take};
/// use nom::error::Error;
/// use nom::sequence::tuple;
/// use nom::IResult;
///
/// fn parse(input: &[u8]) -> IResult<&[u8], (u8, u8)> {
///     bits::<_, Error<(&[u8], usize)>, _, _, _>(tuple((take(4usize), take(8usize)))).parse(input)
/// }
///
/// let input = &[0x12, 0x34, 0xff, 0xff];
///
/// let output = parse(input).expect("We take 1.5 bytes and the input is longer than 2 bytes");
///
/// // The first byte is consumed, the second byte is partially consumed and dropped.
/// let remaining = output.0;
/// assert_eq!(remaining, [0xff, 0xff]);
///
/// let parsed = output.1;
/// assert_eq!(parsed.0, 0x01);
/// assert_eq!(parsed.1, 0x23);
/// ```
pub fn bits<P, PE, I, O, E>(parser: P) -> Bits<P, PE, I, O, E> {
  Bits {
    parser,
    pe: Default::default(),
    i: Default::default(),
    o: Default::default(),
    e: Default::default(),
  }
}

/// Implementation of [`bits`]
pub struct Bits<P, PE, I, O, E> {
  parser: P,
  pe: core::marker::PhantomData<PE>,
  i: core::marker::PhantomData<I>,
  o: core::marker::PhantomData<O>,
  e: core::marker::PhantomData<E>,
}

impl<P, PE, I, O, E> Bits<P, PE, I, O, E>
where
  PE: ParseError<(I, usize)> + ErrorConvert<E>,
  E: ParseError<I>,
  I: Slice<RangeFrom<usize>>,
  P: Parser<(I, usize), O, PE>,
{
  /// See [`Parser::parse`]
  pub fn parse(&mut self, input: I) -> IResult<I, O, E> {
    match self.parser.parse((input, 0)) {
      Ok(((rest, offset), result)) => {
        // If the next byte has been partially read, it will be sliced away as well.
        // The parser functions might already slice away all fully read bytes.
        // That's why `offset / 8` isn't necessarily needed at all times.
        let remaining_bytes_index = offset / 8 + if offset % 8 == 0 { 0 } else { 1 };
        Ok((rest.slice(remaining_bytes_index..), result))
      }
      Err(Err::Incomplete(n)) => Err(Err::Incomplete(n.map(|u| u.get() / 8 + 1))),
      Err(Err::Error(e)) => Err(Err::Error(e.convert())),
      Err(Err::Failure(e)) => Err(Err::Failure(e.convert())),
    }
  }
}

impl<P, PE, I, O, E> Parser<I, O, E> for Bits<P, PE, I, O, E>
where
  PE: ParseError<(I, usize)> + ErrorConvert<E>,
  E: ParseError<I>,
  I: Slice<RangeFrom<usize>>,
  P: Parser<(I, usize), O, PE>,
{
  fn parse(&mut self, input: I) -> IResult<I, O, E> {
    self.parse(input)
  }
}

/// Counterpart to `bits`, `bytes` transforms its bit stream input into a byte slice for the underlying
/// parser, allowing byte-slice parsers to work on bit streams.
///
/// A partial byte remaining in the input will be ignored and the given parser will start parsing
/// at the next full byte.
///
/// ```
/// use nom::bits::{bits, bytes, streaming::take};
/// use nom::combinator::rest;
/// use nom::error::Error;
/// use nom::sequence::tuple;
/// use nom::IResult;
///
/// fn parse(input: &[u8]) -> IResult<&[u8], (u8, u8, &[u8])> {
///   bits::<_, Error<(&[u8], usize)>, _, _, _>(tuple((
///     take(4usize),
///     take(8usize),
///     bytes::<_, Error<&[u8]>, _, _, _>(rest)
///   ))).parse(input)
/// }
///
/// let input = &[0x12, 0x34, 0xff, 0xff];
///
/// assert_eq!(parse( input ), Ok(( &[][..], (0x01, 0x23, &[0xff, 0xff][..]) )));
/// ```
pub fn bytes<P, PE, I, O, E>(parser: P) -> Bytes<P, PE, I, O, E> {
  Bytes {
    parser,
    pe: Default::default(),
    i: Default::default(),
    o: Default::default(),
    e: Default::default(),
  }
}

/// Implementation of [`bits`]
pub struct Bytes<P, PE, I, O, E> {
  parser: P,
  pe: core::marker::PhantomData<PE>,
  i: core::marker::PhantomData<I>,
  o: core::marker::PhantomData<O>,
  e: core::marker::PhantomData<E>,
}

impl<P, PE, I, O, E> Bytes<P, PE, I, O, E>
where
  PE: ParseError<I> + ErrorConvert<E>,
  E: ParseError<(I, usize)>,
  I: Slice<RangeFrom<usize>> + Clone,
  P: Parser<I, O, PE>,
{
  /// See [`Parser::parse`]
  pub fn parse(&mut self, (input, offset): (I, usize)) -> IResult<(I, usize), O, E> {
    let inner = if offset % 8 != 0 {
      input.slice((1 + offset / 8)..)
    } else {
      input.slice((offset / 8)..)
    };
    let i = (input, offset);
    match self.parser.parse(inner) {
      Ok((rest, res)) => Ok(((rest, 0), res)),
      Err(Err::Incomplete(Needed::Unknown)) => Err(Err::Incomplete(Needed::Unknown)),
      Err(Err::Incomplete(Needed::Size(sz))) => Err(match sz.get().checked_mul(8) {
        Some(v) => Err::Incomplete(Needed::new(v)),
        None => Err::Failure(E::from_error_kind(i, ErrorKind::TooLarge)),
      }),
      Err(Err::Error(e)) => Err(Err::Error(e.convert())),
      Err(Err::Failure(e)) => Err(Err::Failure(e.convert())),
    }
  }
}

impl<P, PE, I, O, E> Parser<(I, usize), O, E> for Bytes<P, PE, I, O, E>
where
  PE: ParseError<I> + ErrorConvert<E>,
  E: ParseError<(I, usize)>,
  I: Slice<RangeFrom<usize>> + Clone,
  P: Parser<I, O, PE>,
{
  fn parse(&mut self, input: (I, usize)) -> IResult<(I, usize), O, E> {
    self.parse(input)
  }
}

#[cfg(test)]
mod test {
  use super::*;
  use crate::bits::streaming::take;
  use crate::error::Error;
  use crate::sequence::tuple;

  #[test]
  /// Take the `bits` function and assert that remaining bytes are correctly returned, if the
  /// previous bytes are fully consumed
  fn test_complete_byte_consumption_bits() {
    let input = &[0x12, 0x34, 0x56, 0x78];

    // Take 3 bit slices with sizes [4, 8, 4].
    let result: IResult<&[u8], (u8, u8, u8)> =
      bits::<_, Error<(&[u8], usize)>, _, _, _>(tuple((take(4usize), take(8usize), take(4usize))))
        .parse(input);

    let output = result.expect("We take 2 bytes and the input is longer than 2 bytes");

    let remaining = output.0;
    assert_eq!(remaining, [0x56, 0x78]);

    let parsed = output.1;
    assert_eq!(parsed.0, 0x01);
    assert_eq!(parsed.1, 0x23);
    assert_eq!(parsed.2, 0x04);
  }

  #[test]
  /// Take the `bits` function and assert that remaining bytes are correctly returned, if the
  /// previous bytes are NOT fully consumed. Partially consumed bytes are supposed to be dropped.
  /// I.e. if we consume 1.5 bytes of 4 bytes, 2 bytes will be returned, bits 13-16 will be
  /// dropped.
  fn test_partial_byte_consumption_bits() {
    let input = &[0x12, 0x34, 0x56, 0x78];

    // Take bit slices with sizes [4, 8].
    let result: IResult<&[u8], (u8, u8)> =
      bits::<_, Error<(&[u8], usize)>, _, _, _>(tuple((take(4usize), take(8usize)))).parse(input);

    let output = result.expect("We take 1.5 bytes and the input is longer than 2 bytes");

    let remaining = output.0;
    assert_eq!(remaining, [0x56, 0x78]);

    let parsed = output.1;
    assert_eq!(parsed.0, 0x01);
    assert_eq!(parsed.1, 0x23);
  }

  #[test]
  #[cfg(feature = "std")]
  /// Ensure that in Incomplete error is thrown, if too few bytes are passed for a given parser.
  fn test_incomplete_bits() {
    let input = &[0x12];

    // Take bit slices with sizes [4, 8].
    let result: IResult<&[u8], (u8, u8)> =
      bits::<_, Error<(&[u8], usize)>, _, _, _>(tuple((take(4usize), take(8usize)))).parse(input);

    assert!(result.is_err());
    let error = result.err().unwrap();
    assert_eq!("Parsing requires 2 bytes/chars", error.to_string());
  }
}
