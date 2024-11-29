#[macro_use]
extern crate criterion;

use criterion::black_box;
use criterion::Criterion;

use winnow::ascii::digit1;
use winnow::combinator::delimited;
use winnow::combinator::empty;
use winnow::combinator::fail;
use winnow::combinator::peek;
use winnow::dispatch;
use winnow::token::any;
use winnow::PResult;
use winnow::Parser;

type Stream<'i> = &'i [u8];

static CORPUS: &str = include_str!("ariphmetic.txt");

fn pratt_parser(i: &mut Stream<'_>) -> PResult<i64> {
    use winnow::combinator::precedence;
    // precedence::precedence(
    //     dispatch! {peek(any);
    //         b'(' => delimited(b'(',  pratt_parser, b')'),
    //         _ => digit1.parse_to::<i64>()
    //     },
    //     dispatch! {any;
    //         b'+' => empty.value((9, (&|a| a) as _)),
    //         b'-' => empty.value((9, (&|a: i64| -a) as _)),
    //         _ => fail
    //     },
    //     fail,
    //     dispatch! {any;
    //        b'+' => empty.value((5, 6, (&|a, b|  a + b) as _)),
    //        b'-' => empty.value((5, 6, (&|a, b|  a - b) as _)),
    //        b'*' => empty.value((7, 8, (&|a, b| a * b) as _)),
    //        b'/' => empty.value((7, 8, (&|a, b|  a / b) as _)),
    //        b'%' => empty.value((7, 8, (&|a, b|  a % b) as _)),
    //        b'^' => empty.value((9, 10, (&|a, b|  a ^ b) as _)),
    //        _ => fail
    //     },
    // )
    // .parse_next(i)
    Ok(0)
}

fn shunting_yard_parser(i: &mut Stream<'_>) -> PResult<i64> {
    use winnow::combinator::shunting_yard;
    // shunting_yard::precedence(
    //     dispatch! {peek(any);
    //         b'(' => delimited(b'(',  shunting_yard_parser, b')'),
    //         _ => digit1.parse_to::<i64>()
    //     },
    //     dispatch! {any;
    //         b'+' => empty.value((9, (&|a| a) as _)),
    //         b'-' => empty.value((9, (&|a: i64| -a) as _)),
    //         _ => fail
    //     },
    //     fail,
    //     dispatch! {any;
    //        b'+' => empty.value((5, 6, (&|a, b|  a + b) as _)),
    //        b'-' => empty.value((5, 6, (&|a, b|  a - b) as _)),
    //        b'*' => empty.value((7, 8, (&|a, b| a * b) as _)),
    //        b'/' => empty.value((7, 8, (&|a, b|  a / b) as _)),
    //        b'%' => empty.value((7, 8, (&|a, b|  a % b) as _)),
    //        b'^' => empty.value((9, 10, (&|a, b|  a ^ b) as _)),
    //        _ => fail
    //     },
    // )
    // .parse_next(i)
    Ok(0)
}

fn parse_expression(c: &mut Criterion) {
    // remove the last `\n`
    let input = &CORPUS.as_bytes()[0..CORPUS.as_bytes().len() - 1];
    let mut group = c.benchmark_group("pratt");

    pratt_parser.parse(input).expect("pratt should parse");
    shunting_yard_parser
        .parse(input)
        .expect("shunting yard should parse");

    group.bench_function("pratt", |b| {
        b.iter(|| black_box(pratt_parser.parse(input).unwrap()));
    });

    group.bench_function("shunting yard", |b| {
        b.iter(|| black_box(shunting_yard_parser.parse(input).unwrap()));
    });
}

// https://www.jibbow.com/posts/criterion-flamegraphs/
use pprof::criterion::{Output, PProfProfiler};
criterion_group! {
    name = benches;
    config = Criterion::default().with_profiler(PProfProfiler::new(100, Output::Flamegraph(None)));
    targets =parse_expression
}

criterion_main!(benches);
