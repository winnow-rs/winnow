#[macro_use]
extern crate criterion;

#[global_allocator]
static ALLOC: jemallocator::Jemalloc = jemallocator::Jemalloc;

use criterion::Criterion;
use winnow::number::be_u64;

type Input<'i> = &'i [u8];

fn parser(i: Input<'_>) -> winnow::IResult<Input<'_>, u64> {
  be_u64(i)
}

fn number(c: &mut Criterion) {
  let data = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12];

  parser(&data[..]).expect("should parse correctly");
  c.bench_function("number", move |b| {
    b.iter(|| parser(&data[..]).unwrap());
  });
}

criterion_group!(benches, number);
criterion_main!(benches);
