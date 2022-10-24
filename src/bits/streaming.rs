//! Bit level parsers
//!

use crate::error::{ErrorKind, ParseError};
use crate::internal::{Err, IResult, Needed};
use crate::lib::std::ops::{AddAssign, Div, RangeFrom, Shl, Shr};
use crate::traits::{InputIter, InputLength, Slice, ToUsize};
use crate::Parser;

/// Generates a parser taking `count` bits
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
{
  /// See [`Parser::parse`]
  pub fn parse(&mut self, (input, bit_offset): (I, usize)) -> IResult<(I, usize), O, E> {
    let count = self.count.to_usize();
    if count == 0 {
      Ok(((input, bit_offset), 0u8.into()))
    } else {
      let cnt = (count + bit_offset).div(8);
      if input.input_len() * 8 < count + bit_offset {
        Err(Err::Incomplete(Needed::new(count as usize)))
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
    let input = [].as_ref();
    let count = 0usize;
    assert_eq!(count, 0usize);
    let offset = 0usize;

    let result: crate::IResult<(&[u8], usize), usize> = take(count).parse((input, offset));

    assert_eq!(result, Ok(((input, offset), 0)));
  }

  #[test]
  fn test_tag_ok() {
    let input = [0b00011111].as_ref();
    let offset = 0usize;
    let bits_to_take = 4usize;
    let value_to_tag = 0b0001;

    let result: crate::IResult<(&[u8], usize), usize> =
      tag(value_to_tag, bits_to_take).parse((input, offset));

    assert_eq!(result, Ok(((input, bits_to_take), value_to_tag)));
  }

  #[test]
  fn test_tag_err() {
    let input = [0b00011111].as_ref();
    let offset = 0usize;
    let bits_to_take = 4usize;
    let value_to_tag = 0b1111;

    let result: crate::IResult<(&[u8], usize), usize> =
      tag(value_to_tag, bits_to_take).parse((input, offset));

    assert_eq!(
      result,
      Err(crate::Err::Error(crate::error::Error {
        input: (input, offset),
        code: ErrorKind::TagBits
      }))
    );
  }
}
