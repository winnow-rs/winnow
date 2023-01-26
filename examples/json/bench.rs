use winnow::error::ErrorKind;
use winnow::input::ParseTo;
use winnow::prelude::*;
use winnow::Err;

mod parser;

use parser::json;

type Error<'i> = winnow::error::Error<&'i str>;

fn json_bench(c: &mut criterion::Criterion) {
  let data = "  { \"a\"\t: 42,
  \"b\": [ \"x\", \"y\", 12 ,\"\\u2014\", \"\\uD83D\\uDE10\"] ,
  \"c\": { \"hello\" : \"world\"
  }
  }  ";

  c.bench_function("json", |b| {
    b.iter(|| json::<Error>(data).unwrap());
  });
}

fn recognize_float_bytes(c: &mut criterion::Criterion) {
  println!(
    "recognize_float_bytes result: {:?}",
    recognize_float::<_, (_, ErrorKind), false>(&b"-1.234E-12"[..])
  );
  c.bench_function("recognize float bytes", |b| {
    b.iter(|| recognize_float::<_, (_, ErrorKind), false>(&b"-1.234E-12"[..]));
  });
}

fn recognize_float_str(c: &mut criterion::Criterion) {
  println!(
    "recognize_float_str result: {:?}",
    recognize_float::<_, (_, ErrorKind), false>("-1.234E-12")
  );
  c.bench_function("recognize float str", |b| {
    b.iter(|| recognize_float::<_, (_, ErrorKind), false>("-1.234E-12"));
  });
}

fn float_bytes(c: &mut criterion::Criterion) {
  println!(
    "float_bytes result: {:?}",
    f64::<_, (_, ErrorKind), false>(&b"-1.234E-12"[..])
  );
  c.bench_function("float bytes", |b| {
    b.iter(|| f64::<_, (_, ErrorKind), false>(&b"-1.234E-12"[..]));
  });
}

fn float_str(c: &mut criterion::Criterion) {
  println!(
    "float_str result: {:?}",
    f64::<_, (_, ErrorKind), false>("-1.234E-12")
  );
  c.bench_function("float str", |b| {
    b.iter(|| f64::<_, (_, ErrorKind), false>("-1.234E-12"));
  });
}

fn std_float(input: &[u8]) -> IResult<&[u8], f64, (&[u8], ErrorKind)> {
  match recognize_float(input) {
    Err(e) => Err(e),
    Ok((i, s)) => match s.parse_to() {
      Some(n) => Ok((i, n)),
      None => Err(Err::Error((i, ErrorKind::Float))),
    },
  }
}

fn std_float_bytes(c: &mut criterion::Criterion) {
  println!(
    "std_float_bytes result: {:?}",
    std_float(&b"-1.234E-12"[..])
  );
  c.bench_function("std_float bytes", |b| {
    b.iter(|| std_float(&b"-1.234E-12"[..]));
  });
}

criterion::criterion_group!(
  benches,
  json_bench,
  recognize_float_bytes,
  recognize_float_str,
  float_bytes,
  std_float_bytes,
  float_str
);
criterion::criterion_main!(benches);
