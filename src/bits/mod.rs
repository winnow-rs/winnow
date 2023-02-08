//! Bit level parsers
//!

pub mod complete;
pub mod streaming;
#[cfg(test)]
mod tests;

use crate::error::{ErrMode, ErrorConvert, ErrorKind, Needed, ParseError};
use crate::input::{AsBytes, Input, InputIsStreaming, ToUsize};
use crate::lib::std::ops::{AddAssign, Shl, Shr};
use crate::{IResult, Parser};

/// Converts a byte-level input to a bit-level input, for consumption by a parser that uses bits.
///
/// Afterwards, the input is converted back to a byte-level parser, with any remaining bits thrown
/// away.
///
/// # Example
/// ```
/// use winnow::bits::{bits, take};
/// use winnow::error::Error;
/// use winnow::IResult;
///
/// fn parse(input: &[u8]) -> IResult<&[u8], (u8, u8)> {
///     bits::<_, _, Error<(&[u8], usize)>, _, _>((take(4usize), take(8usize)))(input)
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
pub fn bits<I, O, E1, E2, P>(mut parser: P) -> impl FnMut(I) -> IResult<I, O, E2>
where
    E1: ParseError<(I, usize)> + ErrorConvert<E2>,
    E2: ParseError<I>,
    I: Input,
    P: Parser<(I, usize), O, E1>,
{
    move |input: I| match parser.parse_next((input, 0)) {
        Ok(((rest, offset), result)) => {
            // If the next byte has been partially read, it will be sliced away as well.
            // The parser functions might already slice away all fully read bytes.
            // That's why `offset / 8` isn't necessarily needed at all times.
            let remaining_bytes_index = offset / 8 + if offset % 8 == 0 { 0 } else { 1 };
            let (input, _) = rest.next_slice(remaining_bytes_index);
            Ok((input, result))
        }
        Err(ErrMode::Incomplete(n)) => Err(ErrMode::Incomplete(n.map(|u| u.get() / 8 + 1))),
        Err(e) => Err(e.convert()),
    }
}

/// Counterpart to `bits`, `bytes` transforms its bit stream input into a byte slice for the underlying
/// parser, allowing byte-slice parsers to work on bit streams.
///
/// A partial byte remaining in the input will be ignored and the given parser will start parsing
/// at the next full byte.
///
/// ```
/// use winnow::bits::{bits, bytes, take};
/// use winnow::combinator::rest;
/// use winnow::error::Error;
/// use winnow::IResult;
///
/// fn parse(input: &[u8]) -> IResult<&[u8], (u8, u8, &[u8])> {
///   bits::<_, _, Error<(&[u8], usize)>, _, _>((
///     take(4usize),
///     take(8usize),
///     bytes::<_, _, Error<&[u8]>, _, _>(rest)
///   ))(input)
/// }
///
/// let input = &[0x12, 0x34, 0xff, 0xff];
///
/// assert_eq!(parse( input ), Ok(( &[][..], (0x01, 0x23, &[0xff, 0xff][..]) )));
/// ```
pub fn bytes<I, O, E1, E2, P>(mut parser: P) -> impl FnMut((I, usize)) -> IResult<(I, usize), O, E2>
where
    E1: ParseError<I> + ErrorConvert<E2>,
    E2: ParseError<(I, usize)>,
    I: Input,
    P: Parser<I, O, E1>,
{
    move |(input, offset): (I, usize)| {
        let (inner, _) = if offset % 8 != 0 {
            input.next_slice(1 + offset / 8)
        } else {
            input.next_slice(offset / 8)
        };
        let i = (input, offset);
        match parser.parse_next(inner) {
            Ok((rest, res)) => Ok(((rest, 0), res)),
            Err(ErrMode::Incomplete(Needed::Unknown)) => Err(ErrMode::Incomplete(Needed::Unknown)),
            Err(ErrMode::Incomplete(Needed::Size(sz))) => Err(match sz.get().checked_mul(8) {
                Some(v) => ErrMode::Incomplete(Needed::new(v)),
                None => ErrMode::Cut(E2::from_error_kind(i, ErrorKind::TooLarge)),
            }),
            Err(e) => Err(e.convert()),
        }
    }
}

/// Generates a parser taking `count` bits
///
/// # Example
/// ```rust
/// # use winnow::bits::take;
/// # use winnow::IResult;
/// # use winnow::error::{Error, ErrorKind};
/// // Input is a tuple of (input: I, bit_offset: usize)
/// fn parser(input: (&[u8], usize), count: usize)-> IResult<(&[u8], usize), u8> {
///  take(count)(input)
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
/// assert_eq!(parser(([0b00010010].as_ref(), 0), 12), Err(winnow::error::ErrMode::Backtrack(Error{input: ([0b00010010].as_ref(), 0), kind: ErrorKind::Eof })));
/// ```
#[inline(always)]
pub fn take<I, O, C, E: ParseError<(I, usize)>, const STREAMING: bool>(
    count: C,
) -> impl Fn((I, usize)) -> IResult<(I, usize), O, E>
where
    I: Input<Token = u8> + AsBytes + InputIsStreaming<STREAMING>,
    C: ToUsize,
    O: From<u8> + AddAssign + Shl<usize, Output = O> + Shr<usize, Output = O>,
{
    let count = count.to_usize();
    move |input: (I, usize)| {
        if STREAMING {
            streaming::take_internal(input, count)
        } else {
            complete::take_internal(input, count)
        }
    }
}

/// Generates a parser taking `count` bits and comparing them to `pattern`
///
/// # Example
///
/// ```rust
/// use winnow::error::{Error, ErrorKind};
///
/// /// Compare the lowest `count` bits of `input` against the lowest `count` bits of `pattern`.
/// /// Return Ok and the matching section of `input` if there's a match.
/// /// Return Err if there's no match.
/// fn parser(pattern: u8, count: u8, input: (&[u8], usize)) -> winnow::IResult<(&[u8], usize), u8> {
///     winnow::bits::tag(pattern, count)(input)
/// }
///
/// // The lowest 4 bits of 0b00001111 match the lowest 4 bits of 0b11111111.
/// assert_eq!(
///     parser(0b0000_1111, 4, ([0b1111_1111].as_ref(), 0)),
///     Ok((([0b1111_1111].as_ref(), 4), 0b0000_1111))
/// );
///
/// // The lowest bit of 0b00001111 matches the lowest bit of 0b11111111 (both are 1).
/// assert_eq!(
///     parser(0b00000001, 1, ([0b11111111].as_ref(), 0)),
///     Ok((([0b11111111].as_ref(), 1), 0b00000001))
/// );
///
/// // The lowest 2 bits of 0b11111111 and 0b00000001 are different.
/// assert_eq!(
///     parser(0b000000_01, 2, ([0b111111_11].as_ref(), 0)),
///     Err(winnow::error::ErrMode::Backtrack(Error {
///         input: ([0b11111111].as_ref(), 0),
///         kind: ErrorKind::TagBits
///     }))
/// );
///
/// // The lowest 8 bits of 0b11111111 and 0b11111110 are different.
/// assert_eq!(
///     parser(0b11111110, 8, ([0b11111111].as_ref(), 0)),
///     Err(winnow::error::ErrMode::Backtrack(Error {
///         input: ([0b11111111].as_ref(), 0),
///         kind: ErrorKind::TagBits
///     }))
/// );
/// ```
#[inline(always)]
pub fn tag<I, O, C, E: ParseError<(I, usize)>, const STREAMING: bool>(
    pattern: O,
    count: C,
) -> impl Fn((I, usize)) -> IResult<(I, usize), O, E>
where
    I: Input<Token = u8> + AsBytes + InputIsStreaming<STREAMING>,
    C: ToUsize,
    O: From<u8> + AddAssign + Shl<usize, Output = O> + Shr<usize, Output = O> + PartialEq,
{
    let count = count.to_usize();
    move |input: (I, usize)| {
        if STREAMING {
            streaming::tag_internal(input, &pattern, count)
        } else {
            complete::tag_internal(input, &pattern, count)
        }
    }
}

/// Parses one specific bit as a bool.
///
/// # Example
///
/// ```rust
/// # use winnow::bits::bool;
/// # use winnow::IResult;
/// # use winnow::error::{Error, ErrorKind};
///
/// fn parse(input: (&[u8], usize)) -> IResult<(&[u8], usize), bool> {
///     bool(input)
/// }
///
/// assert_eq!(parse(([0b10000000].as_ref(), 0)), Ok((([0b10000000].as_ref(), 1), true)));
/// assert_eq!(parse(([0b10000000].as_ref(), 1)), Ok((([0b10000000].as_ref(), 2), false)));
/// ```
pub fn bool<I, E: ParseError<(I, usize)>, const STREAMING: bool>(
    input: (I, usize),
) -> IResult<(I, usize), bool, E>
where
    I: Input<Token = u8> + AsBytes + InputIsStreaming<STREAMING>,
{
    #![allow(deprecated)]
    if STREAMING {
        streaming::bool(input)
    } else {
        complete::bool(input)
    }
}
