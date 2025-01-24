#[cfg(test)]
macro_rules! assert_parse(
  ($left: expr, $right: expr) => {
     let res: winnow::error::ModalResult<_, winnow::error::InputError<_>> = $left;
     snapbox::assert_data_eq!(snapbox::data::ToDebug::to_debug(&res), $right);
  };
);

type TestResult<I, O> = winnow::ModalResult<O, winnow::error::InputError<I>>;

automod::dir!("tests/testsuite");
