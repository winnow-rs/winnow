use winnow::branch::alt;
use winnow::bytes::take_till1;
use winnow::bytes::take_while1;
use winnow::multi::many0;
use winnow::prelude::*;

fn contains_token(c: &mut criterion::Criterion) {
    let data = [
        ("contiguous", CONTIGUOUS),
        ("interleaved", INTERLEAVED),
        ("canada", CANADA),
    ];
    let mut group = c.benchmark_group("contains_token");
    for (name, sample) in data {
        let len = sample.len();
        group.throughput(criterion::Throughput::Bytes(len as u64));

        group.bench_with_input(criterion::BenchmarkId::new("str", name), &len, |b, _| {
            fn parser(input: &str) -> IResult<&str, usize> {
                let contains = "0123456789";
                many0(alt((take_while1(contains), take_till1(contains)))).parse_next(input)
            }

            b.iter(|| parser.parse_next(sample).unwrap());
        });
        group.bench_with_input(criterion::BenchmarkId::new("slice", name), &len, |b, _| {
            fn parser(input: &str) -> IResult<&str, usize> {
                let contains = &['0', '1', '2', '3', '4', '5', '6', '7', '8', '9'][..];
                many0(alt((take_while1(contains), take_till1(contains)))).parse_next(input)
            }

            b.iter(|| parser.parse_next(sample).unwrap());
        });
        group.bench_with_input(criterion::BenchmarkId::new("tuple", name), &len, |b, _| {
            fn parser(input: &str) -> IResult<&str, usize> {
                let contains = ('0', '1', '2', '3', '4', '5', '6', '7', '8', '9');
                many0(alt((take_while1(contains), take_till1(contains)))).parse_next(input)
            }

            b.iter(|| parser.parse_next(sample).unwrap());
        });
        group.bench_with_input(
            criterion::BenchmarkId::new("closure-or", name),
            &len,
            |b, _| {
                fn parser(input: &str) -> IResult<&str, usize> {
                    let contains = |c: char| {
                        c == '0'
                            || c == '1'
                            || c == '2'
                            || c == '3'
                            || c == '4'
                            || c == '5'
                            || c == '6'
                            || c == '7'
                            || c == '8'
                            || c == '9'
                    };
                    many0(alt((take_while1(contains), take_till1(contains)))).parse_next(input)
                }

                b.iter(|| parser.parse_next(sample).unwrap());
            },
        );
        group.bench_with_input(
            criterion::BenchmarkId::new("closure-matches", name),
            &len,
            |b, _| {
                fn parser(input: &str) -> IResult<&str, usize> {
                    let contains = |c: char| {
                        matches!(c, '0' | '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9')
                    };
                    many0(alt((take_while1(contains), take_till1(contains)))).parse_next(input)
                }

                b.iter(|| parser.parse_next(sample).unwrap());
            },
        );
    }
    group.finish();
}

const CONTIGUOUS: &str = "012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789";
const INTERLEAVED: &str = "0123456789abc0123456789ab0123456789ab0123456789ab0123456789ab0123456789ab0123456789ab0123456789ab0123456789ab0123456789ab0123456789ab0123456789ab0123456789ab0123456789ab0123456789ab0123456789ab0123456789ab0123456789ab0123456789ab0123456789ab0123456789ab0123456789ab0123456789ab";
const CANADA: &str = include_str!("../third_party/nativejson-benchmark/data/canada.json");

criterion::criterion_group!(benches, contains_token);
criterion::criterion_main!(benches);
