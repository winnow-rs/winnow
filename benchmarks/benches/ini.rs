#[global_allocator]
static ALLOC: jemallocator::Jemalloc = jemallocator::Jemalloc;

use criterion::*;

use nom::input::Located;
use nom::{
  bytes::{one_of, take_while},
  character::{alphanumeric1 as alphanumeric, multispace1 as multispace, space1 as space},
  combinator::opt,
  multi::many0,
  sequence::{delimited, separated_pair, terminated},
  IResult, Parser,
};
use std::collections::HashMap;
use std::str;

type Input<'i> = Located<&'i [u8]>;

fn category(i: Input<'_>) -> IResult<Input<'_>, &str> {
  delimited(one_of('['), take_while(|c| c != b']'), one_of(']'))
    .map_res(str::from_utf8)
    .parse(i)
}

fn key_value(i: Input<'_>) -> IResult<Input<'_>, (&str, &str)> {
  let (i, key) = alphanumeric.map_res(str::from_utf8).parse(i)?;
  let (i, _) = ((opt(space), one_of('='), opt(space))).parse(i)?;
  let (i, val) = take_while(|c| c != b'\n' && c != b';')
    .map_res(str::from_utf8)
    .parse(i)?;
  let (i, _) = opt((one_of(';'), take_while(|c| c != b'\n')))(i)?;
  Ok((i, (key, val)))
}

fn categories(i: Input<'_>) -> IResult<Input<'_>, HashMap<&str, HashMap<&str, &str>>> {
  many0(separated_pair(
    category,
    opt(multispace),
    many0(terminated(key_value, opt(multispace))).map(|vec: Vec<_>| vec.into_iter().collect()),
  ))
  .map(|vec: Vec<_>| vec.into_iter().collect())
  .parse(i)
}

fn bench_ini(c: &mut Criterion) {
  let str = "[owner]
name=John Doe
organization=Acme Widgets Inc.

[database]
server=192.0.2.62
port=143
file=payroll.dat
\0";

  let mut group = c.benchmark_group("ini");
  group.throughput(Throughput::Bytes(str.len() as u64));
  group.bench_function(BenchmarkId::new("parse", str.len()), |b| {
    b.iter(|| categories(Located::new(str.as_bytes())).unwrap())
  });
}

fn bench_ini_keys_and_values(c: &mut Criterion) {
  let str = "server=192.0.2.62
port=143
file=payroll.dat
\0";

  fn acc(i: Input<'_>) -> IResult<Input<'_>, Vec<(&str, &str)>> {
    many0(key_value)(i)
  }

  let mut group = c.benchmark_group("ini keys and values");
  group.throughput(Throughput::Bytes(str.len() as u64));
  group.bench_function(BenchmarkId::new("parse", str.len()), |b| {
    b.iter(|| acc(Located::new(str.as_bytes())).unwrap())
  });
}

fn bench_ini_key_value(c: &mut Criterion) {
  let str = "server=192.0.2.62\n";

  let mut group = c.benchmark_group("ini key value");
  group.throughput(Throughput::Bytes(str.len() as u64));
  group.bench_function(BenchmarkId::new("parse", str.len()), |b| {
    b.iter(|| key_value(Located::new(str.as_bytes())).unwrap())
  });
}

criterion_group!(
  benches,
  bench_ini,
  bench_ini_keys_and_values,
  bench_ini_key_value
);
criterion_main!(benches);
