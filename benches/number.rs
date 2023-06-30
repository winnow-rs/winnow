#[macro_use]
extern crate criterion;

use criterion::Criterion;

use winnow::ascii::float;
use winnow::binary::be_u64;
use winnow::error::ErrMode;
use winnow::error::Error;
use winnow::error::ErrorKind;
use winnow::prelude::*;
use winnow::stream::ParseSlice;

type Stream<'i> = &'i [u8];

fn parser(i: Stream<'_>) -> IResult<Stream<'_>, u64> {
    be_u64.parse_peek(i)
}

fn number(c: &mut Criterion) {
    let data = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12];

    parser(&data[..]).expect("should parse correctly");
    c.bench_function("number", move |b| {
        b.iter(|| parser(&data[..]).unwrap());
    });
}

fn float_bytes(c: &mut Criterion) {
    println!(
        "float_bytes result: {:?}",
        float::<_, f64, Error<_>>.parse_peek(&b"-1.234E-12"[..])
    );
    c.bench_function("float bytes", |b| {
        b.iter(|| float::<_, f64, Error<_>>.parse_peek(&b"-1.234E-12"[..]));
    });
}

fn float_str(c: &mut Criterion) {
    println!(
        "float_str result: {:?}",
        float::<_, f64, Error<_>>.parse_peek("-1.234E-12")
    );
    c.bench_function("float str", |b| {
        b.iter(|| float::<_, f64, Error<_>>.parse_peek("-1.234E-12"));
    });
}

fn std_float(input: &[u8]) -> IResult<&[u8], f64, Error<&[u8]>> {
    match input.parse_slice() {
        Some(n) => Ok((&[], n)),
        None => Err(ErrMode::Backtrack(Error {
            input,
            kind: ErrorKind::Slice,
        })),
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

criterion_group!(benches, number, float_bytes, std_float_bytes, float_str);
criterion_main!(benches);
