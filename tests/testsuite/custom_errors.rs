#![allow(dead_code)]

use winnow::ascii::digit1 as digit;
#[cfg(feature = "alloc")]
use winnow::combinator::repeat;
use winnow::combinator::terminated;
use winnow::error::IResult;
use winnow::error::{ErrorKind, ParserError};
use winnow::prelude::*;
use winnow::stream::Stream;
use winnow::unpeek;
use winnow::Partial;

#[derive(Debug)]
pub(crate) struct CustomError(String);

impl<'a> From<(&'a str, ErrorKind)> for CustomError {
    fn from(error: (&'a str, ErrorKind)) -> Self {
        CustomError(format!("error code was: {error:?}"))
    }
}

impl<'a> ParserError<Partial<&'a str>> for CustomError {
    fn from_error_kind(_: &Partial<&'a str>, kind: ErrorKind) -> Self {
        CustomError(format!("error code was: {kind:?}"))
    }

    fn append(
        self,
        _: &Partial<&'a str>,
        _: &<Partial<&'a str> as Stream>::Checkpoint,
        kind: ErrorKind,
    ) -> Self {
        CustomError(format!("{self:?}\nerror code was: {kind:?}"))
    }
}

fn test1(input: Partial<&str>) -> IResult<Partial<&str>, &str, CustomError> {
    //fix_error!(input, CustomError, tag!("abcd"))
    "abcd".parse_peek(input)
}

fn test2(input: Partial<&str>) -> IResult<Partial<&str>, &str, CustomError> {
    //terminated!(input, test1, fix_error!(CustomError, digit))
    terminated(unpeek(test1), digit).parse_peek(input)
}

fn test3(input: Partial<&str>) -> IResult<Partial<&str>, &str, CustomError> {
    unpeek(test1)
        .verify(|s: &str| s.starts_with("abcd"))
        .parse_peek(input)
}

#[cfg(feature = "alloc")]
fn test4(input: Partial<&str>) -> IResult<Partial<&str>, Vec<&str>, CustomError> {
    repeat(4, unpeek(test1)).parse_peek(input)
}
