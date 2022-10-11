use crate::Err;
use crate::IResult;
use crate::Parser;

/// Implementation of `Parser::map`
#[cfg_attr(nightly, warn(rustdoc::missing_doc_code_examples))]
pub struct Map<F, G, O1> {
  f: F,
  g: G,
  phantom: core::marker::PhantomData<O1>,
}

impl<F, G, O1> Map<F, G, O1> {
  pub(crate) fn new(f: F, g: G) -> Self {
    Self {
      f,
      g,
      phantom: Default::default(),
    }
  }
}

impl<'a, I, O1, O2, E, F: Parser<I, O1, E>, G: Fn(O1) -> O2> Parser<I, O2, E> for Map<F, G, O1> {
  fn parse(&mut self, i: I) -> IResult<I, O2, E> {
    match self.f.parse(i) {
      Err(e) => Err(e),
      Ok((i, o)) => Ok((i, (self.g)(o))),
    }
  }
}

/// Implementation of `Parser::flat_map`
#[cfg_attr(nightly, warn(rustdoc::missing_doc_code_examples))]
pub struct FlatMap<F, G, O1> {
  f: F,
  g: G,
  phantom: core::marker::PhantomData<O1>,
}

impl<F, G, O1> FlatMap<F, G, O1> {
  pub(crate) fn new(f: F, g: G) -> Self {
    Self {
      f,
      g,
      phantom: Default::default(),
    }
  }
}

impl<'a, I, O1, O2, E, F: Parser<I, O1, E>, G: Fn(O1) -> H, H: Parser<I, O2, E>> Parser<I, O2, E>
  for FlatMap<F, G, O1>
{
  fn parse(&mut self, i: I) -> IResult<I, O2, E> {
    let (i, o1) = self.f.parse(i)?;
    (self.g)(o1).parse(i)
  }
}

/// Implementation of `Parser::and_then`
#[cfg_attr(nightly, warn(rustdoc::missing_doc_code_examples))]
pub struct AndThen<F, G, O1> {
  f: F,
  g: G,
  phantom: core::marker::PhantomData<O1>,
}

impl<F, G, O1> AndThen<F, G, O1> {
  pub(crate) fn new(f: F, g: G) -> Self {
    Self {
      f,
      g,
      phantom: Default::default(),
    }
  }
}

impl<'a, I, O1, O2, E, F: Parser<I, O1, E>, G: Parser<O1, O2, E>> Parser<I, O2, E>
  for AndThen<F, G, O1>
{
  fn parse(&mut self, i: I) -> IResult<I, O2, E> {
    let (i, o1) = self.f.parse(i)?;
    let (_, o2) = self.g.parse(o1)?;
    Ok((i, o2))
  }
}

/// Implementation of `Parser::and`
#[cfg_attr(nightly, warn(rustdoc::missing_doc_code_examples))]
pub struct And<F, G> {
  f: F,
  g: G,
}

impl<F, G> And<F, G> {
  pub(crate) fn new(f: F, g: G) -> Self {
    Self { f, g }
  }
}

impl<'a, I, O1, O2, E, F: Parser<I, O1, E>, G: Parser<I, O2, E>> Parser<I, (O1, O2), E>
  for And<F, G>
{
  fn parse(&mut self, i: I) -> IResult<I, (O1, O2), E> {
    let (i, o1) = self.f.parse(i)?;
    let (i, o2) = self.g.parse(i)?;
    Ok((i, (o1, o2)))
  }
}

/// Implementation of `Parser::or`
#[cfg_attr(nightly, warn(rustdoc::missing_doc_code_examples))]
pub struct Or<F, G> {
  f: F,
  g: G,
}

impl<F, G> Or<F, G> {
  pub(crate) fn new(f: F, g: G) -> Self {
    Self { f, g }
  }
}

impl<'a, I: Clone, O, E: crate::error::ParseError<I>, F: Parser<I, O, E>, G: Parser<I, O, E>>
  Parser<I, O, E> for Or<F, G>
{
  fn parse(&mut self, i: I) -> IResult<I, O, E> {
    match self.f.parse(i.clone()) {
      Err(Err::Error(e1)) => match self.g.parse(i) {
        Err(Err::Error(e2)) => Err(Err::Error(e1.or(e2))),
        res => res,
      },
      res => res,
    }
  }
}

/// Implementation of `Parser::into`
#[cfg_attr(nightly, warn(rustdoc::missing_doc_code_examples))]
pub struct Into<F, O1, O2: From<O1>, E1, E2: From<E1>> {
  f: F,
  phantom_out1: core::marker::PhantomData<O1>,
  phantom_err1: core::marker::PhantomData<E1>,
  phantom_out2: core::marker::PhantomData<O2>,
  phantom_err2: core::marker::PhantomData<E2>,
}

impl<F, O1, O2: From<O1>, E1, E2: From<E1>> Into<F, O1, O2, E1, E2> {
  pub(crate) fn new(f: F) -> Self {
    Self {
      f,
      phantom_out1: Default::default(),
      phantom_err1: Default::default(),
      phantom_out2: Default::default(),
      phantom_err2: Default::default(),
    }
  }
}

impl<
    'a,
    I: Clone,
    O1,
    O2: From<O1>,
    E1,
    E2: crate::error::ParseError<I> + From<E1>,
    F: Parser<I, O1, E1>,
  > Parser<I, O2, E2> for Into<F, O1, O2, E1, E2>
{
  fn parse(&mut self, i: I) -> IResult<I, O2, E2> {
    match self.f.parse(i) {
      Ok((i, o)) => Ok((i, o.into())),
      Err(Err::Error(e)) => Err(Err::Error(e.into())),
      Err(Err::Failure(e)) => Err(Err::Failure(e.into())),
      Err(Err::Incomplete(e)) => Err(Err::Incomplete(e)),
    }
  }
}
