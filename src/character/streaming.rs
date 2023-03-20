//! Character specific parsers and combinators, streaming version
//!
//! Functions recognizing specific characters

use crate::error::ErrMode;
use crate::error::ErrorKind;
use crate::error::Needed;
use crate::error::ParseError;
use crate::stream::{split_at_offset1_partial, split_at_offset_partial, AsBStr, AsChar, Stream};
use crate::stream::{Compare, CompareResult};
use crate::IResult;

pub(crate) fn char<I, Error: ParseError<I>>(c: char) -> impl Fn(I) -> IResult<I, char, Error>
where
    I: Stream,
    <I as Stream>::Token: AsChar,
{
    move |i: I| char_internal(i, c)
}

pub(crate) fn char_internal<I, Error: ParseError<I>>(i: I, c: char) -> IResult<I, char, Error>
where
    I: Stream,
    <I as Stream>::Token: AsChar,
{
    let (input, token) = i
        .next_token()
        .map(|(i, t)| (i, t.as_char()))
        .ok_or_else(|| ErrMode::Incomplete(Needed::new(1)))?;
    if c == token {
        Ok((input, token))
    } else {
        Err(ErrMode::Backtrack(Error::from_error_kind(
            i,
            ErrorKind::Char,
        )))
    }
}

pub(crate) fn crlf<T, E: ParseError<T>>(input: T) -> IResult<T, <T as Stream>::Slice, E>
where
    T: Stream,
    T: Compare<&'static str>,
{
    const CRLF: &str = "\r\n";
    match input.compare(CRLF) {
        CompareResult::Ok => Ok(input.next_slice(CRLF.len())),
        CompareResult::Incomplete => Err(ErrMode::Incomplete(Needed::new(CRLF.len()))),
        CompareResult::Error => {
            let e: ErrorKind = ErrorKind::CrLf;
            Err(ErrMode::from_error_kind(input, e))
        }
    }
}

pub(crate) fn not_line_ending<T, E: ParseError<T>>(input: T) -> IResult<T, <T as Stream>::Slice, E>
where
    T: Stream + AsBStr,
    T: Compare<&'static str>,
    <T as Stream>::Token: AsChar,
{
    match input.offset_for(|item| {
        let c = item.as_char();
        c == '\r' || c == '\n'
    }) {
        None => Err(ErrMode::Incomplete(Needed::Unknown)),
        Some(offset) => {
            let (new_input, res) = input.next_slice(offset);
            let bytes = new_input.as_bstr();
            let nth = bytes[0];
            if nth == b'\r' {
                let comp = new_input.compare("\r\n");
                match comp {
                    //FIXME: calculate the right index
                    CompareResult::Ok => {}
                    CompareResult::Incomplete => {
                        return Err(ErrMode::Incomplete(Needed::Unknown));
                    }
                    CompareResult::Error => {
                        let e: ErrorKind = ErrorKind::Tag;
                        return Err(ErrMode::from_error_kind(input, e));
                    }
                }
            }
            Ok((new_input, res))
        }
    }
}

pub(crate) fn line_ending<T, E: ParseError<T>>(input: T) -> IResult<T, <T as Stream>::Slice, E>
where
    T: Stream,
    T: Compare<&'static str>,
{
    const LF: &str = "\n";
    const CRLF: &str = "\r\n";
    match input.compare(LF) {
        CompareResult::Ok => Ok(input.next_slice(LF.len())),
        CompareResult::Incomplete => Err(ErrMode::Incomplete(Needed::new(1))),
        CompareResult::Error => match input.compare("\r\n") {
            CompareResult::Ok => Ok(input.next_slice(CRLF.len())),
            CompareResult::Incomplete => Err(ErrMode::Incomplete(Needed::new(2))),
            CompareResult::Error => Err(ErrMode::from_error_kind(input, ErrorKind::CrLf)),
        },
    }
}

pub(crate) fn newline<I, Error: ParseError<I>>(input: I) -> IResult<I, char, Error>
where
    I: Stream,
    <I as Stream>::Token: AsChar,
{
    char('\n')(input)
}

pub(crate) fn tab<I, Error: ParseError<I>>(input: I) -> IResult<I, char, Error>
where
    I: Stream,
    <I as Stream>::Token: AsChar,
{
    char('\t')(input)
}

pub(crate) fn alpha0<T, E: ParseError<T>>(input: T) -> IResult<T, <T as Stream>::Slice, E>
where
    T: Stream,
    <T as Stream>::Token: AsChar,
{
    split_at_offset_partial(&input, |item| !item.is_alpha())
}

pub(crate) fn alpha1<T, E: ParseError<T>>(input: T) -> IResult<T, <T as Stream>::Slice, E>
where
    T: Stream,
    <T as Stream>::Token: AsChar,
{
    split_at_offset1_partial(&input, |item| !item.is_alpha(), ErrorKind::Alpha)
}

pub(crate) fn digit0<T, E: ParseError<T>>(input: T) -> IResult<T, <T as Stream>::Slice, E>
where
    T: Stream,
    <T as Stream>::Token: AsChar,
{
    split_at_offset_partial(&input, |item| !item.is_dec_digit())
}

pub(crate) fn digit1<T, E: ParseError<T>>(input: T) -> IResult<T, <T as Stream>::Slice, E>
where
    T: Stream,
    <T as Stream>::Token: AsChar,
{
    split_at_offset1_partial(&input, |item| !item.is_dec_digit(), ErrorKind::Digit)
}

pub(crate) fn hex_digit0<T, E: ParseError<T>>(input: T) -> IResult<T, <T as Stream>::Slice, E>
where
    T: Stream,
    <T as Stream>::Token: AsChar,
{
    split_at_offset_partial(&input, |item| !item.is_hex_digit())
}

pub(crate) fn hex_digit1<T, E: ParseError<T>>(input: T) -> IResult<T, <T as Stream>::Slice, E>
where
    T: Stream,
    <T as Stream>::Token: AsChar,
{
    split_at_offset1_partial(&input, |item| !item.is_hex_digit(), ErrorKind::HexDigit)
}

pub(crate) fn oct_digit0<T, E: ParseError<T>>(input: T) -> IResult<T, <T as Stream>::Slice, E>
where
    T: Stream,
    <T as Stream>::Token: AsChar,
{
    split_at_offset_partial(&input, |item| !item.is_oct_digit())
}

pub(crate) fn oct_digit1<T, E: ParseError<T>>(input: T) -> IResult<T, <T as Stream>::Slice, E>
where
    T: Stream,
    <T as Stream>::Token: AsChar,
{
    split_at_offset1_partial(&input, |item| !item.is_oct_digit(), ErrorKind::OctDigit)
}

pub(crate) fn alphanumeric0<T, E: ParseError<T>>(input: T) -> IResult<T, <T as Stream>::Slice, E>
where
    T: Stream,
    <T as Stream>::Token: AsChar,
{
    split_at_offset_partial(&input, |item| !item.is_alphanum())
}

pub(crate) fn alphanumeric1<T, E: ParseError<T>>(input: T) -> IResult<T, <T as Stream>::Slice, E>
where
    T: Stream,
    <T as Stream>::Token: AsChar,
{
    split_at_offset1_partial(&input, |item| !item.is_alphanum(), ErrorKind::AlphaNumeric)
}

pub(crate) fn space0<T, E: ParseError<T>>(input: T) -> IResult<T, <T as Stream>::Slice, E>
where
    T: Stream,
    <T as Stream>::Token: AsChar,
{
    split_at_offset_partial(&input, |item| {
        let c = item.as_char();
        !(c == ' ' || c == '\t')
    })
}
pub(crate) fn space1<T, E: ParseError<T>>(input: T) -> IResult<T, <T as Stream>::Slice, E>
where
    T: Stream,
    <T as Stream>::Token: AsChar,
{
    split_at_offset1_partial(
        &input,
        |item| {
            let c = item.as_char();
            !(c == ' ' || c == '\t')
        },
        ErrorKind::Space,
    )
}

pub(crate) fn multispace0<T, E: ParseError<T>>(input: T) -> IResult<T, <T as Stream>::Slice, E>
where
    T: Stream,
    <T as Stream>::Token: AsChar,
{
    split_at_offset_partial(&input, |item| {
        let c = item.as_char();
        !(c == ' ' || c == '\t' || c == '\r' || c == '\n')
    })
}

pub(crate) fn multispace1<T, E: ParseError<T>>(input: T) -> IResult<T, <T as Stream>::Slice, E>
where
    T: Stream,
    <T as Stream>::Token: AsChar,
{
    split_at_offset1_partial(
        &input,
        |item| {
            let c = item.as_char();
            !(c == ' ' || c == '\t' || c == '\r' || c == '\n')
        },
        ErrorKind::MultiSpace,
    )
}
