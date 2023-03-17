//! Bit level parsers
//!

use crate::error::{ErrMode, ErrorKind, ParseError};
use crate::lib::std::ops::{AddAssign, Div, Shl, Shr};
use crate::stream::{AsBytes, Stream};
use crate::IResult;

pub(crate) fn take_internal<I, O, E: ParseError<(I, usize)>>(
    (input, bit_offset): (I, usize),
    count: usize,
) -> IResult<(I, usize), O, E>
where
    I: Stream<Token = u8> + AsBytes,
    O: From<u8> + AddAssign + Shl<usize, Output = O> + Shr<usize, Output = O>,
{
    if count == 0 {
        Ok(((input, bit_offset), 0u8.into()))
    } else {
        let cnt = (count + bit_offset).div(8);
        if input.eof_offset() * 8 < count + bit_offset {
            Err(ErrMode::from_error_kind(
                (input, bit_offset),
                ErrorKind::Eof,
            ))
        } else {
            let mut acc: O = 0_u8.into();
            let mut offset: usize = bit_offset;
            let mut remaining: usize = count;
            let mut end_offset: usize = 0;

            for byte in input.as_bytes().iter().copied().take(cnt + 1) {
                if remaining == 0 {
                    break;
                }
                let val: O = if offset == 0 {
                    byte.into()
                } else {
                    (byte << offset >> offset).into()
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
            let (input, _) = input.next_slice(cnt);
            Ok(((input, end_offset), acc))
        }
    }
}
