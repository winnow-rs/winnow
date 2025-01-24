mod dispatch;
mod seq;

#[cfg(test)]
macro_rules! assert_parse(
  ($left: expr, $right: expr) => {
     let res: $crate::error::ModalResult<_, $crate::error::InputError<_>> = $left;
     snapbox::assert_data_eq!(snapbox::data::ToDebug::to_debug(&res), $right);
  };
);

#[cfg(test)]
mod tests;
