#[macro_use]
extern crate criterion;

use criterion::Criterion;

use winnow::character::{f64, recognize_float};
use winnow::error::ErrMode;
use winnow::error::Error;
use winnow::error::ErrorKind;
use winnow::input::ParseTo;
use winnow::number::be_u64;
use winnow::prelude::*;

type Input<'i> = &'i [u8];

fn parser(i: Input<'_>) -> IResult<Input<'_>, u64> {
  be_u64(i)
}

fn number(c: &mut Criterion) {
  let data = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12];

  parser(&data[..]).expect("should parse correctly");
  c.bench_function("number", move |b| {
    b.iter(|| parser(&data[..]).unwrap());
  });
}

fn recognize_float_bytes(c: &mut Criterion) {
  println!(
    "recognize_float_bytes result: {:?}",
    recognize_float::<_, Error<_>, false>(&b"-1.234E-12"[..])
  );
  c.bench_function("recognize float bytes", |b| {
    b.iter(|| recognize_float::<_, Error<_>, false>(&b"-1.234E-12"[..]));
  });
}

fn recognize_float_str(c: &mut Criterion) {
  println!(
    "recognize_float_str result: {:?}",
    recognize_float::<_, Error<_>, false>("-1.234E-12")
  );
  c.bench_function("recognize float str", |b| {
    b.iter(|| recognize_float::<_, Error<_>, false>("-1.234E-12"));
  });
}

fn float_bytes(c: &mut Criterion) {
  println!(
    "float_bytes result: {:?}",
    f64::<_, Error<_>, false>(&b"-1.234E-12"[..])
  );
  c.bench_function("float bytes", |b| {
    b.iter(|| f64::<_, Error<_>, false>(&b"-1.234E-12"[..]));
  });
}

fn float_str(c: &mut Criterion) {
  println!(
    "float_str result: {:?}",
    f64::<_, Error<_>, false>("-1.234E-12")
  );
  c.bench_function("float str", |b| {
    b.iter(|| f64::<_, Error<_>, false>("-1.234E-12"));
  });
}

fn std_float(input: &[u8]) -> IResult<&[u8], f64, Error<&[u8]>> {
  match recognize_float(input) {
    Err(e) => Err(e),
    Ok((i, s)) => match s.parse_to() {
      Some(n) => Ok((i, n)),
      None => Err(ErrMode::Backtrack(Error {
        input: i,
        kind: ErrorKind::Float,
      })),
    },
  }
}

fn std_float_bytes(c: &mut Criterion) {
  println!(
    "std_float_bytes result: {:?}",
    std_float(&b"-1.234E-12"[..])
  );
  c.bench_function("std_float bytes", |b| {
    b.iter(|| std_float(&b"-1.234E-12"[..]));
  });
}

criterion_group!(
  benches,
  number,
  recognize_float_bytes,
  recognize_float_str,
  float_bytes,
  std_float_bytes,
  float_str
);
criterion_main!(benches);
