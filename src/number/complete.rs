//! Parsers recognizing numbers, complete input version

#![allow(clippy::match_same_arms)]

use crate::error::ParseError;
use crate::error::{ErrMode, ErrorKind};
use crate::lib::std::ops::{Add, Shl};
use crate::stream::{AsBytes, Stream};
use crate::*;

#[inline]
pub(crate) fn be_u8<I, E: ParseError<I>>(input: I) -> IResult<I, u8, E>
where
    I: Stream<Token = u8>,
{
    u8(input)
}

#[inline]
pub(crate) fn be_u16<I, E: ParseError<I>>(input: I) -> IResult<I, u16, E>
where
    I: Stream<Token = u8>,
    <I as Stream>::Slice: AsBytes,
{
    be_uint(input, 2)
}

#[inline]
pub(crate) fn be_u24<I, E: ParseError<I>>(input: I) -> IResult<I, u32, E>
where
    I: Stream<Token = u8>,
    <I as Stream>::Slice: AsBytes,
{
    be_uint(input, 3)
}

#[inline]
pub(crate) fn be_u32<I, E: ParseError<I>>(input: I) -> IResult<I, u32, E>
where
    I: Stream<Token = u8>,
    <I as Stream>::Slice: AsBytes,
{
    be_uint(input, 4)
}

#[inline]
pub(crate) fn be_u64<I, E: ParseError<I>>(input: I) -> IResult<I, u64, E>
where
    I: Stream<Token = u8>,
    <I as Stream>::Slice: AsBytes,
{
    be_uint(input, 8)
}

#[inline]
pub(crate) fn be_u128<I, E: ParseError<I>>(input: I) -> IResult<I, u128, E>
where
    I: Stream<Token = u8>,
    <I as Stream>::Slice: AsBytes,
{
    be_uint(input, 16)
}

#[inline]
fn be_uint<I, Uint, E: ParseError<I>>(input: I, bound: usize) -> IResult<I, Uint, E>
where
    I: Stream<Token = u8>,
    <I as Stream>::Slice: AsBytes,
    Uint: Default + Shl<u8, Output = Uint> + Add<Uint, Output = Uint> + From<u8>,
{
    let offset = input
        .offset_at(bound)
        .map_err(|_err| ErrMode::Backtrack(E::from_error_kind(input.clone(), ErrorKind::Eof)))?;
    let (input, number) = input.next_slice(offset);
    let number = number.as_bytes();

    let mut res = Uint::default();
    // special case to avoid shift a byte with overflow
    if bound > 1 {
        for byte in number.iter().copied().take(bound) {
            res = (res << 8) + byte.into();
        }
    } else {
        for byte in number.iter().copied().take(bound) {
            res = byte.into();
        }
    }

    Ok((input, res))
}

#[inline]
pub(crate) fn be_i8<I, E: ParseError<I>>(input: I) -> IResult<I, i8, E>
where
    I: Stream<Token = u8>,
{
    be_u8.map(|x| x as i8).parse_next(input)
}

#[inline]
pub(crate) fn be_i16<I, E: ParseError<I>>(input: I) -> IResult<I, i16, E>
where
    I: Stream<Token = u8>,
    <I as Stream>::Slice: AsBytes,
{
    be_u16.map(|x| x as i16).parse_next(input)
}

#[inline]
pub(crate) fn be_i24<I, E: ParseError<I>>(input: I) -> IResult<I, i32, E>
where
    I: Stream<Token = u8>,
    <I as Stream>::Slice: AsBytes,
{
    // Same as the unsigned version but we need to sign-extend manually here
    be_u24
        .map(|x| {
            if x & 0x80_00_00 != 0 {
                (x | 0xff_00_00_00) as i32
            } else {
                x as i32
            }
        })
        .parse_next(input)
}

#[inline]
pub(crate) fn be_i32<I, E: ParseError<I>>(input: I) -> IResult<I, i32, E>
where
    I: Stream<Token = u8>,
    <I as Stream>::Slice: AsBytes,
{
    be_u32.map(|x| x as i32).parse_next(input)
}

#[inline]
pub(crate) fn be_i64<I, E: ParseError<I>>(input: I) -> IResult<I, i64, E>
where
    I: Stream<Token = u8>,
    <I as Stream>::Slice: AsBytes,
{
    be_u64.map(|x| x as i64).parse_next(input)
}

#[inline]
pub(crate) fn be_i128<I, E: ParseError<I>>(input: I) -> IResult<I, i128, E>
where
    I: Stream<Token = u8>,
    <I as Stream>::Slice: AsBytes,
{
    be_u128.map(|x| x as i128).parse_next(input)
}

#[inline]
pub(crate) fn le_u8<I, E: ParseError<I>>(input: I) -> IResult<I, u8, E>
where
    I: Stream<Token = u8>,
{
    u8(input)
}

#[inline]
pub(crate) fn le_u16<I, E: ParseError<I>>(input: I) -> IResult<I, u16, E>
where
    I: Stream<Token = u8>,
    <I as Stream>::Slice: AsBytes,
{
    le_uint(input, 2)
}

#[inline]
pub(crate) fn le_u24<I, E: ParseError<I>>(input: I) -> IResult<I, u32, E>
where
    I: Stream<Token = u8>,
    <I as Stream>::Slice: AsBytes,
{
    le_uint(input, 3)
}

#[inline]
pub(crate) fn le_u32<I, E: ParseError<I>>(input: I) -> IResult<I, u32, E>
where
    I: Stream<Token = u8>,
    <I as Stream>::Slice: AsBytes,
{
    le_uint(input, 4)
}

#[inline]
pub(crate) fn le_u64<I, E: ParseError<I>>(input: I) -> IResult<I, u64, E>
where
    I: Stream<Token = u8>,
    <I as Stream>::Slice: AsBytes,
{
    le_uint(input, 8)
}

#[inline]
pub(crate) fn le_u128<I, E: ParseError<I>>(input: I) -> IResult<I, u128, E>
where
    I: Stream<Token = u8>,
    <I as Stream>::Slice: AsBytes,
{
    le_uint(input, 16)
}

#[inline]
fn le_uint<I, Uint, E: ParseError<I>>(input: I, bound: usize) -> IResult<I, Uint, E>
where
    I: Stream<Token = u8>,
    <I as Stream>::Slice: AsBytes,
    Uint: Default + Shl<u8, Output = Uint> + Add<Uint, Output = Uint> + From<u8>,
{
    let offset = input
        .offset_at(bound)
        .map_err(|_err| ErrMode::Backtrack(E::from_error_kind(input.clone(), ErrorKind::Eof)))?;
    let (input, number) = input.next_slice(offset);
    let number = number.as_bytes();

    let mut res = Uint::default();
    for (index, byte) in number.iter_offsets().take(bound) {
        res = res + (Uint::from(byte) << (8 * index as u8));
    }

    Ok((input, res))
}

#[inline]
pub(crate) fn le_i8<I, E: ParseError<I>>(input: I) -> IResult<I, i8, E>
where
    I: Stream<Token = u8>,
{
    be_u8.map(|x| x as i8).parse_next(input)
}

#[inline]
pub(crate) fn le_i16<I, E: ParseError<I>>(input: I) -> IResult<I, i16, E>
where
    I: Stream<Token = u8>,
    <I as Stream>::Slice: AsBytes,
{
    le_u16.map(|x| x as i16).parse_next(input)
}

#[inline]
pub(crate) fn le_i24<I, E: ParseError<I>>(input: I) -> IResult<I, i32, E>
where
    I: Stream<Token = u8>,
    <I as Stream>::Slice: AsBytes,
{
    // Same as the unsigned version but we need to sign-extend manually here
    le_u24
        .map(|x| {
            if x & 0x80_00_00 != 0 {
                (x | 0xff_00_00_00) as i32
            } else {
                x as i32
            }
        })
        .parse_next(input)
}

#[inline]
pub(crate) fn le_i32<I, E: ParseError<I>>(input: I) -> IResult<I, i32, E>
where
    I: Stream<Token = u8>,
    <I as Stream>::Slice: AsBytes,
{
    le_u32.map(|x| x as i32).parse_next(input)
}

#[inline]
pub(crate) fn le_i64<I, E: ParseError<I>>(input: I) -> IResult<I, i64, E>
where
    I: Stream<Token = u8>,
    <I as Stream>::Slice: AsBytes,
{
    le_u64.map(|x| x as i64).parse_next(input)
}

#[inline]
pub(crate) fn le_i128<I, E: ParseError<I>>(input: I) -> IResult<I, i128, E>
where
    I: Stream<Token = u8>,
    <I as Stream>::Slice: AsBytes,
{
    le_u128.map(|x| x as i128).parse_next(input)
}

pub(crate) fn u8<I, E: ParseError<I>>(input: I) -> IResult<I, u8, E>
where
    I: Stream<Token = u8>,
{
    input
        .next_token()
        .ok_or_else(|| ErrMode::Backtrack(E::from_error_kind(input, ErrorKind::Eof)))
}

#[inline]
pub(crate) fn u16<I, E: ParseError<I>>(
    endian: crate::number::Endianness,
) -> fn(I) -> IResult<I, u16, E>
where
    I: Stream<Token = u8>,
    <I as Stream>::Slice: AsBytes,
{
    match endian {
        crate::number::Endianness::Big => be_u16,
        crate::number::Endianness::Little => le_u16,
        #[cfg(target_endian = "big")]
        crate::number::Endianness::Native => be_u16,
        #[cfg(target_endian = "little")]
        crate::number::Endianness::Native => le_u16,
    }
}

#[inline]
pub(crate) fn u24<I, E: ParseError<I>>(
    endian: crate::number::Endianness,
) -> fn(I) -> IResult<I, u32, E>
where
    I: Stream<Token = u8>,
    <I as Stream>::Slice: AsBytes,
{
    match endian {
        crate::number::Endianness::Big => be_u24,
        crate::number::Endianness::Little => le_u24,
        #[cfg(target_endian = "big")]
        crate::number::Endianness::Native => be_u24,
        #[cfg(target_endian = "little")]
        crate::number::Endianness::Native => le_u24,
    }
}

#[inline]
pub(crate) fn u32<I, E: ParseError<I>>(
    endian: crate::number::Endianness,
) -> fn(I) -> IResult<I, u32, E>
where
    I: Stream<Token = u8>,
    <I as Stream>::Slice: AsBytes,
{
    match endian {
        crate::number::Endianness::Big => be_u32,
        crate::number::Endianness::Little => le_u32,
        #[cfg(target_endian = "big")]
        crate::number::Endianness::Native => be_u32,
        #[cfg(target_endian = "little")]
        crate::number::Endianness::Native => le_u32,
    }
}

#[inline]
pub(crate) fn u64<I, E: ParseError<I>>(
    endian: crate::number::Endianness,
) -> fn(I) -> IResult<I, u64, E>
where
    I: Stream<Token = u8>,
    <I as Stream>::Slice: AsBytes,
{
    match endian {
        crate::number::Endianness::Big => be_u64,
        crate::number::Endianness::Little => le_u64,
        #[cfg(target_endian = "big")]
        crate::number::Endianness::Native => be_u64,
        #[cfg(target_endian = "little")]
        crate::number::Endianness::Native => le_u64,
    }
}

#[inline]
pub(crate) fn u128<I, E: ParseError<I>>(
    endian: crate::number::Endianness,
) -> fn(I) -> IResult<I, u128, E>
where
    I: Stream<Token = u8>,
    <I as Stream>::Slice: AsBytes,
{
    match endian {
        crate::number::Endianness::Big => be_u128,
        crate::number::Endianness::Little => le_u128,
        #[cfg(target_endian = "big")]
        crate::number::Endianness::Native => be_u128,
        #[cfg(target_endian = "little")]
        crate::number::Endianness::Native => le_u128,
    }
}

#[inline]
pub(crate) fn i8<I, E: ParseError<I>>(i: I) -> IResult<I, i8, E>
where
    I: Stream<Token = u8>,
{
    u8.map(|x| x as i8).parse_next(i)
}

#[inline]
pub(crate) fn i16<I, E: ParseError<I>>(
    endian: crate::number::Endianness,
) -> fn(I) -> IResult<I, i16, E>
where
    I: Stream<Token = u8>,
    <I as Stream>::Slice: AsBytes,
{
    match endian {
        crate::number::Endianness::Big => be_i16,
        crate::number::Endianness::Little => le_i16,
        #[cfg(target_endian = "big")]
        crate::number::Endianness::Native => be_i16,
        #[cfg(target_endian = "little")]
        crate::number::Endianness::Native => le_i16,
    }
}

#[inline]
pub(crate) fn i24<I, E: ParseError<I>>(
    endian: crate::number::Endianness,
) -> fn(I) -> IResult<I, i32, E>
where
    I: Stream<Token = u8>,
    <I as Stream>::Slice: AsBytes,
{
    match endian {
        crate::number::Endianness::Big => be_i24,
        crate::number::Endianness::Little => le_i24,
        #[cfg(target_endian = "big")]
        crate::number::Endianness::Native => be_i24,
        #[cfg(target_endian = "little")]
        crate::number::Endianness::Native => le_i24,
    }
}

#[inline]
pub(crate) fn i32<I, E: ParseError<I>>(
    endian: crate::number::Endianness,
) -> fn(I) -> IResult<I, i32, E>
where
    I: Stream<Token = u8>,
    <I as Stream>::Slice: AsBytes,
{
    match endian {
        crate::number::Endianness::Big => be_i32,
        crate::number::Endianness::Little => le_i32,
        #[cfg(target_endian = "big")]
        crate::number::Endianness::Native => be_i32,
        #[cfg(target_endian = "little")]
        crate::number::Endianness::Native => le_i32,
    }
}

#[inline]
pub(crate) fn i64<I, E: ParseError<I>>(
    endian: crate::number::Endianness,
) -> fn(I) -> IResult<I, i64, E>
where
    I: Stream<Token = u8>,
    <I as Stream>::Slice: AsBytes,
{
    match endian {
        crate::number::Endianness::Big => be_i64,
        crate::number::Endianness::Little => le_i64,
        #[cfg(target_endian = "big")]
        crate::number::Endianness::Native => be_i64,
        #[cfg(target_endian = "little")]
        crate::number::Endianness::Native => le_i64,
    }
}

#[inline]
pub(crate) fn i128<I, E: ParseError<I>>(
    endian: crate::number::Endianness,
) -> fn(I) -> IResult<I, i128, E>
where
    I: Stream<Token = u8>,
    <I as Stream>::Slice: AsBytes,
{
    match endian {
        crate::number::Endianness::Big => be_i128,
        crate::number::Endianness::Little => le_i128,
        #[cfg(target_endian = "big")]
        crate::number::Endianness::Native => be_i128,
        #[cfg(target_endian = "little")]
        crate::number::Endianness::Native => le_i128,
    }
}

#[inline]
pub(crate) fn be_f32<I, E: ParseError<I>>(input: I) -> IResult<I, f32, E>
where
    I: Stream<Token = u8>,
    <I as Stream>::Slice: AsBytes,
{
    match be_u32(input) {
        Err(e) => Err(e),
        Ok((i, o)) => Ok((i, f32::from_bits(o))),
    }
}

#[inline]
pub(crate) fn be_f64<I, E: ParseError<I>>(input: I) -> IResult<I, f64, E>
where
    I: Stream<Token = u8>,
    <I as Stream>::Slice: AsBytes,
{
    match be_u64(input) {
        Err(e) => Err(e),
        Ok((i, o)) => Ok((i, f64::from_bits(o))),
    }
}

#[inline]
pub(crate) fn le_f32<I, E: ParseError<I>>(input: I) -> IResult<I, f32, E>
where
    I: Stream<Token = u8>,
    <I as Stream>::Slice: AsBytes,
{
    match le_u32(input) {
        Err(e) => Err(e),
        Ok((i, o)) => Ok((i, f32::from_bits(o))),
    }
}

#[inline]
pub(crate) fn le_f64<I, E: ParseError<I>>(input: I) -> IResult<I, f64, E>
where
    I: Stream<Token = u8>,
    <I as Stream>::Slice: AsBytes,
{
    match le_u64(input) {
        Err(e) => Err(e),
        Ok((i, o)) => Ok((i, f64::from_bits(o))),
    }
}

#[inline]
pub(crate) fn f32<I, E: ParseError<I>>(
    endian: crate::number::Endianness,
) -> fn(I) -> IResult<I, f32, E>
where
    I: Stream<Token = u8>,
    <I as Stream>::Slice: AsBytes,
{
    match endian {
        crate::number::Endianness::Big => be_f32,
        crate::number::Endianness::Little => le_f32,
        #[cfg(target_endian = "big")]
        crate::number::Endianness::Native => be_f32,
        #[cfg(target_endian = "little")]
        crate::number::Endianness::Native => le_f32,
    }
}

#[inline]
pub(crate) fn f64<I, E: ParseError<I>>(
    endian: crate::number::Endianness,
) -> fn(I) -> IResult<I, f64, E>
where
    I: Stream<Token = u8>,
    <I as Stream>::Slice: AsBytes,
{
    match endian {
        crate::number::Endianness::Big => be_f64,
        crate::number::Endianness::Little => le_f64,
        #[cfg(target_endian = "big")]
        crate::number::Endianness::Native => be_f64,
        #[cfg(target_endian = "little")]
        crate::number::Endianness::Native => le_f64,
    }
}
