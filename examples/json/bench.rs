use winnow::input::Streaming;

mod json;
mod parser;
mod parser_dispatch;
mod parser_streaming;

fn json_bench(c: &mut criterion::Criterion) {
  let data = [("small", SMALL), ("canada", CANADA)];
  let mut group = c.benchmark_group("json");
  for (name, sample) in data {
    let len = sample.len();
    group.throughput(criterion::Throughput::Bytes(len as u64));

    group.bench_with_input(
      criterion::BenchmarkId::new("complete", name),
      &len,
      |b, _| {
        type Error<'i> = winnow::error::Error<parser::Input<'i>>;

        b.iter(|| parser::json::<Error>(sample).unwrap());
      },
    );
    group.bench_with_input(
      criterion::BenchmarkId::new("dispatch", name),
      &len,
      |b, _| {
        type Error<'i> = winnow::error::Error<parser::Input<'i>>;

        b.iter(|| parser_dispatch::json::<Error>(sample).unwrap());
      },
    );
    group.bench_with_input(
      criterion::BenchmarkId::new("streaming", name),
      &len,
      |b, _| {
        type Error<'i> = winnow::error::Error<parser_streaming::Input<'i>>;

        b.iter(|| parser_streaming::json::<Error>(Streaming(sample)).unwrap());
      },
    );
  }
  group.finish();
}

const SMALL: &str = "  { \"a\"\t: 42,
  \"b\": [ \"x\", \"y\", 12 ,\"\\u2014\", \"\\uD83D\\uDE10\"] ,
  \"c\": { \"hello\" : \"world\"
  }
  }  ";

const CANADA: &str = include_str!("../../third_party/nativejson-benchmark/data/canada.json");

criterion::criterion_group!(benches, json_bench,);
criterion::criterion_main!(benches);
