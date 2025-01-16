mod dispatch;
mod seq;

#[cfg(test)]
macro_rules! assert_parse(
  ($left: expr, $right: expr) => {
    let res: $crate::error::IResult<_, _, InputError<_>> = $left;
    assert_eq!(res, $right);
  };
);

#[cfg(test)]
mod tests;
