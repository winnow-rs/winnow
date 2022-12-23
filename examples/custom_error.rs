use nom8::error::ErrorKind;
use nom8::error::ParseError;
use nom8::Err::Error;
use nom8::IResult;

#[derive(Debug, PartialEq)]
pub enum CustomError<I> {
  MyError,
  Nom(I, ErrorKind),
}

impl<I> ParseError<I> for CustomError<I> {
  fn from_error_kind(input: I, kind: ErrorKind) -> Self {
    CustomError::Nom(input, kind)
  }

  fn append(_: I, _: ErrorKind, other: Self) -> Self {
    other
  }
}

pub fn parse(_input: &str) -> IResult<&str, &str, CustomError<&str>> {
  Err(Error(CustomError::MyError))
}

fn main() {}

#[cfg(test)]
mod tests {
  use super::parse;
  use super::CustomError;
  use nom8::Err::Error;

  #[test]
  fn it_works() {
    let err = parse("").unwrap_err();
    match err {
      Error(e) => assert_eq!(e, CustomError::MyError),
      _ => panic!("Unexpected error: {:?}", err),
    }
  }
}
