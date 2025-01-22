#[cfg(test)]
macro_rules! assert_parse(
  ($left: expr, $right: expr) => {
     let res: winnow::error::PResult<_, winnow::error::TreeError<_>> = $left;
     snapbox::assert_data_eq!(snapbox::data::ToDebug::to_debug(&res), $right);
  };
);

type TestResult<I, O> = winnow::PResult<O, winnow::error::TreeError<I>>;

automod::dir!("tests/testsuite");
