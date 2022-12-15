#![allow(dead_code)]

use nom::bytes::tag;
use nom::character::digit1 as digit;
use nom::error::{ErrorKind, ParseError};
use nom::input::Streaming;
#[cfg(feature = "alloc")]
use nom::multi::count;
use nom::prelude::*;
use nom::sequence::terminated;
use nom::IResult;

#[derive(Debug)]
pub struct CustomError(String);

impl<'a> From<(&'a str, ErrorKind)> for CustomError {
  fn from(error: (&'a str, ErrorKind)) -> Self {
    CustomError(format!("error code was: {:?}", error))
  }
}

impl<'a> ParseError<Streaming<&'a str>> for CustomError {
  fn from_error_kind(_: Streaming<&'a str>, kind: ErrorKind) -> Self {
    CustomError(format!("error code was: {:?}", kind))
  }

  fn append(_: Streaming<&'a str>, kind: ErrorKind, other: CustomError) -> Self {
    CustomError(format!("{:?}\nerror code was: {:?}", other, kind))
  }
}

fn test1(input: Streaming<&str>) -> IResult<Streaming<&str>, &str, CustomError> {
  //fix_error!(input, CustomError, tag!("abcd"))
  tag("abcd")(input)
}

fn test2(input: Streaming<&str>) -> IResult<Streaming<&str>, &str, CustomError> {
  //terminated!(input, test1, fix_error!(CustomError, digit))
  terminated(test1, digit)(input)
}

fn test3(input: Streaming<&str>) -> IResult<Streaming<&str>, &str, CustomError> {
  test1.verify(|s: &str| s.starts_with("abcd")).parse(input)
}

#[cfg(feature = "alloc")]
fn test4(input: Streaming<&str>) -> IResult<Streaming<&str>, Vec<&str>, CustomError> {
  count(test1, 4)(input)
}
