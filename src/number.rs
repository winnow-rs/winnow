//! Deprecated, see [`binary`]
#![deprecated(since = "0.4.2", note = "Replaced with `binary`")]
#![allow(clippy::match_same_arms)]

use crate::binary;
use crate::error::ParseError;
use crate::stream::{AsBytes, Stream, StreamIsPartial};
use crate::IResult;
use crate::Parser;

pub use binary::Endianness;

/// Deprecated, see [`binary::be_u8`]
#[deprecated(since = "0.4.2", note = "Replaced with `binary::be_u8`")]
#[inline(always)]
pub fn be_u8<I, E: ParseError<I>>(input: I) -> IResult<I, u8, E>
where
    I: StreamIsPartial,
    I: Stream<Token = u8>,
{
    binary::be_u8(input)
}

/// Deprecated, see [`binary::be_u16`]
#[deprecated(since = "0.4.2", note = "Replaced with `binary::be_u16`")]
#[inline(always)]
pub fn be_u16<I, E: ParseError<I>>(input: I) -> IResult<I, u16, E>
where
    I: StreamIsPartial,
    I: Stream<Token = u8>,
    <I as Stream>::Slice: AsBytes,
{
    binary::be_u16(input)
}

/// Deprecated, see [`binary::be_u24`]
#[deprecated(since = "0.4.2", note = "Replaced with `binary::be_u24`")]
#[inline(always)]
pub fn be_u24<I, E: ParseError<I>>(input: I) -> IResult<I, u32, E>
where
    I: StreamIsPartial,
    I: Stream<Token = u8>,
    <I as Stream>::Slice: AsBytes,
{
    binary::be_u24(input)
}

/// Deprecated, see [`binary::be_u32`]
#[deprecated(since = "0.4.2", note = "Replaced with `binary::be_u32`")]
#[inline(always)]
pub fn be_u32<I, E: ParseError<I>>(input: I) -> IResult<I, u32, E>
where
    I: StreamIsPartial,
    I: Stream<Token = u8>,
    <I as Stream>::Slice: AsBytes,
{
    binary::be_u32(input)
}

/// Deprecated, see [`binary::be_u64`]
#[deprecated(since = "0.4.2", note = "Replaced with `binary::be_u64`")]
#[inline(always)]
pub fn be_u64<I, E: ParseError<I>>(input: I) -> IResult<I, u64, E>
where
    I: StreamIsPartial,
    I: Stream<Token = u8>,
    <I as Stream>::Slice: AsBytes,
{
    binary::be_u64(input)
}

/// Deprecated, see [`binary::be_u128`]
#[deprecated(since = "0.4.2", note = "Replaced with `binary::be_u128`")]
#[inline(always)]
pub fn be_u128<I, E: ParseError<I>>(input: I) -> IResult<I, u128, E>
where
    I: StreamIsPartial,
    I: Stream<Token = u8>,
    <I as Stream>::Slice: AsBytes,
{
    binary::be_u128(input)
}

/// Deprecated, see [`binary::be_i8`]
#[deprecated(since = "0.4.2", note = "Replaced with `binary::be_i8`")]
#[inline(always)]
pub fn be_i8<I, E: ParseError<I>>(input: I) -> IResult<I, i8, E>
where
    I: StreamIsPartial,
    I: Stream<Token = u8>,
{
    binary::be_i8(input)
}

/// Deprecated, see [`binary::be_i16`]
#[deprecated(since = "0.4.2", note = "Replaced with `binary::be_i16`")]
#[inline(always)]
pub fn be_i16<I, E: ParseError<I>>(input: I) -> IResult<I, i16, E>
where
    I: StreamIsPartial,
    I: Stream<Token = u8>,
    <I as Stream>::Slice: AsBytes,
{
    binary::be_i16(input)
}

/// Deprecated, see [`binary::be_i24`]
#[deprecated(since = "0.4.2", note = "Replaced with `binary::be_i24`")]
#[inline(always)]
pub fn be_i24<I, E: ParseError<I>>(input: I) -> IResult<I, i32, E>
where
    I: StreamIsPartial,
    I: Stream<Token = u8>,
    <I as Stream>::Slice: AsBytes,
{
    binary::be_i24(input)
}

/// Deprecated, see [`binary::be_i32`]
#[deprecated(since = "0.4.2", note = "Replaced with `binary::be_i32`")]
#[inline(always)]
pub fn be_i32<I, E: ParseError<I>>(input: I) -> IResult<I, i32, E>
where
    I: StreamIsPartial,
    I: Stream<Token = u8>,
    <I as Stream>::Slice: AsBytes,
{
    binary::be_i32(input)
}

/// Deprecated, see [`binary::be_i64`]
#[deprecated(since = "0.4.2", note = "Replaced with `binary::be_i64`")]
#[inline(always)]
pub fn be_i64<I, E: ParseError<I>>(input: I) -> IResult<I, i64, E>
where
    I: StreamIsPartial,
    I: Stream<Token = u8>,
    <I as Stream>::Slice: AsBytes,
{
    binary::be_i64(input)
}

/// Deprecated, see [`binary::be_i128`]
#[deprecated(since = "0.4.2", note = "Replaced with `binary::be_i128`")]
#[inline(always)]
pub fn be_i128<I, E: ParseError<I>>(input: I) -> IResult<I, i128, E>
where
    I: StreamIsPartial,
    I: Stream<Token = u8>,
    <I as Stream>::Slice: AsBytes,
{
    binary::be_i128(input)
}

/// Deprecated, see [`binary::le_u8`]
#[deprecated(since = "0.4.2", note = "Replaced with `binary::le_u8`")]
#[inline(always)]
pub fn le_u8<I, E: ParseError<I>>(input: I) -> IResult<I, u8, E>
where
    I: StreamIsPartial,
    I: Stream<Token = u8>,
{
    binary::le_u8(input)
}

/// Deprecated, see [`binary::le_u16`]
#[deprecated(since = "0.4.2", note = "Replaced with `binary::le_u16`")]
#[inline(always)]
pub fn le_u16<I, E: ParseError<I>>(input: I) -> IResult<I, u16, E>
where
    I: StreamIsPartial,
    I: Stream<Token = u8>,
    <I as Stream>::Slice: AsBytes,
{
    binary::le_u16(input)
}

/// Deprecated, see [`binary::le_u24`]
#[deprecated(since = "0.4.2", note = "Replaced with `binary::le_u24`")]
#[inline(always)]
pub fn le_u24<I, E: ParseError<I>>(input: I) -> IResult<I, u32, E>
where
    I: StreamIsPartial,
    I: Stream<Token = u8>,
    <I as Stream>::Slice: AsBytes,
{
    binary::le_u24(input)
}

/// Deprecated, see [`binary::le_u32`]
#[deprecated(since = "0.4.2", note = "Replaced with `binary::le_u32`")]
#[inline(always)]
pub fn le_u32<I, E: ParseError<I>>(input: I) -> IResult<I, u32, E>
where
    I: StreamIsPartial,
    I: Stream<Token = u8>,
    <I as Stream>::Slice: AsBytes,
{
    binary::le_u32(input)
}

/// Deprecated, see [`binary::le_u64`]
#[deprecated(since = "0.4.2", note = "Replaced with `binary::le_u64`")]
#[inline(always)]
pub fn le_u64<I, E: ParseError<I>>(input: I) -> IResult<I, u64, E>
where
    I: StreamIsPartial,
    I: Stream<Token = u8>,
    <I as Stream>::Slice: AsBytes,
{
    binary::le_u64(input)
}

/// Deprecated, see [`binary::le_u128`]
#[deprecated(since = "0.4.2", note = "Replaced with `binary::le_u128`")]
#[inline(always)]
pub fn le_u128<I, E: ParseError<I>>(input: I) -> IResult<I, u128, E>
where
    I: StreamIsPartial,
    I: Stream<Token = u8>,
    <I as Stream>::Slice: AsBytes,
{
    binary::le_u128(input)
}

/// Deprecated, see [`binary::le_i8`]
#[deprecated(since = "0.4.2", note = "Replaced with `binary::le_i8`")]
#[inline(always)]
pub fn le_i8<I, E: ParseError<I>>(input: I) -> IResult<I, i8, E>
where
    I: StreamIsPartial,
    I: Stream<Token = u8>,
{
    binary::le_i8(input)
}

/// Deprecated, see [`binary::le_i16`]
#[deprecated(since = "0.4.2", note = "Replaced with `binary::le_i16`")]
#[inline(always)]
pub fn le_i16<I, E: ParseError<I>>(input: I) -> IResult<I, i16, E>
where
    I: StreamIsPartial,
    I: Stream<Token = u8>,
    <I as Stream>::Slice: AsBytes,
{
    binary::le_i16(input)
}

/// Deprecated, see [`binary::le_i24`]
#[deprecated(since = "0.4.2", note = "Replaced with `binary::le_i24`")]
#[inline(always)]
pub fn le_i24<I, E: ParseError<I>>(input: I) -> IResult<I, i32, E>
where
    I: StreamIsPartial,
    I: Stream<Token = u8>,
    <I as Stream>::Slice: AsBytes,
{
    binary::le_i24(input)
}

/// Deprecated, see [`binary::le_i32`]
#[deprecated(since = "0.4.2", note = "Replaced with `binary::le_i32`")]
#[inline(always)]
pub fn le_i32<I, E: ParseError<I>>(input: I) -> IResult<I, i32, E>
where
    I: StreamIsPartial,
    I: Stream<Token = u8>,
    <I as Stream>::Slice: AsBytes,
{
    binary::le_i32(input)
}

/// Deprecated, see [`binary::le_i64`]
#[deprecated(since = "0.4.2", note = "Replaced with `binary::le_i64`")]
#[inline(always)]
pub fn le_i64<I, E: ParseError<I>>(input: I) -> IResult<I, i64, E>
where
    I: StreamIsPartial,
    I: Stream<Token = u8>,
    <I as Stream>::Slice: AsBytes,
{
    binary::le_i64(input)
}

/// Deprecated, see [`binary::le_i128`]
#[deprecated(since = "0.4.2", note = "Replaced with `binary::le_i128`")]
#[inline(always)]
pub fn le_i128<I, E: ParseError<I>>(input: I) -> IResult<I, i128, E>
where
    I: StreamIsPartial,
    I: Stream<Token = u8>,
    <I as Stream>::Slice: AsBytes,
{
    binary::le_i128(input)
}

/// Deprecated, see [`binary::u8`]
#[deprecated(since = "0.4.2", note = "Replaced with `binary::u8`")]
#[inline(always)]
pub fn u8<I, E: ParseError<I>>(input: I) -> IResult<I, u8, E>
where
    I: StreamIsPartial,
    I: Stream<Token = u8>,
{
    binary::u8.parse_next(input)
}

/// Deprecated, see [`binary::u16`]
#[deprecated(since = "0.4.2", note = "Replaced with `binary::u16`")]
#[inline(always)]
pub fn u16<I, E: ParseError<I>>(endian: crate::number::Endianness) -> impl Parser<I, u16, E>
where
    I: StreamIsPartial,
    I: Stream<Token = u8>,
    <I as Stream>::Slice: AsBytes,
{
    binary::u16(endian)
}

/// Deprecated, see [`binary::u24`]
#[deprecated(since = "0.4.2", note = "Replaced with `binary::u24`")]
#[inline(always)]
pub fn u24<I, E: ParseError<I>>(endian: crate::number::Endianness) -> impl Parser<I, u32, E>
where
    I: StreamIsPartial,
    I: Stream<Token = u8>,
    <I as Stream>::Slice: AsBytes,
{
    binary::u24(endian)
}

/// Deprecated, see [`binary::u32`]
#[deprecated(since = "0.4.2", note = "Replaced with `binary::u32`")]
#[inline(always)]
pub fn u32<I, E: ParseError<I>>(endian: crate::number::Endianness) -> impl Parser<I, u32, E>
where
    I: StreamIsPartial,
    I: Stream<Token = u8>,
    <I as Stream>::Slice: AsBytes,
{
    binary::u32(endian)
}

/// Deprecated, see [`binary::u64`]
#[deprecated(since = "0.4.2", note = "Replaced with `binary::u64`")]
#[inline(always)]
pub fn u64<I, E: ParseError<I>>(endian: crate::number::Endianness) -> impl Parser<I, u64, E>
where
    I: StreamIsPartial,
    I: Stream<Token = u8>,
    <I as Stream>::Slice: AsBytes,
{
    binary::u64(endian)
}

/// Deprecated, see [`binary::u128`]
#[deprecated(since = "0.4.2", note = "Replaced with `binary::u128`")]
#[inline(always)]
pub fn u128<I, E: ParseError<I>>(endian: crate::number::Endianness) -> impl Parser<I, u128, E>
where
    I: StreamIsPartial,
    I: Stream<Token = u8>,
    <I as Stream>::Slice: AsBytes,
{
    binary::u128(endian)
}

/// Deprecated, see [`binary::i8`]
#[deprecated(since = "0.4.2", note = "Replaced with `binary::i8`")]
#[inline(always)]
pub fn i8<I, E: ParseError<I>>(input: I) -> IResult<I, i8, E>
where
    I: StreamIsPartial,
    I: Stream<Token = u8>,
{
    binary::i8.parse_next(input)
}

/// Deprecated, see [`binary::i16`]
#[deprecated(since = "0.4.2", note = "Replaced with `binary::i16`")]
#[inline(always)]
pub fn i16<I, E: ParseError<I>>(endian: crate::number::Endianness) -> impl Parser<I, i16, E>
where
    I: StreamIsPartial,
    I: Stream<Token = u8>,
    <I as Stream>::Slice: AsBytes,
{
    binary::i16(endian)
}

/// Deprecated, see [`binary::i24`]
#[deprecated(since = "0.4.2", note = "Replaced with `binary::i24`")]
#[inline(always)]
pub fn i24<I, E: ParseError<I>>(endian: crate::number::Endianness) -> impl Parser<I, i32, E>
where
    I: StreamIsPartial,
    I: Stream<Token = u8>,
    <I as Stream>::Slice: AsBytes,
{
    binary::i24(endian)
}

/// Deprecated, see [`binary::i32`]
#[deprecated(since = "0.4.2", note = "Replaced with `binary::i32`")]
#[inline(always)]
pub fn i32<I, E: ParseError<I>>(endian: crate::number::Endianness) -> impl Parser<I, i32, E>
where
    I: StreamIsPartial,
    I: Stream<Token = u8>,
    <I as Stream>::Slice: AsBytes,
{
    binary::i32(endian)
}

/// Deprecated, see [`binary::i64`]
#[deprecated(since = "0.4.2", note = "Replaced with `binary::i64`")]
#[inline(always)]
pub fn i64<I, E: ParseError<I>>(endian: crate::number::Endianness) -> impl Parser<I, i64, E>
where
    I: StreamIsPartial,
    I: Stream<Token = u8>,
    <I as Stream>::Slice: AsBytes,
{
    binary::i64(endian)
}

/// Deprecated, see [`binary::i128`]
#[deprecated(since = "0.4.2", note = "Replaced with `binary::i128`")]
#[inline(always)]
pub fn i128<I, E: ParseError<I>>(endian: crate::number::Endianness) -> impl Parser<I, i128, E>
where
    I: StreamIsPartial,
    I: Stream<Token = u8>,
    <I as Stream>::Slice: AsBytes,
{
    binary::i128(endian)
}

/// Deprecated, see [`binary::be_f32`]
#[deprecated(since = "0.4.2", note = "Replaced with `binary::be_f32`")]
#[inline(always)]
pub fn be_f32<I, E: ParseError<I>>(input: I) -> IResult<I, f32, E>
where
    I: StreamIsPartial,
    I: Stream<Token = u8>,
    <I as Stream>::Slice: AsBytes,
{
    binary::be_f32(input)
}

/// Deprecated, see [`binary::be_f64`]
#[deprecated(since = "0.4.2", note = "Replaced with `binary::be_f64`")]
#[inline(always)]
pub fn be_f64<I, E: ParseError<I>>(input: I) -> IResult<I, f64, E>
where
    I: StreamIsPartial,
    I: Stream<Token = u8>,
    <I as Stream>::Slice: AsBytes,
{
    binary::be_f64(input)
}

/// Deprecated, see [`binary::le_f32`]
#[deprecated(since = "0.4.2", note = "Replaced with `binary::le_f32`")]
#[inline(always)]
pub fn le_f32<I, E: ParseError<I>>(input: I) -> IResult<I, f32, E>
where
    I: StreamIsPartial,
    I: Stream<Token = u8>,
    <I as Stream>::Slice: AsBytes,
{
    binary::le_f32(input)
}

/// Deprecated, see [`binary::le_f64`]
#[deprecated(since = "0.4.2", note = "Replaced with `binary::le_f64`")]
#[inline(always)]
pub fn le_f64<I, E: ParseError<I>>(input: I) -> IResult<I, f64, E>
where
    I: StreamIsPartial,
    I: Stream<Token = u8>,
    <I as Stream>::Slice: AsBytes,
{
    binary::le_f64(input)
}

/// Deprecated, see [`binary::f32`]
#[deprecated(since = "0.4.2", note = "Replaced with `binary::f32`")]
#[inline(always)]
pub fn f32<I, E: ParseError<I>>(endian: crate::number::Endianness) -> impl Parser<I, f32, E>
where
    I: StreamIsPartial,
    I: Stream<Token = u8>,
    <I as Stream>::Slice: AsBytes,
{
    binary::f32(endian)
}

/// Deprecated, see [`binary::f64`]
#[deprecated(since = "0.4.2", note = "Replaced with `binary::f64`")]
#[inline(always)]
pub fn f64<I, E: ParseError<I>>(endian: crate::number::Endianness) -> impl Parser<I, f64, E>
where
    I: StreamIsPartial,
    I: Stream<Token = u8>,
    <I as Stream>::Slice: AsBytes,
{
    binary::f64(endian)
}
