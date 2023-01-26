#![cfg(feature = "alloc")]

mod parser;

use winnow::error::convert_error;
use winnow::error::Error;
use winnow::error::VerboseError;
use winnow::prelude::*;

use parser::json;

fn main() -> Result<(), lexopt::Error> {
  let args = Args::parse()?;

  let data = args.input.as_deref().unwrap_or(if args.invalid {
    "  { \"a\"\t: 42,
  \"b\": [ \"x\", \"y\", 12 ] ,
  \"c\": { 1\"hello\" : \"world\"
  }
  } "
  } else {
    "  { \"a\"\t: 42,
  \"b\": [ \"x\", \"y\", 12 ] ,
  \"c\": { \"hello\" : \"world\"
  }
  } "
  });

  if args.verbose {
    match json::<VerboseError<&str>>(data).finish() {
      Ok(json) => {
        println!("{:#?}", json);
      }
      Err(err) => {
        if args.pretty {
          println!("{}", convert_error(data, err));
        } else {
          println!("{:#?}", err);
        }
      }
    }
  } else {
    match json::<Error<&str>>(data).finish() {
      Ok(json) => {
        println!("{:#?}", json);
      }
      Err(err) => {
        println!("{:?}", err);
      }
    }
  }

  Ok(())
}

#[derive(Default)]
struct Args {
  input: Option<String>,
  invalid: bool,
  verbose: bool,
  pretty: bool,
}

impl Args {
  fn parse() -> Result<Self, lexopt::Error> {
    use lexopt::prelude::*;

    let mut res = Args::default();

    let mut args = lexopt::Parser::from_env();
    while let Some(arg) = args.next()? {
      match arg {
        Long("invalid") => {
          res.invalid = true;
        }
        Long("verbose") => {
          res.verbose = true;
          // Only case where verbose matters
          res.invalid = true;
        }
        Long("pretty") => {
          res.verbose = true;
          // Only case where pretty matters
          res.pretty = true;
          res.invalid = true;
        }
        Value(input) => {
          res.input = Some(input.string()?);
        }
        _ => return Err(arg.unexpected()),
      }
    }
    Ok(res)
  }
}
