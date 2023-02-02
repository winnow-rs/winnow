use winnow::input::Streaming;

mod json;
mod parser;
mod parser_dispatch;
mod parser_streaming;

fn json_bench(c: &mut criterion::Criterion) {
  let data = "  { \"a\"\t: 42,
  \"b\": [ \"x\", \"y\", 12 ,\"\\u2014\", \"\\uD83D\\uDE10\"] ,
  \"c\": { \"hello\" : \"world\"
  }
  }  ";

  c.bench_function("compete", |b| {
    type Error<'i> = winnow::error::Error<parser::Input<'i>>;

    b.iter(|| parser::json::<Error>(data).unwrap());
  });
  c.bench_function("dispatch", |b| {
    type Error<'i> = winnow::error::Error<parser::Input<'i>>;

    b.iter(|| parser_dispatch::json::<Error>(data).unwrap());
  });
  c.bench_function("streaming", |b| {
    type Error<'i> = winnow::error::Error<parser_streaming::Input<'i>>;

    b.iter(|| parser_streaming::json::<Error>(Streaming(data)).unwrap());
  });
}

criterion::criterion_group!(benches, json_bench,);
criterion::criterion_main!(benches);
