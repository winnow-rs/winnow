//! Choice combinators

#[cfg(test)]
mod tests;

use crate::error::ErrorKind;
use crate::error::ParseError;
use crate::internal::{Err, IResult, Parser};

/// Tests a list of parsers one by one until one succeeds.
///
/// It takes as argument a tuple of parsers. There is a maximum of 21
/// parsers. If you need more, it is possible to nest them in other `alt` calls,
/// like this: `alt(parser_a, alt(parser_b, parser_c))`
///
/// ```rust
/// # use nom::error_position;
/// # use nom::{Err,error::ErrorKind, Needed, IResult};
/// use nom::character::complete::{alpha1, digit1};
/// use nom::branch::alt;
/// # fn main() {
/// fn parser(input: &str) -> IResult<&str, &str> {
///   alt((alpha1, digit1)).parse(input)
/// };
///
/// // the first parser, alpha1, recognizes the input
/// assert_eq!(parser("abc"), Ok(("", "abc")));
///
/// // the first parser returns an error, so alt tries the second one
/// assert_eq!(parser("123456"), Ok(("", "123456")));
///
/// // both parsers failed, and with the default error type, alt will return the last error
/// assert_eq!(parser(" "), Err(Err::Error(error_position!(" ", ErrorKind::Digit))));
/// # }
/// ```
///
/// With a custom error type, it is possible to have alt return the error of the parser
/// that went the farthest in the input data
pub fn alt<L, I, O, E>(choices: L) -> Alt<L, I, O, E> {
  Alt {
    choices,
    i: Default::default(),
    o: Default::default(),
    e: Default::default(),
  }
}

/// Implementation of [`alt`]
pub struct Alt<L, I, O, E> {
  choices: L,
  i: core::marker::PhantomData<I>,
  o: core::marker::PhantomData<O>,
  e: core::marker::PhantomData<E>,
}

// Manually implement Parser for (A,), the 1-tuple type
impl<P, I, O, E> Alt<(P,), I, O, E>
where
  P: Parser<I, O, E>,
{
  /// See [`Parser::parse`]
  pub fn parse(&mut self, input: I) -> IResult<I, O, E> {
    self.choices.0.parse(input)
  }
}

impl<P, I, O, E> Parser<I, O, E> for Alt<(P,), I, O, E>
where
  P: Parser<I, O, E>,
{
  fn parse(&mut self, input: I) -> IResult<I, O, E> {
    self.parse(input)
  }
}

macro_rules! impl_parser_alt(
  ($($id:ident)+) => (
    impl<
      PI, PO, PE,
      $($id: Parser<PI, PO, PE>),+
    > Alt<( $($id),+ ), PI, PO, PE>
    where
        PI: Clone,
        PE: ParseError<PI>
    {
      /// See [`Parser::parse`]
      pub fn parse(&mut self, input: PI) -> IResult<PI, PO, PE> {
        match self.choices.0.parse(input.clone()) {
          Err(Err::Error(e)) => impl_parser_alt_inner!(1, self, input, e, $($id)+),
          res => res,
        }
      }
    }

    impl<
      PI, PO, PE,
      $($id: Parser<PI, PO, PE>),+
    > Parser<PI, PO, PE> for Alt<( $($id),+ ), PI, PO, PE>
    where
        PI: Clone,
        PE: ParseError<PI>
    {
      fn parse(&mut self, input: PI) -> IResult<PI, PO, PE> {
        self.parse(input)
      }
    }
  );
);

macro_rules! impl_parser_alt_inner(
  ($it:tt, $self:expr, $input:expr, $err:expr, $head:ident $($id:ident)+) => (
    match $self.choices.$it.parse($input.clone()) {
      Err(Err::Error(e)) => {
        let err = $err.or(e);
        succ!($it, impl_parser_alt_inner!($self, $input, err, $($id)+))
      }
      res => res,
    }
  );
  ($it:tt, $self:expr, $input:expr, $err:expr, $head:ident) => (
    Err(Err::Error(PE::append($input, ErrorKind::Alt, $err)))
  );
);

macro_rules! alt_trait(
  ($first:ident $second:ident $($id: ident)+) => (
    alt_trait!(__impl $first $second; $($id)+);
  );
  (__impl $($current:ident)*; $head:ident $($id: ident)+) => (
    impl_parser_alt!($($current)*);

    alt_trait!(__impl $($current)* $head; $($id)+);
  );
  (__impl $($current:ident)*; $head:ident) => (
    impl_parser_alt!($($current)*);
    impl_parser_alt!($($current)* $head);
  );
);

alt_trait!(A B C D E F G H I J K L M N O P Q R S T U);

/// Applies a list of parsers in any order.
///
/// Permutation will succeed if all of the child parsers succeeded.
/// It takes as argument a tuple of parsers, and returns a
/// tuple of the parser results.
///
/// ```rust
/// # use nom::{Err,error::{Error, ErrorKind}, Needed, IResult, Parser};
/// use nom::character::complete::{alpha1, digit1};
/// use nom::branch::permutation;
/// # fn main() {
/// fn parser(input: &str) -> IResult<&str, (&str, &str)> {
///   permutation((alpha1, digit1)).parse(input)
/// }
///
/// // permutation recognizes alphabetic characters then digit
/// assert_eq!(parser("abc123"), Ok(("", ("abc", "123"))));
///
/// // but also in inverse order
/// assert_eq!(parser("123abc"), Ok(("", ("abc", "123"))));
///
/// // it will fail if one of the parsers failed
/// assert_eq!(parser("abc;"), Err(Err::Error(Error::new(";", ErrorKind::Digit))));
/// # }
/// ```
///
/// The parsers are applied greedily: if there are multiple unapplied parsers
/// that could parse the next slice of input, the first one is used.
/// ```rust
/// # use nom::{Err, error::{Error, ErrorKind}, IResult, Parser};
/// use nom::branch::permutation;
/// use nom::character::complete::{anychar, char};
///
/// fn parser(input: &str) -> IResult<&str, (char, char)> {
///   permutation((anychar, char('a'))).parse(input)
/// }
///
/// // anychar parses 'b', then char('a') parses 'a'
/// assert_eq!(parser("ba"), Ok(("", ('b', 'a'))));
///
/// // anychar parses 'a', then char('a') fails on 'b',
/// // even though char('a') followed by anychar would succeed
/// assert_eq!(parser("ab"), Err(Err::Error(Error::new("b", ErrorKind::Char))));
/// ```
///
pub fn permutation<L, I, E>(choices: L) -> Permutation<L, I, E> {
  Permutation {
    choices,
    i: Default::default(),
    e: Default::default(),
  }
}

/// Implementation of [`permutation`]
pub struct Permutation<L, I, E> {
  choices: L,
  i: core::marker::PhantomData<I>,
  e: core::marker::PhantomData<E>,
}

macro_rules! impl_parser_permutation(
  ($($parser:ident $output:ident $item:ident),+) => (
    impl<
      PI, PE,
      $($output),+ ,
      $($parser: Parser<PI, $output, PE>),+
    > Parser<PI, ( $($output),+ ), PE> for Permutation<( $($parser),+ ), PI, PE>
    where
        PI: Clone,
        PE: ParseError<PI>
    {
      fn parse(&mut self, mut input: PI) -> IResult<PI, ( $($output),+ ), PE> {
        let mut res = ($(Option::<$output>::None),+);

        loop {
          let mut err: Option<PE> = None;
          impl_parser_permutation_inner!(0, self, input, res, err, $($parser)+);

          // If we reach here, every iterator has either been applied before,
          // or errored on the remaining input
          if let Some(err) = err {
            // There are remaining parsers, and all errored on the remaining input
            return Err(Err::Error(PE::append(input, ErrorKind::Permutation, err)));
          }

          // All parsers were applied
          match res {
            ($(Some($item)),+) => return Ok((input, ($($item),+))),
            _ => unreachable!(),
          }
        }
      }
    }
  );
);

macro_rules! impl_parser_permutation_inner(
  ($it:tt, $self:expr, $input:ident, $res:expr, $err:expr, $head:ident $($id:ident)*) => (
    if $res.$it.is_none() {
      match $self.choices.$it.parse($input.clone()) {
        Ok((i, o)) => {
          $input = i;
          $res.$it = Some(o);
          continue;
        }
        Err(Err::Error(e)) => {
          $err = Some(match $err {
            Some(err) => err.or(e),
            None => e,
          });
        }
        Err(e) => return Err(e),
      };
    }
    succ!($it, impl_parser_permutation_inner!($self, $input, $res, $err, $($id)*));
  );
  ($it:tt, $self:expr, $input:ident, $res:expr, $err:expr,) => ();
);

macro_rules! permutation_trait(
  (
    $parser1:ident $output1:ident $item1:ident
    $parser2:ident $output2:ident $item2:ident
    $($parser3:ident $output3:ident $item3:ident)*
  ) => (
    permutation_trait!(__impl $parser1 $output1 $item1, $parser2 $output2 $item2; $($parser3 $output3 $item3)*);
  );
  (
    __impl $($parser:ident $output:ident $item:ident),+;
    $parser1:ident $output1:ident $item1:ident $($parser2:ident $output2:ident $item2:ident)*
  ) => (
    impl_parser_permutation!($($parser $output $item),+);
    permutation_trait!(__impl $($parser $output $item),+ , $parser1 $output1 $item1; $($parser2 $output2 $item2)*);
  );
  (__impl $($parser:ident $output:ident $item:ident),+;) => (
    impl_parser_permutation!($($parser $output $item),+);
  );
);

permutation_trait!(
  AParser AParserOutput a_value
  BParser BParserOutput b_value
  CParser CParserOutput c_value
  DParser DParserOutput d_value
  EParser EParserOutput e_value
  FParser FParserOutput f_value
  GParser GParserOutput g_value
  HParser HParserOutput h_value
  IParser IParserOutput i_value
  JParser JParserOutput j_value
  KParser KParserOutput k_value
  LParser LParserOutput l_value
  MParser MParserOutput m_value
  NParser NParserOutput n_value
  OParser OParserOutput o_value
  PParser PParserOutput p_value
  QParser QParserOutput q_value
  RParser RParserOutput r_value
  SParser SParserOutput s_value
  TParser TParserOutput t_value
  UParser UParserOutput u_value
);
