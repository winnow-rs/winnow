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

criterion::criterion_group!(benches, json_bench,);
criterion::criterion_main!(benches);
