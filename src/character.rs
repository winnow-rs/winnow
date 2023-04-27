//! Deprecated, see [`ascii`]
#![deprecated(since = "0.4.2", note = "Replaced with `ascii`")]

use crate::ascii;
use crate::error::ParseError;
use crate::stream::Compare;
use crate::stream::ContainsToken;
use crate::stream::{AsBStr, AsChar, Offset, ParseSlice, Stream, StreamIsPartial};
use crate::IResult;
use crate::Parser;

/// Deprecated, replaced by [`ascii::crlf`]
#[deprecated(since = "0.4.2", note = "Replaced with `ascii::crlf`")]
#[inline(always)]
pub fn crlf<I, E: ParseError<I>>(input: I) -> IResult<I, <I as Stream>::Slice, E>
where
    I: StreamIsPartial,
    I: Stream,
    I: Compare<&'static str>,
{
    ascii::crlf.parse_next(input)
}

/// Deprecated, replaced by [`ascii::not_line_ending`]
#[deprecated(since = "0.4.2", note = "Replaced with `ascii::not_line_ending`")]
#[inline(always)]
pub fn not_line_ending<I, E: ParseError<I>>(input: I) -> IResult<I, <I as Stream>::Slice, E>
where
    I: StreamIsPartial,
    I: Stream + AsBStr,
    I: Compare<&'static str>,
    <I as Stream>::Token: AsChar,
{
    ascii::not_line_ending.parse_next(input)
}

/// Deprecated, replaced by [`ascii::line_ending`]
#[deprecated(since = "0.4.2", note = "Replaced with `ascii::line_ending`")]
#[inline(always)]
pub fn line_ending<I, E: ParseError<I>>(input: I) -> IResult<I, <I as Stream>::Slice, E>
where
    I: StreamIsPartial,
    I: Stream,
    I: Compare<&'static str>,
{
    ascii::line_ending.parse_next(input)
}

/// Deprecated, replaced by [`ascii::newline`]
#[deprecated(since = "0.4.2", note = "Replaced with `ascii::newline`")]
#[inline(always)]
pub fn newline<I, Error: ParseError<I>>(input: I) -> IResult<I, char, Error>
where
    I: StreamIsPartial,
    I: Stream,
    <I as Stream>::Token: AsChar + Copy,
{
    ascii::newline.parse_next(input)
}

/// Deprecated, replaced by [`ascii::tab`]
#[deprecated(since = "0.4.2", note = "Replaced with `ascii::tab`")]
#[inline(always)]
pub fn tab<I, Error: ParseError<I>>(input: I) -> IResult<I, char, Error>
where
    I: StreamIsPartial,
    I: Stream,
    <I as Stream>::Token: AsChar + Copy,
{
    ascii::tab.parse_next(input)
}

/// Deprecated, replaced by [`ascii::alpha0`]
#[deprecated(since = "0.4.2", note = "Replaced with `ascii::alpha0`")]
#[inline(always)]
pub fn alpha0<I, E: ParseError<I>>(input: I) -> IResult<I, <I as Stream>::Slice, E>
where
    I: StreamIsPartial,
    I: Stream,
    <I as Stream>::Token: AsChar,
{
    ascii::alpha0.parse_next(input)
}

/// Deprecated, replaced by [`ascii::alpha1`]
#[deprecated(since = "0.4.2", note = "Replaced with `ascii::alpha1`")]
#[inline(always)]
pub fn alpha1<I, E: ParseError<I>>(input: I) -> IResult<I, <I as Stream>::Slice, E>
where
    I: StreamIsPartial,
    I: Stream,
    <I as Stream>::Token: AsChar,
{
    ascii::alpha1.parse_next(input)
}

/// Deprecated, replaced by [`ascii::digit0`]
#[deprecated(since = "0.4.2", note = "Replaced with `ascii::digit0`")]
#[inline(always)]
pub fn digit0<I, E: ParseError<I>>(input: I) -> IResult<I, <I as Stream>::Slice, E>
where
    I: StreamIsPartial,
    I: Stream,
    <I as Stream>::Token: AsChar,
{
    ascii::digit0.parse_next(input)
}

/// Deprecated, replaced by [`ascii::digit1`]
#[deprecated(since = "0.4.2", note = "Replaced with `ascii::digit1`")]
#[inline(always)]
pub fn digit1<I, E: ParseError<I>>(input: I) -> IResult<I, <I as Stream>::Slice, E>
where
    I: StreamIsPartial,
    I: Stream,
    <I as Stream>::Token: AsChar,
{
    ascii::digit1.parse_next(input)
}

/// Deprecated, replaced by [`ascii::hex_digit0`]
#[deprecated(since = "0.4.2", note = "Replaced with `ascii::hex_digit0`")]
#[inline(always)]
pub fn hex_digit0<I, E: ParseError<I>>(input: I) -> IResult<I, <I as Stream>::Slice, E>
where
    I: StreamIsPartial,
    I: Stream,
    <I as Stream>::Token: AsChar,
{
    ascii::hex_digit0.parse_next(input)
}

/// Deprecated, replaced by [`ascii::hex_digit1`]
#[deprecated(since = "0.4.2", note = "Replaced with `ascii::hex_digit1`")]
#[inline(always)]
pub fn hex_digit1<I, E: ParseError<I>>(input: I) -> IResult<I, <I as Stream>::Slice, E>
where
    I: StreamIsPartial,
    I: Stream,
    <I as Stream>::Token: AsChar,
{
    ascii::hex_digit1.parse_next(input)
}

/// Deprecated, replaced by [`ascii::oct_digit0`]
#[deprecated(since = "0.4.2", note = "Replaced with `ascii::oct_digit0`")]
#[inline(always)]
pub fn oct_digit0<I, E: ParseError<I>>(input: I) -> IResult<I, <I as Stream>::Slice, E>
where
    I: StreamIsPartial,
    I: Stream,
    <I as Stream>::Token: AsChar,
{
    ascii::oct_digit0.parse_next(input)
}

/// Deprecated, replaced by [`ascii::oct_digit1`]
#[deprecated(since = "0.4.2", note = "Replaced with `ascii::oct_digit1`")]
#[inline(always)]
pub fn oct_digit1<I, E: ParseError<I>>(input: I) -> IResult<I, <I as Stream>::Slice, E>
where
    I: StreamIsPartial,
    I: Stream,
    <I as Stream>::Token: AsChar,
{
    ascii::oct_digit0.parse_next(input)
}

/// Deprecated, replaced by [`ascii::alphanumeric0`]
#[deprecated(since = "0.4.2", note = "Replaced with `ascii::alphanumeric0`")]
#[inline(always)]
pub fn alphanumeric0<I, E: ParseError<I>>(input: I) -> IResult<I, <I as Stream>::Slice, E>
where
    I: StreamIsPartial,
    I: Stream,
    <I as Stream>::Token: AsChar,
{
    ascii::alphanumeric0.parse_next(input)
}

/// Deprecated, replaced by [`ascii::alphanumeric1`]
#[deprecated(since = "0.4.2", note = "Replaced with `ascii::alphanumeric1`")]
#[inline(always)]
pub fn alphanumeric1<I, E: ParseError<I>>(input: I) -> IResult<I, <I as Stream>::Slice, E>
where
    I: StreamIsPartial,
    I: Stream,
    <I as Stream>::Token: AsChar,
{
    ascii::alphanumeric1.parse_next(input)
}

/// Deprecated, replaced by [`ascii::space0`]
#[deprecated(since = "0.4.2", note = "Replaced with `ascii::space0`")]
#[inline(always)]
pub fn space0<I, E: ParseError<I>>(input: I) -> IResult<I, <I as Stream>::Slice, E>
where
    I: StreamIsPartial,
    I: Stream,
    <I as Stream>::Token: AsChar + Copy,
{
    ascii::space0.parse_next(input)
}

/// Deprecated, replaced by [`ascii::space1`]
#[deprecated(since = "0.4.2", note = "Replaced with `ascii::space1`")]
#[inline(always)]
pub fn space1<I, E: ParseError<I>>(input: I) -> IResult<I, <I as Stream>::Slice, E>
where
    I: StreamIsPartial,
    I: Stream,
    <I as Stream>::Token: AsChar + Copy,
{
    ascii::space1.parse_next(input)
}

/// Deprecated, replaced by [`ascii::multispace0`]
#[deprecated(since = "0.4.2", note = "Replaced with `ascii::multispace0`")]
#[inline(always)]
pub fn multispace0<I, E: ParseError<I>>(input: I) -> IResult<I, <I as Stream>::Slice, E>
where
    I: StreamIsPartial,
    I: Stream,
    <I as Stream>::Token: AsChar + Copy,
{
    ascii::multispace0.parse_next(input)
}

/// Deprecated, replaced by [`ascii::multispace1`]
#[deprecated(since = "0.4.2", note = "Replaced with `ascii::multispace1`")]
#[inline(always)]
pub fn multispace1<I, E: ParseError<I>>(input: I) -> IResult<I, <I as Stream>::Slice, E>
where
    I: StreamIsPartial,
    I: Stream,
    <I as Stream>::Token: AsChar + Copy,
{
    ascii::multispace1.parse_next(input)
}

/// Deprecated, replaced by [`ascii::dec_uint`]
#[deprecated(since = "0.4.2", note = "Replaced with `ascii::dec_uint`")]
#[inline(always)]
pub fn dec_uint<I, O, E: ParseError<I>>(input: I) -> IResult<I, O, E>
where
    I: StreamIsPartial,
    I: Stream,
    <I as Stream>::Token: AsChar + Copy,
    O: Uint,
{
    ascii::dec_uint.parse_next(input)
}

pub use ascii::Uint;

/// Deprecated, replaced by [`ascii::dec_int`]
#[deprecated(since = "0.4.2", note = "Replaced with `ascii::dec_int`")]
#[inline(always)]
pub fn dec_int<I, O, E: ParseError<I>>(input: I) -> IResult<I, O, E>
where
    I: StreamIsPartial,
    I: Stream,
    <I as Stream>::Token: AsChar + Copy,
    O: Int,
{
    ascii::dec_int.parse_next(input)
}

pub use ascii::Int;

/// Deprecated, replaced by [`ascii::hex_uint`]
#[deprecated(since = "0.4.2", note = "Replaced with `ascii::hex_uint`")]
#[inline(always)]
pub fn hex_uint<I, O, E: ParseError<I>>(input: I) -> IResult<I, O, E>
where
    I: StreamIsPartial,
    I: Stream,
    O: HexUint,
    <I as Stream>::Token: AsChar,
    <I as Stream>::Slice: AsBStr,
{
    ascii::hex_uint.parse_next(input)
}

pub use ascii::HexUint;

/// Deprecated, replaced by [`ascii::float`]
#[deprecated(since = "0.4.2", note = "Replaced with `ascii::float`")]
#[inline(always)]
pub fn float<I, O, E: ParseError<I>>(input: I) -> IResult<I, O, E>
where
    I: StreamIsPartial,
    I: Stream,
    I: Offset + Compare<&'static str>,
    <I as Stream>::Slice: ParseSlice<O>,
    <I as Stream>::Token: AsChar + Copy,
    <I as Stream>::IterOffsets: Clone,
    I: AsBStr,
    &'static str: ContainsToken<<I as Stream>::Token>,
{
    ascii::float.parse_next(input)
}

/// Deprecated, replaced by [`ascii::escaped`]
#[deprecated(since = "0.4.2", note = "Replaced with `ascii::escaped`")]
#[inline(always)]
pub fn escaped<'a, I: 'a, Error, F, G, O1, O2>(
    normal: F,
    control_char: char,
    escapable: G,
) -> impl Parser<I, <I as Stream>::Slice, Error>
where
    I: StreamIsPartial,
    I: Stream + Offset,
    <I as Stream>::Token: crate::stream::AsChar,
    F: Parser<I, O1, Error>,
    G: Parser<I, O2, Error>,
    Error: ParseError<I>,
{
    ascii::escaped(normal, control_char, escapable)
}

#[cfg(feature = "alloc")]
/// Deprecated, replaced by [`ascii::escaped_transform`]
#[deprecated(since = "0.4.2", note = "Replaced with `ascii::escaped_transform`")]
#[inline(always)]
pub fn escaped_transform<I, Error, F, G, Output>(
    normal: F,
    control_char: char,
    transform: G,
) -> impl Parser<I, Output, Error>
where
    I: StreamIsPartial,
    I: Stream + Offset,
    <I as Stream>::Token: crate::stream::AsChar,
    Output: crate::stream::Accumulate<<I as Stream>::Slice>,
    F: Parser<I, <I as Stream>::Slice, Error>,
    G: Parser<I, <I as Stream>::Slice, Error>,
    Error: ParseError<I>,
{
    ascii::escaped_transform(normal, control_char, transform)
}
