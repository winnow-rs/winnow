#[cfg(test)]
macro_rules! assert_parse(
  ($left: expr, $right: expr) => {
     let res: winnow::error::IResult<_, _, InputError<_>> = $left;
     snapbox::assert_data_eq!(snapbox::data::ToDebug::to_debug(&res), $right);
  };
);

automod::dir!("tests/testsuite");
