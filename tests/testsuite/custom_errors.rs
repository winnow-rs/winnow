#![allow(dead_code)]

use winnow::ascii::digit1 as digit;
#[cfg(feature = "alloc")]
use winnow::combinator::repeat;
use winnow::combinator::terminated;
use winnow::error::ParserError;
use winnow::prelude::*;
use winnow::stream::Stream;
use winnow::Partial;

#[derive(Debug)]
pub(crate) struct CustomError(String);

impl<'a> ParserError<Partial<&'a str>> for CustomError {
    type Inner = Self;

    fn from_input(_: &Partial<&'a str>) -> Self {
        CustomError("error".to_owned())
    }

    fn append(self, _: &Partial<&'a str>, _: &<Partial<&'a str> as Stream>::Checkpoint) -> Self {
        self
    }

    fn into_inner(self) -> Result<Self::Inner, Self> {
        Ok(self)
    }
}

fn test1<'i>(input: &mut Partial<&'i str>) -> ModalResult<&'i str, CustomError> {
    //fix_error!(input, CustomError, tag!("abcd"))
    "abcd".parse_next(input)
}

fn test2<'i>(input: &mut Partial<&'i str>) -> ModalResult<&'i str, CustomError> {
    //terminated!(input, test1, fix_error!(CustomError, digit))
    terminated(test1, digit).parse_next(input)
}

fn test3<'i>(input: &mut Partial<&'i str>) -> ModalResult<&'i str, CustomError> {
    test1
        .verify(|s: &str| s.starts_with("abcd"))
        .parse_next(input)
}

#[cfg(feature = "alloc")]
fn test4<'i>(input: &mut Partial<&'i str>) -> ModalResult<Vec<&'i str>, CustomError> {
    repeat(4, test1).parse_next(input)
}
