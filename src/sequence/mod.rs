//! Combinators applying parsers in sequence

#[cfg(test)]
mod tests;

use crate::error::ParseError;
use crate::internal::{IResult, Parser};

/// Gets an object from the first parser,
/// then gets another object from the second parser.
///
/// # Arguments
/// * `first` The first parser to apply.
/// * `second` The second parser to apply.
///
/// # Example
/// ```rust
/// # use nom::{Err, error::ErrorKind, Needed};
/// # use nom::Needed::Size;
/// use nom::sequence::pair;
/// use nom::bytes::complete::tag;
///
/// let mut parser = pair(tag("abc"), tag("efg"));
///
/// assert_eq!(parser.parse("abcefg"), Ok(("", ("abc", "efg"))));
/// assert_eq!(parser.parse("abcefghij"), Ok(("hij", ("abc", "efg"))));
/// assert_eq!(parser.parse(""), Err(Err::Error(("", ErrorKind::Tag))));
/// assert_eq!(parser.parse("123"), Err(Err::Error(("123", ErrorKind::Tag))));
/// ```
///
/// This can also be written as:
/// ```rust
/// # use nom::{Err, error::ErrorKind, Needed, Parser};
/// # use nom::Needed::Size;
/// use nom::bytes::complete::tag;
///
/// let mut parser = (tag("abc"), tag("efg"));
///
/// assert_eq!(parser.parse("abcefg"), Ok(("", ("abc", "efg"))));
/// assert_eq!(parser.parse("abcefghij"), Ok(("hij", ("abc", "efg"))));
/// assert_eq!(parser.parse(""), Err(Err::Error(("", ErrorKind::Tag))));
/// assert_eq!(parser.parse("123"), Err(Err::Error(("123", ErrorKind::Tag))));
/// ```
pub fn pair<F, G, FO, GO, I, E>(first: F, second: G) -> Tuple<(F, G), I, (FO, GO), E> {
  tuple((first, second))
}

/// Matches an object from the first parser and discards it,
/// then gets an object from the second parser.
///
/// # Arguments
/// * `first` The opening parser.
/// * `second` The second parser to get object.
///
/// ```rust
/// # use nom::{Err, error::ErrorKind, Needed};
/// # use nom::Needed::Size;
/// use nom::sequence::preceded;
/// use nom::bytes::complete::tag;
///
/// let mut parser = preceded(tag("abc"), tag("efg"));
///
/// assert_eq!(parser.parse("abcefg"), Ok(("", "efg")));
/// assert_eq!(parser.parse("abcefghij"), Ok(("hij", "efg")));
/// assert_eq!(parser.parse(""), Err(Err::Error(("", ErrorKind::Tag))));
/// assert_eq!(parser.parse("123"), Err(Err::Error(("123", ErrorKind::Tag))));
/// ```
pub fn preceded<F, G, FO, I, O, E>(first: F, second: G) -> Preceded<F, G, FO, I, O, E> {
  Preceded {
    first,
    second,
    fo: Default::default(),
    i: Default::default(),
    o: Default::default(),
    e: Default::default(),
  }
}

/// Implementation of [`preceded`]
pub struct Preceded<F, G, FO, I, O, E> {
  first: F,
  second: G,
  fo: core::marker::PhantomData<FO>,
  i: core::marker::PhantomData<I>,
  o: core::marker::PhantomData<O>,
  e: core::marker::PhantomData<E>,
}

impl<F, G, FO, I, O, E> Preceded<F, G, FO, I, O, E>
where
  F: Parser<I, FO, E>,
  G: Parser<I, O, E>,
  E: ParseError<I>,
{
  /// See [`Parser::parse`]
  pub fn parse(&mut self, input: I) -> IResult<I, O, E> {
    let (input, _) = self.first.parse(input)?;
    self.second.parse(input)
  }
}

impl<F, G, FO, I, O, E> Parser<I, O, E> for Preceded<F, G, FO, I, O, E>
where
  F: Parser<I, FO, E>,
  G: Parser<I, O, E>,
  E: ParseError<I>,
{
  fn parse(&mut self, input: I) -> IResult<I, O, E> {
    self.parse(input)
  }
}

/// Gets an object from the first parser,
/// then matches an object from the second parser and discards it.
///
/// # Arguments
/// * `first` The first parser to apply.
/// * `second` The second parser to match an object.
///
/// ```rust
/// # use nom::{Err, error::ErrorKind, Needed};
/// # use nom::Needed::Size;
/// use nom::sequence::terminated;
/// use nom::bytes::complete::tag;
///
/// let mut parser = terminated(tag("abc"), tag("efg"));
///
/// assert_eq!(parser.parse("abcefg"), Ok(("", "abc")));
/// assert_eq!(parser.parse("abcefghij"), Ok(("hij", "abc")));
/// assert_eq!(parser.parse(""), Err(Err::Error(("", ErrorKind::Tag))));
/// assert_eq!(parser.parse("123"), Err(Err::Error(("123", ErrorKind::Tag))));
/// ```
pub fn terminated<F, G, GO, I, O, E>(first: F, second: G) -> Terminated<F, G, GO, I, O, E> {
  Terminated {
    first,
    second,
    go: Default::default(),
    i: Default::default(),
    o: Default::default(),
    e: Default::default(),
  }
}

/// Implementation of [`terminated`]
pub struct Terminated<F, G, GO, I, O, E> {
  first: F,
  second: G,
  go: core::marker::PhantomData<GO>,
  i: core::marker::PhantomData<I>,
  o: core::marker::PhantomData<O>,
  e: core::marker::PhantomData<E>,
}

impl<F, G, GO, I, O, E> Terminated<F, G, GO, I, O, E>
where
  F: Parser<I, O, E>,
  G: Parser<I, GO, E>,
  E: ParseError<I>,
{
  /// See [`Parser::parse`]
  pub fn parse(&mut self, input: I) -> IResult<I, O, E> {
    let (input, output) = self.first.parse(input)?;
    self.second.parse(input).map(|(i, _)| (i, output))
  }
}

impl<F, G, GO, I, O, E> Parser<I, O, E> for Terminated<F, G, GO, I, O, E>
where
  F: Parser<I, O, E>,
  G: Parser<I, GO, E>,
  E: ParseError<I>,
{
  fn parse(&mut self, input: I) -> IResult<I, O, E> {
    self.parse(input)
  }
}

/// Gets an object from the first parser,
/// then matches an object from the sep_parser and discards it,
/// then gets another object from the second parser.
///
/// # Arguments
/// * `first` The first parser to apply.
/// * `sep` The separator parser to apply.
/// * `second` The second parser to apply.
///
/// ```rust
/// # use nom::{Err, error::ErrorKind, Needed};
/// # use nom::Needed::Size;
/// use nom::sequence::separated_pair;
/// use nom::bytes::complete::tag;
///
/// let mut parser = separated_pair(tag("abc"), tag("|"), tag("efg"));
///
/// assert_eq!(parser.parse("abc|efg"), Ok(("", ("abc", "efg"))));
/// assert_eq!(parser.parse("abc|efghij"), Ok(("hij", ("abc", "efg"))));
/// assert_eq!(parser.parse(""), Err(Err::Error(("", ErrorKind::Tag))));
/// assert_eq!(parser.parse("123"), Err(Err::Error(("123", ErrorKind::Tag))));
/// ```
pub fn separated_pair<F, G, H, FO, GO, HO, I, E>(
  first: F,
  sep: G,
  second: H,
) -> SeparatedPair<F, G, H, FO, GO, HO, I, E> {
  SeparatedPair {
    first,
    sep,
    second,
    fo: Default::default(),
    go: Default::default(),
    ho: Default::default(),
    i: Default::default(),
    e: Default::default(),
  }
}

/// Implementation of [`separated_pair`]
pub struct SeparatedPair<F, G, H, FO, GO, HO, I, E> {
  first: F,
  sep: G,
  second: H,
  fo: core::marker::PhantomData<FO>,
  go: core::marker::PhantomData<GO>,
  ho: core::marker::PhantomData<HO>,
  i: core::marker::PhantomData<I>,
  e: core::marker::PhantomData<E>,
}

impl<F, G, H, FO, GO, HO, I, E> SeparatedPair<F, G, H, FO, GO, HO, I, E>
where
  F: Parser<I, FO, E>,
  G: Parser<I, GO, E>,
  H: Parser<I, HO, E>,
  E: ParseError<I>,
{
  /// See [`Parser::parse`]
  pub fn parse(&mut self, input: I) -> IResult<I, (FO, HO), E> {
    let (input, o1) = self.first.parse(input)?;
    let (input, _) = self.sep.parse(input)?;
    self.second.parse(input).map(|(i, o2)| (i, (o1, o2)))
  }
}

impl<F, G, H, FO, GO, HO, I, E> Parser<I, (FO, HO), E> for SeparatedPair<F, G, H, FO, GO, HO, I, E>
where
  F: Parser<I, FO, E>,
  G: Parser<I, GO, E>,
  H: Parser<I, HO, E>,
  E: ParseError<I>,
{
  fn parse(&mut self, input: I) -> IResult<I, (FO, HO), E> {
    self.parse(input)
  }
}

/// Matches an object from the first parser and discards it,
/// then gets an object from the second parser,
/// and finally matches an object from the third parser and discards it.
///
/// # Arguments
/// * `first` The first parser to apply and discard.
/// * `second` The second parser to apply.
/// * `third` The third parser to apply and discard.
///
/// ```rust
/// # use nom::{Err, error::ErrorKind, Needed};
/// # use nom::Needed::Size;
/// use nom::sequence::delimited;
/// use nom::bytes::complete::tag;
///
/// let mut parser = delimited(tag("("), tag("abc"), tag(")"));
///
/// assert_eq!(parser.parse("(abc)"), Ok(("", "abc")));
/// assert_eq!(parser.parse("(abc)def"), Ok(("def", "abc")));
/// assert_eq!(parser.parse(""), Err(Err::Error(("", ErrorKind::Tag))));
/// assert_eq!(parser.parse("123"), Err(Err::Error(("123", ErrorKind::Tag))));
/// ```
pub fn delimited<F, G, H, FO, HO, I, O, E>(
  first: F,
  second: G,
  third: H,
) -> Delimited<F, G, H, FO, HO, I, O, E> {
  Delimited {
    first,
    second,
    third,
    fo: Default::default(),
    ho: Default::default(),
    i: Default::default(),
    o: Default::default(),
    e: Default::default(),
  }
}

/// Implementation of [`delimited`]
pub struct Delimited<F, G, H, FO, HO, I, O, E> {
  first: F,
  second: G,
  third: H,
  fo: core::marker::PhantomData<FO>,
  ho: core::marker::PhantomData<HO>,
  i: core::marker::PhantomData<I>,
  o: core::marker::PhantomData<O>,
  e: core::marker::PhantomData<E>,
}

impl<F, G, H, FO, HO, I, O, E> Delimited<F, G, H, FO, HO, I, O, E>
where
  F: Parser<I, FO, E>,
  G: Parser<I, O, E>,
  H: Parser<I, HO, E>,
  E: ParseError<I>,
{
  /// See [`Parser::parse`]
  pub fn parse(&mut self, input: I) -> IResult<I, O, E> {
    let (input, _) = self.first.parse(input)?;
    let (input, o2) = self.second.parse(input)?;
    self.third.parse(input).map(|(i, _)| (i, o2))
  }
}

impl<F, G, H, FO, HO, I, O, E> Parser<I, O, E> for Delimited<F, G, H, FO, HO, I, O, E>
where
  F: Parser<I, FO, E>,
  G: Parser<I, O, E>,
  H: Parser<I, HO, E>,
  E: ParseError<I>,
{
  fn parse(&mut self, input: I) -> IResult<I, O, E> {
    self.parse(input)
  }
}

/// Applies a tuple of parsers one by one and returns their results as a tuple.
///
/// There is a maximum of 21 parsers
///
/// # Example
/// ```rust
/// # use nom::{Err, error::ErrorKind};
/// use nom::sequence::tuple;
/// use nom::character::complete::{alpha1, digit1};
/// let mut parser = tuple((alpha1, digit1, alpha1));
///
/// assert_eq!(parser.parse("abc123def"), Ok(("", ("abc", "123", "def"))));
/// assert_eq!(parser.parse("123def"), Err(Err::Error(("123def", ErrorKind::Alpha))));
/// ```
///
/// This can also be written as:
/// ```rust
/// # use nom::{Err, error::ErrorKind, Parser};
/// use nom::character::complete::{alpha1, digit1};
/// let mut parser = (alpha1, digit1, alpha1);
///
/// assert_eq!(parser.parse("abc123def"), Ok(("", ("abc", "123", "def"))));
/// assert_eq!(parser.parse("123def"), Err(Err::Error(("123def", ErrorKind::Alpha))));
/// ```
pub fn tuple<P, PI, PO, PE>(sequence: P) -> Tuple<P, PI, PO, PE> {
  Tuple {
    sequence,
    pi: Default::default(),
    po: Default::default(),
    pe: Default::default(),
  }
}

/// Implementation of [`tuple()`]
pub struct Tuple<P, PI, PO, PE> {
  sequence: P,
  pi: core::marker::PhantomData<PI>,
  po: core::marker::PhantomData<PO>,
  pe: core::marker::PhantomData<PE>,
}

impl<P, PI, PO, PE> Tuple<P, PI, PO, PE>
where
  P: Parser<PI, PO, PE>,
{
  /// See [`Parser::parse`]
  pub fn parse(&mut self, input: PI) -> IResult<PI, PO, PE> {
    self.sequence.parse(input)
  }
}

impl<P, PI, PO, PE> Parser<PI, PO, PE> for Tuple<P, PI, PO, PE>
where
  P: Parser<PI, PO, PE>,
{
  fn parse(&mut self, input: PI) -> IResult<PI, PO, PE> {
    self.parse(input)
  }
}

// Special case: implement `Tuple` for `()`, the unit type.
// This can come up in macros which accept a variable number of arguments.
// Literally, `()` is an empty tuple, so it should simply parse nothing.
impl<PI, PE> Parser<PI, (), PE> for ()
where
  PE: ParseError<PI>,
{
  fn parse(&mut self, input: PI) -> IResult<PI, (), PE> {
    Ok((input, ()))
  }
}

impl<A, AO, PI, PE> Parser<PI, (AO,), PE> for (A,)
where
  A: Parser<PI, AO, PE>,
  PE: ParseError<PI>,
{
  fn parse(&mut self, input: PI) -> IResult<PI, (AO,), PE> {
    self.0.parse(input).map(|(i, o)| (i, (o,)))
  }
}

macro_rules! impl_tuple(
  ($($name:ident $ty: ident),+) => (
    impl<
      $($ty),+ ,
      $($name: Parser<PI, $ty, PE>),+ ,
      PI, PE,
    > Parser<PI, ( $($ty),+ ), PE> for ( $($name),+ )
    where
        PI: Clone,
        PE: ParseError<PI>,
    {
      fn parse(&mut self, input: PI) -> IResult<PI, ( $($ty),+ ), PE> {
        impl_tuple_inner!(0, self, input, (), $($name)+)
      }
    }
  );
);

macro_rules! impl_tuple_inner(
  ($it:tt, $self:expr, $input:expr, (), $head:ident $($id:ident)+) => ({
    let (i, o) = $self.$it.parse($input.clone())?;

    succ!($it, impl_tuple_inner!($self, i, ( o ), $($id)+))
  });
  ($it:tt, $self:expr, $input:expr, ($($parsed:tt)*), $head:ident $($id:ident)+) => ({
    let (i, o) = $self.$it.parse($input.clone())?;

    succ!($it, impl_tuple_inner!($self, i, ($($parsed)* , o), $($id)+))
  });
  ($it:tt, $self:expr, $input:expr, ($($parsed:tt)*), $head:ident) => ({
    let (i, o) = $self.$it.parse($input.clone())?;

    Ok((i, ($($parsed)* , o)))
  });
);

macro_rules! tuple_trait(
  ($name1:ident $ty1:ident, $name2: ident $ty2:ident, $($name:ident $ty:ident),*) => (
    tuple_trait!(__impl $name1 $ty1, $name2 $ty2; $($name $ty),*);
  );
  (__impl $($name:ident $ty: ident),+; $name1:ident $ty1:ident, $($name2:ident $ty2:ident),*) => (
    impl_tuple!($($name $ty),+);
    tuple_trait!(__impl $($name $ty),+ , $name1 $ty1; $($name2 $ty2),*);
  );
  (__impl $($name:ident $ty: ident),+; $name1:ident $ty1:ident) => (
    impl_tuple!($($name $ty),+);
    impl_tuple!($($name $ty),+, $name1 $ty1);
  );
);

tuple_trait!(FnA A, FnB B, FnC C, FnD D, FnE E, FnF F, FnG G, FnH H, FnI I, FnJ J, FnK K, FnL L,
  FnM M, FnN N, FnO O, FnP P, FnQ Q, FnR R, FnS S, FnT T, FnU U);
