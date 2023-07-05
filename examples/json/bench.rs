use winnow::prelude::*;
use winnow::Partial;

mod json;
mod parser;
mod parser_dispatch;
mod parser_partial;

fn json_bench(c: &mut criterion::Criterion) {
    let data = [("small", SMALL), ("canada", CANADA)];
    let mut group = c.benchmark_group("json");
    for (name, sample) in data {
        let len = sample.len();
        group.throughput(criterion::Throughput::Bytes(len as u64));

        group.bench_with_input(criterion::BenchmarkId::new("basic", name), &len, |b, _| {
            type Error<'i> = winnow::error::Error<parser::Stream<'i>>;

            b.iter(|| parser::json::<Error>.parse_peek(sample).unwrap());
        });
        group.bench_with_input(criterion::BenchmarkId::new("unit", name), &len, |b, _| {
            type Error<'i> = ();

            b.iter(|| parser::json::<Error>.parse_peek(sample).unwrap());
        });
        group.bench_with_input(
            criterion::BenchmarkId::new("verbose", name),
            &len,
            |b, _| {
                type Error<'i> = winnow::error::VerboseError<parser::Stream<'i>>;

                b.iter(|| parser::json::<Error>.parse_peek(sample).unwrap());
            },
        );
        group.bench_with_input(
            criterion::BenchmarkId::new("dispatch", name),
            &len,
            |b, _| {
                type Error<'i> = winnow::error::Error<parser_dispatch::Stream<'i>>;

                b.iter(|| parser_dispatch::json::<Error>.parse_peek(sample).unwrap());
            },
        );
        group.bench_with_input(
            criterion::BenchmarkId::new("streaming", name),
            &len,
            |b, _| {
                type Error<'i> = winnow::error::Error<parser_partial::Stream<'i>>;

                b.iter(|| {
                    parser_partial::json::<Error>
                        .parse_peek(Partial::new(sample))
                        .unwrap()
                });
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
