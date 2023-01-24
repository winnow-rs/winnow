#[macro_use]
extern crate criterion;

#[global_allocator]
static ALLOC: jemallocator::Jemalloc = jemallocator::Jemalloc;

use criterion::Criterion;
use winnow::{
  branch::alt,
  bytes::one_of,
  character::{digit1, space0},
  multi::fold_many0,
  sequence::delimited,
  IResult, Parser,
};

// Parser definition

type Input<'i> = &'i [u8];

// We transform an integer string into a i64, ignoring surrounding whitespaces
// We look for a digit suite, and try to convert it.
// If there are no digits, we look for a parenthesized expression.
fn factor(input: Input<'_>) -> IResult<Input<'_>, i64> {
  delimited(
    space0,
    alt((
      digit1.map_res(|digits| unsafe { std::str::from_utf8_unchecked(digits) }.parse()),
      delimited(one_of('('), expr, one_of(')')),
    )),
    space0,
  )(input)
}

// We read an initial factor and for each time we find
// a * or / operator followed by another factor, we do
// the math by folding everything
fn term(input: Input<'_>) -> IResult<Input<'_>, i64> {
  let (input, init) = factor(input)?;
  fold_many0(
    (one_of("*/"), factor),
    move || init,
    |acc, (op, val)| {
      if op == b'*' {
        acc * val
      } else {
        acc / val
      }
    },
  )(input)
}

fn expr(input: Input<'_>) -> IResult<Input<'_>, i64> {
  let (input, init) = term(input)?;
  fold_many0(
    (one_of("+-"), term),
    move || init,
    |acc, (op, val)| {
      if op == b'+' {
        acc + val
      } else {
        acc - val
      }
    },
  )(input)
}

#[allow(clippy::eq_op, clippy::erasing_op)]
fn arithmetic(c: &mut Criterion) {
  let data = b"  2*2 / ( 5 - 1) + 3 / 4 * (2 - 7 + 567 *12 /2) + 3*(1+2*( 45 /2));";

  assert_eq!(
    expr(data),
    Ok((
      &b";"[..],
      2 * 2 / (5 - 1) + 3 / 4 * (2 - 7 + 567 * 12 / 2) + 3 * (1 + 2 * (45 / 2)),
    ))
  );
  c.bench_function("arithmetic", |b| {
    b.iter(|| expr(data).unwrap());
  });
}

criterion_group!(benches, arithmetic);
criterion_main!(benches);
