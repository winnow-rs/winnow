//! Deprecated, see [`binary::bits`]
#![deprecated(since = "0.4.2", note = "Replaced with `binary::bits`")]

use crate::binary;
use crate::error::{ErrorConvert, ParseError};
use crate::lib::std::ops::{AddAssign, Shl, Shr};
use crate::stream::{AsBytes, Stream, StreamIsPartial, ToUsize};
use crate::{IResult, Parser};

/// Deprecated, replaced with [`binary::bits::bits`]
#[deprecated(since = "0.4.2", note = "Replaced with `binary::bits::bits`")]
#[inline(always)]
pub fn bits<I, O, E1, E2, P>(parser: P) -> impl Parser<I, O, E2>
where
    E1: ParseError<(I, usize)> + ErrorConvert<E2>,
    E2: ParseError<I>,
    I: Stream,
    P: Parser<(I, usize), O, E1>,
{
    binary::bits::bits(parser)
}

/// Deprecated, replaced with [`binary::bits::bytes`]
#[deprecated(since = "0.4.2", note = "Replaced with `binary::bits::bytes`")]
#[inline(always)]
pub fn bytes<I, O, E1, E2, P>(parser: P) -> impl Parser<(I, usize), O, E2>
where
    E1: ParseError<I> + ErrorConvert<E2>,
    E2: ParseError<(I, usize)>,
    I: Stream<Token = u8>,
    P: Parser<I, O, E1>,
{
    binary::bits::bytes(parser)
}

/// Deprecated, replaced with [`binary::bits::take`]
#[deprecated(since = "0.4.2", note = "Replaced with `binary::bits::take`")]
#[inline(always)]
pub fn take<I, O, C, E: ParseError<(I, usize)>>(count: C) -> impl Parser<(I, usize), O, E>
where
    I: Stream<Token = u8> + AsBytes + StreamIsPartial,
    C: ToUsize,
    O: From<u8> + AddAssign + Shl<usize, Output = O> + Shr<usize, Output = O>,
{
    binary::bits::take(count)
}

/// Deprecated, replaced with [`binary::bits::tag`]
#[deprecated(since = "0.4.2", note = "Replaced with `binary::bits::tag`")]
#[inline(always)]
pub fn tag<I, O, C, E: ParseError<(I, usize)>>(
    pattern: O,
    count: C,
) -> impl Parser<(I, usize), O, E>
where
    I: Stream<Token = u8> + AsBytes + StreamIsPartial,
    C: ToUsize,
    O: From<u8> + AddAssign + Shl<usize, Output = O> + Shr<usize, Output = O> + PartialEq,
{
    binary::bits::tag(pattern, count)
}

/// Deprecated, replaced with [`binary::bits::bool`]
#[deprecated(since = "0.4.2", note = "Replaced with `binary::bits::bool`")]
#[inline(always)]
pub fn bool<I, E: ParseError<(I, usize)>>(input: (I, usize)) -> IResult<(I, usize), bool, E>
where
    I: Stream<Token = u8> + AsBytes + StreamIsPartial,
{
    binary::bits::bool(input)
}
