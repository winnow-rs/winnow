#![allow(clippy::unreadable_literal)]
#![cfg(target_pointer_width = "64")]

#[cfg(feature = "alloc")]
use winnow::binary::be_u64;
#[cfg(feature = "alloc")]
use winnow::binary::length_data;
#[cfg(feature = "alloc")]
use winnow::combinator::repeat;
use winnow::error::{ErrMode, Needed};
use winnow::prelude::*;
use winnow::token::take;
use winnow::Partial;

// Parser definition

// We request a length that would trigger an overflow if computing consumed + requested
#[allow(clippy::type_complexity)]
fn parser02(i: Partial<&[u8]>) -> IResult<Partial<&[u8]>, (&[u8], &[u8])> {
    (take(1_usize), take(18446744073709551615_usize)).parse_peek(i)
}

#[test]
fn overflow_incomplete_tuple() {
    assert_eq!(
        parser02(Partial::new(&b"3"[..])),
        Err(ErrMode::Incomplete(Needed::new(18446744073709551615)))
    );
}

#[test]
#[cfg(feature = "alloc")]
fn overflow_incomplete_length_bytes() {
    fn multi(i: Partial<&[u8]>) -> IResult<Partial<&[u8]>, Vec<&[u8]>> {
        repeat(0.., length_data(be_u64)).parse_peek(i)
    }

    // Trigger an overflow in length_data
    assert_eq!(
        multi(Partial::new(
            &b"\x00\x00\x00\x00\x00\x00\x00\x01\xaa\xff\xff\xff\xff\xff\xff\xff\xff"[..]
        )),
        Err(ErrMode::Incomplete(Needed::new(18446744073709551615)))
    );
}

#[test]
#[cfg(feature = "alloc")]
fn overflow_incomplete_many0() {
    fn multi(i: Partial<&[u8]>) -> IResult<Partial<&[u8]>, Vec<&[u8]>> {
        repeat(0.., length_data(be_u64)).parse_peek(i)
    }

    // Trigger an overflow in repeat
    assert_eq!(
        multi(Partial::new(
            &b"\x00\x00\x00\x00\x00\x00\x00\x01\xaa\xff\xff\xff\xff\xff\xff\xff\xef"[..]
        )),
        Err(ErrMode::Incomplete(Needed::new(18446744073709551599)))
    );
}

#[test]
#[cfg(feature = "alloc")]
fn overflow_incomplete_many1() {
    use winnow::combinator::repeat;

    fn multi(i: Partial<&[u8]>) -> IResult<Partial<&[u8]>, Vec<&[u8]>> {
        repeat(1.., length_data(be_u64)).parse_peek(i)
    }

    // Trigger an overflow in repeat
    assert_eq!(
        multi(Partial::new(
            &b"\x00\x00\x00\x00\x00\x00\x00\x01\xaa\xff\xff\xff\xff\xff\xff\xff\xef"[..]
        )),
        Err(ErrMode::Incomplete(Needed::new(18446744073709551599)))
    );
}

#[test]
#[cfg(feature = "alloc")]
fn overflow_incomplete_many_till0() {
    use winnow::combinator::repeat_till0;

    #[allow(clippy::type_complexity)]
    fn multi(i: Partial<&[u8]>) -> IResult<Partial<&[u8]>, (Vec<&[u8]>, &[u8])> {
        repeat_till0(length_data(be_u64), "abc").parse_peek(i)
    }

    // Trigger an overflow in repeat_till0
    assert_eq!(
        multi(Partial::new(
            &b"\x00\x00\x00\x00\x00\x00\x00\x01\xaa\xff\xff\xff\xff\xff\xff\xff\xef"[..]
        )),
        Err(ErrMode::Incomplete(Needed::new(18446744073709551599)))
    );
}

#[test]
#[cfg(feature = "alloc")]
fn overflow_incomplete_many_m_n() {
    use winnow::combinator::repeat;

    fn multi(i: Partial<&[u8]>) -> IResult<Partial<&[u8]>, Vec<&[u8]>> {
        repeat(2..=4, length_data(be_u64)).parse_peek(i)
    }

    // Trigger an overflow in repeat
    assert_eq!(
        multi(Partial::new(
            &b"\x00\x00\x00\x00\x00\x00\x00\x01\xaa\xff\xff\xff\xff\xff\xff\xff\xef"[..]
        )),
        Err(ErrMode::Incomplete(Needed::new(18446744073709551599)))
    );
}

#[test]
#[cfg(feature = "alloc")]
fn overflow_incomplete_count() {
    fn counter(i: Partial<&[u8]>) -> IResult<Partial<&[u8]>, Vec<&[u8]>> {
        repeat(2, length_data(be_u64)).parse_peek(i)
    }

    assert_eq!(
        counter(Partial::new(
            &b"\x00\x00\x00\x00\x00\x00\x00\x01\xaa\xff\xff\xff\xff\xff\xff\xff\xef"[..]
        )),
        Err(ErrMode::Incomplete(Needed::new(18446744073709551599)))
    );
}

#[test]
#[cfg(feature = "alloc")]
fn overflow_incomplete_length_count() {
    use winnow::binary::be_u8;
    use winnow::binary::length_count;

    fn multi(i: Partial<&[u8]>) -> IResult<Partial<&[u8]>, Vec<&[u8]>> {
        length_count(be_u8, length_data(be_u64)).parse_peek(i)
    }

    assert_eq!(
        multi(Partial::new(
            &b"\x04\x00\x00\x00\x00\x00\x00\x00\x01\xaa\xff\xff\xff\xff\xff\xff\xff\xee"[..]
        )),
        Err(ErrMode::Incomplete(Needed::new(18446744073709551598)))
    );
}

#[test]
#[cfg(feature = "alloc")]
fn overflow_incomplete_length_data() {
    fn multi(i: Partial<&[u8]>) -> IResult<Partial<&[u8]>, Vec<&[u8]>> {
        repeat(0.., length_data(be_u64)).parse_peek(i)
    }

    assert_eq!(
        multi(Partial::new(
            &b"\x00\x00\x00\x00\x00\x00\x00\x01\xaa\xff\xff\xff\xff\xff\xff\xff\xff"[..]
        )),
        Err(ErrMode::Incomplete(Needed::new(18446744073709551615)))
    );
}
