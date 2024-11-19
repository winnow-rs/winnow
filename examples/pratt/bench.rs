mod parser;

use criterion::black_box;
use winnow::prelude::*;

fn pratt(c: &mut criterion::Criterion) {
    let i =
        "a = 2*-2 * (a ? 1 + 2 * 4 - --a.bar + 2 : 2) / ( &**foo.a->p! -+1) + 3^1 / 4 == 1 * (2 - 7 + 567 *12 /2) + 3*(1+2*( 45 /2))";

    // let mut group = c.benchmark_group("pratt");
    //
    // parser::pratt_parser.parse(i).expect("pratt should parse");
    // parser::shunting_yard_parser
    //     .parse(i)
    //     .expect("shunting yard should parse");
    //
    // group.bench_function("pratt", |b| {
    //     b.iter(|| black_box(parser::pratt_parser.parse(i).unwrap()));
    // });
    //
    // group.bench_function("shunting_yard", |b| {
    //     b.iter(|| black_box(parser::shunting_yard_parser.parse(i).unwrap()));
    // });
}

criterion::criterion_group!(benches, pratt);
criterion::criterion_main!(benches);
