mod parser;

use std::cell::RefCell;

use criterion::{black_box, BatchSize};
use winnow::{prelude::*, Stateful};

fn pratt(c: &mut criterion::Criterion) {
    let input =
        "a = 2*-2 * (a ? 1 + 2 * 4 - --a.bar + 2 : 2) / ( &**foo.a->p! -+1) + 3^1 / 4 == 1 * (2 - 7 + 567 *12 /2) + 3*(1+2*( 45 /2))";
    let mut group = c.benchmark_group("pratt");

    {
        let bump = RefCell::new(bumpalo::Bump::new());

        {
            let i = Stateful {
                input,
                state: &*bump.borrow(),
            };
            parser::pratt_parser.parse(i).expect("pratt should parse");
        }
        bump.borrow_mut().reset();
        {
            let i = Stateful {
                input,
                state: &*bump.borrow(),
            };
            parser::shunting_yard_parser
                .parse(i)
                .expect("shunting yard should parse");
        }
        bump.borrow_mut().reset();

        {
            group.bench_function("pratt", |b| {
                b.iter_batched(
                    || {
                            bump.borrow_mut().reset();
                        &bump
                    },
                    |b| {
                        let i = Stateful {
                            input,
                            state: &*b.borrow(),
                        };
                        black_box(parser::pratt_parser.parse(i).unwrap());
                    },
                    BatchSize::SmallInput,
                );
            });
        }
        {
            group.bench_function("shunting_yard", |b| {
                b.iter_batched(
                    || {
                            bump.borrow_mut().reset();
                        &bump
                    },
                    |b| {
                        let i = Stateful {
                            input,
                            state: &*b.borrow(),
                        };
                        black_box(parser::shunting_yard_parser.parse(i).unwrap());
                    },
                    BatchSize::SmallInput,
                );
            });
        }
    }

    // group.bench_function("pratt_with_new_bump_each_time", |b| {
    //     b.iter_batched(
    //         || bumpalo::Bump::new(),
    //         |b| {
    //             let i = Stateful { input, state: &b };
    //             black_box(parser::pratt_parser.parse(i).unwrap());
    //         },
    //         BatchSize::SmallInput,
    //     );
    // });
    //
    // group.bench_function("shunting_yard_with_new_bump_each_time", |b| {
    //     b.iter_batched(
    //         || bumpalo::Bump::new(),
    //         |b| {
    //             let i = Stateful { input, state: &b };
    //             black_box(parser::shunting_yard_parser.parse(i).unwrap());
    //         },
    //         BatchSize::SmallInput,
    //     );
    // });
}

criterion::criterion_group!(benches, pratt);
criterion::criterion_main!(benches);
