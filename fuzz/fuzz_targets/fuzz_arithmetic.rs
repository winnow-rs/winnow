#![no_main]
use libfuzzer_sys::fuzz_target;
use std::str;
use winnow::error::ParserError;

use winnow::prelude::*;
use winnow::{
    ascii::{digit1 as digit, space0 as space},
    combinator::alt,
    combinator::repeat,
    combinator::{delimited, terminated},
};

use std::cell::RefCell;

thread_local! {
    pub static LEVEL: RefCell<u32> = const { RefCell::new(0) };
}

fn reset() {
    LEVEL.with(|l| {
        *l.borrow_mut() = 0;
    });
}

fn incr(i: &mut &str) -> PResult<()> {
    LEVEL.with(|l| {
        *l.borrow_mut() += 1;

        // limit the number of recursions, the fuzzer keeps running into them
        if *l.borrow() >= 8192 {
            Err(winnow::error::ErrMode::from_error_kind(
                i,
                winnow::error::ErrorKind::Repeat,
            ))
        } else {
            Ok(())
        }
    })
}

fn decr() {
    LEVEL.with(|l| {
        *l.borrow_mut() -= 1;
    });
}

fn parens(i: &mut &str) -> PResult<i64> {
    delimited(
        space,
        delimited(terminated("(", incr), expr, ")".map(|_| decr())),
        space,
    )
    .parse_next(i)
}

fn factor(i: &mut &str) -> PResult<i64> {
    alt((delimited(space, digit, space).parse_to(), parens)).parse_next(i)
}

fn term(i: &mut &str) -> PResult<i64> {
    incr(i)?;
    let init = factor(i).inspect_err(|_e| {
        decr();
    })?;

    let res = repeat(0.., alt((('*', factor), ('/', factor.verify(|i| *i != 0)))))
        .fold(
            || init,
            |acc, (op, val): (char, i64)| {
                if op == '*' {
                    acc.saturating_mul(val)
                } else {
                    match acc.checked_div(val) {
                        Some(v) => v,
                        // we get a division with overflow because we can get acc = i64::MIN and val = -1
                        // the division by zero is already checked earlier by verify
                        None => i64::MAX,
                    }
                }
            },
        )
        .parse_next(i);

    decr();
    res
}

fn expr(i: &mut &str) -> PResult<i64> {
    incr(i)?;
    let init = term(i).inspect_err(|_e| {
        decr();
    })?;

    let res = repeat(0.., (alt(('+', '-')), term))
        .fold(
            || init,
            |acc, (op, val): (char, i64)| {
                if op == '+' {
                    acc.saturating_add(val)
                } else {
                    acc.saturating_sub(val)
                }
            },
        )
        .parse_next(i);

    decr();
    res
}

fuzz_target!(|data: &[u8]| {
    reset();
    // fuzzed code goes here
    let _ = match str::from_utf8(data) {
        Ok(mut v) => {
            //println!("v: {}", v);
            factor(&mut v)
        }
        Err(_) => factor(&mut "2"),
    };
});
