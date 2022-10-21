//! Bit level parsers
//!

use crate::error::{ErrorKind, ParseError};
use crate::internal::{Err, IResult};
use crate::lib::std::ops::{AddAssign, Div, RangeFrom, Shl, Shr};
use crate::traits::{InputIter, InputLength, Slice, ToUsize};
use crate::Parser;

/// Generates a parser taking `count` bits
///
/// # Example
/// ```rust
/// # use nom::bits::complete::take;
/// # use nom::IResult;
/// # use nom::error::{Error, ErrorKind};
/// // Input is a tuple of (input: I, bit_offset: usize)
/// fn parser(input: (&[u8], usize), count: usize)-> IResult<(&[u8], usize), u8> {
///  take(count).parse(input)
/// }
///
/// // Consumes 0 bits, returns 0
/// assert_eq!(parser(([0b00010010].as_ref(), 0), 0), Ok((([0b00010010].as_ref(), 0), 0)));
///
/// // Consumes 4 bits, returns their values and increase offset to 4
/// assert_eq!(parser(([0b00010010].as_ref(), 0), 4), Ok((([0b00010010].as_ref(), 4), 0b00000001)));
///
/// // Consumes 4 bits, offset is 4, returns their values and increase offset to 0 of next byte
/// assert_eq!(parser(([0b00010010].as_ref(), 4), 4), Ok((([].as_ref(), 0), 0b00000010)));
///
/// // Tries to consume 12 bits but only 8 are available
/// assert_eq!(parser(([0b00010010].as_ref(), 0), 12), Err(nom::Err::Error(Error{input: ([0b00010010].as_ref(), 0), code: ErrorKind::Eof })));
/// ```
pub fn take<C, I, O, E>(count: C) -> Take<C, I, O, E> {
  Take {
    count,
    i: Default::default(),
    o: Default::default(),
    e: Default::default(),
  }
}

/// Implementation of [`take`]
pub struct Take<C, I, O, E> {
  count: C,
  i: core::marker::PhantomData<I>,
  o: core::marker::PhantomData<O>,
  e: core::marker::PhantomData<E>,
}

impl<C, I, O, E> Take<C, I, O, E>
where
  I: Slice<RangeFrom<usize>> + InputIter<Item = u8> + InputLength,
  C: ToUsize,
  O: From<u8> + AddAssign + Shl<usize, Output = O> + Shr<usize, Output = O>,
  E: ParseError<(I, usize)>,
{
  /// See [`Parser::parse`]
  pub fn parse(&mut self, (input, bit_offset): (I, usize)) -> IResult<(I, usize), O, E> {
    let count = self.count.to_usize();
    if count == 0 {
      Ok(((input, bit_offset), 0u8.into()))
    } else {
      let cnt = (count + bit_offset).div(8);
      if input.input_len() * 8 < count + bit_offset {
        Err(Err::Error(E::from_error_kind(
          (input, bit_offset),
          ErrorKind::Eof,
        )))
      } else {
        let mut acc: O = 0_u8.into();
        let mut offset: usize = bit_offset;
        let mut remaining: usize = count;
        let mut end_offset: usize = 0;

        for byte in input.iter_elements().take(cnt + 1) {
          if remaining == 0 {
            break;
          }
          let val: O = if offset == 0 {
            byte.into()
          } else {
            ((byte << offset) as u8 >> offset).into()
          };

          if remaining < 8 - offset {
            acc += val >> (8 - offset - remaining);
            end_offset = remaining + offset;
            break;
          } else {
            acc += val << (remaining - (8 - offset));
            remaining -= 8 - offset;
            offset = 0;
          }
        }
        Ok(((input.slice(cnt..), end_offset), acc))
      }
    }
  }
}

impl<C, I, O, E> Parser<(I, usize), O, E> for Take<C, I, O, E>
where
  I: Slice<RangeFrom<usize>> + InputIter<Item = u8> + InputLength,
  C: ToUsize,
  O: From<u8> + AddAssign + Shl<usize, Output = O> + Shr<usize, Output = O>,
  E: ParseError<(I, usize)>,
{
  fn parse(&mut self, input: (I, usize)) -> IResult<(I, usize), O, E> {
    self.parse(input)
  }
}

/// Generates a parser taking `count` bits and comparing them to `pattern`
pub fn tag<C, I, O, E>(pattern: O, count: C) -> Tag<C, I, O, E> {
  Tag {
    pattern,
    count,
    i: Default::default(),
    e: Default::default(),
  }
}

/// Implementation of [`tag`]
pub struct Tag<C, I, O, E> {
  pattern: O,
  count: C,
  i: core::marker::PhantomData<I>,
  e: core::marker::PhantomData<E>,
}

impl<C, I, O, E> Tag<C, I, O, E>
where
  I: Slice<RangeFrom<usize>> + InputIter<Item = u8> + InputLength + Clone,
  C: ToUsize,
  O: From<u8> + AddAssign + Shl<usize, Output = O> + Shr<usize, Output = O> + PartialEq,
  E: ParseError<(I, usize)>,
{
  /// See [`Parser::parse`]
  pub fn parse(&mut self, input: (I, usize)) -> IResult<(I, usize), O, E> {
    let count = self.count.to_usize();
    let inp = input.clone();

    take(count).parse(input).and_then(|(i, o)| {
      if self.pattern == o {
        Ok((i, o))
      } else {
        Err(Err::Error(error_position!(inp, ErrorKind::TagBits)))
      }
    })
  }
}

impl<C, I, O, E> Parser<(I, usize), O, E> for Tag<C, I, O, E>
where
  I: Slice<RangeFrom<usize>> + InputIter<Item = u8> + InputLength + Clone,
  C: ToUsize,
  O: From<u8> + AddAssign + Shl<usize, Output = O> + Shr<usize, Output = O> + PartialEq,
  E: ParseError<(I, usize)>,
{
  fn parse(&mut self, input: (I, usize)) -> IResult<(I, usize), O, E> {
    self.parse(input)
  }
}

#[cfg(test)]
mod test {
  use super::*;

  #[test]
  fn test_take_0() {
    let input = [0b00010010].as_ref();
    let count = 0usize;
    assert_eq!(count, 0usize);
    let offset = 0usize;

    let result: crate::IResult<(&[u8], usize), usize> = take(count).parse((input, offset));

    assert_eq!(result, Ok(((input, offset), 0)));
  }

  #[test]
  fn test_take_eof() {
    let input = [0b00010010].as_ref();

    let result: crate::IResult<(&[u8], usize), usize> = take(1usize).parse((input, 8));

    assert_eq!(
      result,
      Err(crate::Err::Error(crate::error::Error {
        input: (input, 8),
        code: ErrorKind::Eof
      }))
    )
  }

  #[test]
  fn test_take_span_over_multiple_bytes() {
    let input = [0b00010010, 0b00110100, 0b11111111, 0b11111111].as_ref();

    let result: crate::IResult<(&[u8], usize), usize> = take(24usize).parse((input, 4));

    assert_eq!(
      result,
      Ok((([0b11111111].as_ref(), 4), 0b1000110100111111111111))
    );
  }
}
