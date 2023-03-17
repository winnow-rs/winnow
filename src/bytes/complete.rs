//! Parsers recognizing bytes streams, complete input version

use crate::error::ErrMode;
use crate::error::ErrorKind;
use crate::error::ParseError;
use crate::lib::std::result::Result::Ok;
use crate::stream::{
    split_at_offset1_complete, split_at_offset_complete, Compare, CompareResult, ContainsToken,
    FindSlice, Offset, SliceLen, Stream,
};
use crate::{IResult, Parser};

pub(crate) fn any<I, E: ParseError<I>>(input: I) -> IResult<I, <I as Stream>::Token, E>
where
    I: Stream,
{
    input
        .next_token()
        .ok_or_else(|| ErrMode::from_error_kind(input, ErrorKind::Eof))
}

pub(crate) fn tag_internal<T, I, Error: ParseError<I>>(
    i: I,
    t: T,
) -> IResult<I, <I as Stream>::Slice, Error>
where
    I: Stream + Compare<T>,
    T: SliceLen,
{
    let tag_len = t.slice_len();
    match i.compare(t) {
        CompareResult::Ok => Ok(i.next_slice(tag_len)),
        CompareResult::Incomplete | CompareResult::Error => {
            let e: ErrorKind = ErrorKind::Tag;
            Err(ErrMode::from_error_kind(i, e))
        }
    }
}

pub(crate) fn tag_no_case_internal<T, I, Error: ParseError<I>>(
    i: I,
    t: T,
) -> IResult<I, <I as Stream>::Slice, Error>
where
    I: Stream + Compare<T>,
    T: SliceLen,
{
    let tag_len = t.slice_len();

    match (i).compare_no_case(t) {
        CompareResult::Ok => Ok(i.next_slice(tag_len)),
        CompareResult::Incomplete | CompareResult::Error => {
            let e: ErrorKind = ErrorKind::Tag;
            Err(ErrMode::from_error_kind(i, e))
        }
    }
}

pub(crate) fn one_of_internal<I, T, E: ParseError<I>>(
    input: I,
    list: &T,
) -> IResult<I, <I as Stream>::Token, E>
where
    I: Stream,
    <I as Stream>::Token: Copy,
    T: ContainsToken<<I as Stream>::Token>,
{
    input
        .next_token()
        .filter(|(_, t)| list.contains_token(*t))
        .ok_or_else(|| ErrMode::from_error_kind(input, ErrorKind::OneOf))
}

pub(crate) fn none_of_internal<I, T, E: ParseError<I>>(
    input: I,
    list: &T,
) -> IResult<I, <I as Stream>::Token, E>
where
    I: Stream,
    <I as Stream>::Token: Copy,
    T: ContainsToken<<I as Stream>::Token>,
{
    input
        .next_token()
        .filter(|(_, t)| !list.contains_token(*t))
        .ok_or_else(|| ErrMode::from_error_kind(input, ErrorKind::NoneOf))
}

pub(crate) fn take_while_internal<T, I, Error: ParseError<I>>(
    i: I,
    list: &T,
) -> IResult<I, <I as Stream>::Slice, Error>
where
    I: Stream,
    T: ContainsToken<<I as Stream>::Token>,
{
    split_at_offset_complete(&i, |c| !list.contains_token(c))
}

pub(crate) fn take_while1_internal<T, I, Error: ParseError<I>>(
    i: I,
    list: &T,
) -> IResult<I, <I as Stream>::Slice, Error>
where
    I: Stream,
    T: ContainsToken<<I as Stream>::Token>,
{
    let e: ErrorKind = ErrorKind::TakeWhile1;
    split_at_offset1_complete(&i, |c| !list.contains_token(c), e)
}

pub(crate) fn take_while_m_n_internal<T, I, Error: ParseError<I>>(
    input: I,
    m: usize,
    n: usize,
    list: &T,
) -> IResult<I, <I as Stream>::Slice, Error>
where
    I: Stream,
    T: ContainsToken<<I as Stream>::Token>,
{
    if n < m {
        return Err(ErrMode::from_error_kind(input, ErrorKind::TakeWhileMN));
    }

    let mut final_count = 0;
    for (processed, (offset, token)) in input.iter_offsets().enumerate() {
        if !list.contains_token(token) {
            if processed < m {
                return Err(ErrMode::from_error_kind(input, ErrorKind::TakeWhileMN));
            } else {
                return Ok(input.next_slice(offset));
            }
        } else {
            if processed == n {
                return Ok(input.next_slice(offset));
            }
            final_count = processed + 1;
        }
    }

    if m <= final_count {
        Ok(input.next_slice(input.eof_offset()))
    } else {
        Err(ErrMode::from_error_kind(input, ErrorKind::TakeWhileMN))
    }
}

pub(crate) fn take_till_internal<T, I, Error: ParseError<I>>(
    i: I,
    list: &T,
) -> IResult<I, <I as Stream>::Slice, Error>
where
    I: Stream,
    T: ContainsToken<<I as Stream>::Token>,
{
    split_at_offset_complete(&i, |c| list.contains_token(c))
}

pub(crate) fn take_till1_internal<T, I, Error: ParseError<I>>(
    i: I,
    list: &T,
) -> IResult<I, <I as Stream>::Slice, Error>
where
    I: Stream,
    T: ContainsToken<<I as Stream>::Token>,
{
    let e: ErrorKind = ErrorKind::TakeTill1;
    split_at_offset1_complete(&i, |c| list.contains_token(c), e)
}

pub(crate) fn take_internal<I, Error: ParseError<I>>(
    i: I,
    c: usize,
) -> IResult<I, <I as Stream>::Slice, Error>
where
    I: Stream,
{
    match i.offset_at(c) {
        Ok(offset) => Ok(i.next_slice(offset)),
        Err(_needed) => Err(ErrMode::from_error_kind(i, ErrorKind::Eof)),
    }
}

pub(crate) fn take_until_internal<T, I, Error: ParseError<I>>(
    i: I,
    t: T,
) -> IResult<I, <I as Stream>::Slice, Error>
where
    I: Stream + FindSlice<T>,
    T: SliceLen,
{
    match i.find_slice(t) {
        Some(offset) => Ok(i.next_slice(offset)),
        None => Err(ErrMode::from_error_kind(i, ErrorKind::TakeUntil)),
    }
}

pub(crate) fn take_until1_internal<T, I, Error: ParseError<I>>(
    i: I,
    t: T,
) -> IResult<I, <I as Stream>::Slice, Error>
where
    I: Stream + FindSlice<T>,
    T: SliceLen,
{
    match i.find_slice(t) {
        None | Some(0) => Err(ErrMode::from_error_kind(i, ErrorKind::TakeUntil)),
        Some(offset) => Ok(i.next_slice(offset)),
    }
}

pub(crate) fn escaped_internal<'a, I: 'a, Error, F, G, O1, O2>(
    input: I,
    normal: &mut F,
    control_char: char,
    escapable: &mut G,
) -> IResult<I, <I as Stream>::Slice, Error>
where
    I: Stream + Offset,
    <I as Stream>::Token: crate::stream::AsChar,
    F: Parser<I, O1, Error>,
    G: Parser<I, O2, Error>,
    Error: ParseError<I>,
{
    use crate::stream::AsChar;

    let mut i = input.clone();

    while i.eof_offset() > 0 {
        let current_len = i.eof_offset();

        match normal.parse_next(i.clone()) {
            Ok((i2, _)) => {
                // return if we consumed everything or if the normal parser
                // does not consume anything
                if i2.eof_offset() == 0 {
                    return Ok(input.next_slice(input.eof_offset()));
                } else if i2.eof_offset() == current_len {
                    let offset = input.offset_to(&i2);
                    return Ok(input.next_slice(offset));
                } else {
                    i = i2;
                }
            }
            Err(ErrMode::Backtrack(_)) => {
                if i.next_token().expect("eof_offset > 0").1.as_char() == control_char {
                    let next = control_char.len_utf8();
                    if next >= i.eof_offset() {
                        return Err(ErrMode::from_error_kind(input, ErrorKind::Escaped));
                    } else {
                        match escapable.parse_next(i.next_slice(next).0) {
                            Ok((i2, _)) => {
                                if i2.eof_offset() == 0 {
                                    return Ok(input.next_slice(input.eof_offset()));
                                } else {
                                    i = i2;
                                }
                            }
                            Err(e) => return Err(e),
                        }
                    }
                } else {
                    let offset = input.offset_to(&i);
                    if offset == 0 {
                        return Err(ErrMode::from_error_kind(input, ErrorKind::Escaped));
                    }
                    return Ok(input.next_slice(offset));
                }
            }
            Err(e) => {
                return Err(e);
            }
        }
    }

    Ok(input.next_slice(input.eof_offset()))
}

#[cfg(feature = "alloc")]
pub(crate) fn escaped_transform_internal<I, Error, F, G, Output>(
    input: I,
    normal: &mut F,
    control_char: char,
    transform: &mut G,
) -> IResult<I, Output, Error>
where
    I: Stream + Offset,
    <I as Stream>::Token: crate::stream::AsChar,
    Output: crate::stream::Accumulate<<I as Stream>::Slice>,
    F: Parser<I, <I as Stream>::Slice, Error>,
    G: Parser<I, <I as Stream>::Slice, Error>,
    Error: ParseError<I>,
{
    use crate::stream::AsChar;

    let mut offset = 0;
    let mut res = Output::initial(Some(input.eof_offset()));

    let i = input.clone();

    while offset < i.eof_offset() {
        let current_len = i.eof_offset();
        let (remainder, _) = i.next_slice(offset);
        match normal.parse_next(remainder.clone()) {
            Ok((i2, o)) => {
                res.accumulate(o);
                if i2.eof_offset() == 0 {
                    return Ok((i.next_slice(i.eof_offset()).0, res));
                } else if i2.eof_offset() == current_len {
                    return Ok((remainder, res));
                } else {
                    offset = input.offset_to(&i2);
                }
            }
            Err(ErrMode::Backtrack(_)) => {
                if remainder.next_token().expect("eof_offset > 0").1.as_char() == control_char {
                    let next = offset + control_char.len_utf8();
                    let eof_offset = input.eof_offset();

                    if next >= eof_offset {
                        return Err(ErrMode::from_error_kind(
                            remainder,
                            ErrorKind::EscapedTransform,
                        ));
                    } else {
                        match transform.parse_next(i.next_slice(next).0) {
                            Ok((i2, o)) => {
                                res.accumulate(o);
                                if i2.eof_offset() == 0 {
                                    return Ok((i.next_slice(i.eof_offset()).0, res));
                                } else {
                                    offset = input.offset_to(&i2);
                                }
                            }
                            Err(e) => return Err(e),
                        }
                    }
                } else {
                    if offset == 0 {
                        return Err(ErrMode::from_error_kind(
                            remainder,
                            ErrorKind::EscapedTransform,
                        ));
                    }
                    return Ok((remainder, res));
                }
            }
            Err(e) => return Err(e),
        }
    }
    Ok((input.next_slice(offset).0, res))
}
