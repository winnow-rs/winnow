#![allow(dead_code)]

use winnow::character::digit1 as digit;
use winnow::error::{ErrorKind, ParseError};
#[cfg(feature = "alloc")]
use winnow::multi::count;
use winnow::prelude::*;
use winnow::sequence::terminated;
use winnow::IResult;
use winnow::Partial;

#[derive(Debug)]
pub struct CustomError(String);

impl<'a> From<(&'a str, ErrorKind)> for CustomError {
    fn from(error: (&'a str, ErrorKind)) -> Self {
        CustomError(format!("error code was: {:?}", error))
    }
}

impl<'a> ParseError<Partial<&'a str>> for CustomError {
    fn from_error_kind(_: Partial<&'a str>, kind: ErrorKind) -> Self {
        CustomError(format!("error code was: {:?}", kind))
    }

    fn append(self, _: Partial<&'a str>, kind: ErrorKind) -> Self {
        CustomError(format!("{:?}\nerror code was: {:?}", self, kind))
    }
}

fn test1(input: Partial<&str>) -> IResult<Partial<&str>, &str, CustomError> {
    //fix_error!(input, CustomError, tag!("abcd"))
    "abcd".parse_next(input)
}

fn test2(input: Partial<&str>) -> IResult<Partial<&str>, &str, CustomError> {
    //terminated!(input, test1, fix_error!(CustomError, digit))
    terminated(test1, digit).parse_next(input)
}

fn test3(input: Partial<&str>) -> IResult<Partial<&str>, &str, CustomError> {
    test1
        .verify(|s: &str| s.starts_with("abcd"))
        .parse_next(input)
}

#[cfg(feature = "alloc")]
fn test4(input: Partial<&str>) -> IResult<Partial<&str>, Vec<&str>, CustomError> {
    count(test1, 4).parse_next(input)
}
