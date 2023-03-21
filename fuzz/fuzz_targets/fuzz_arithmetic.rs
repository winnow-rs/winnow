#![no_main]
use libfuzzer_sys::fuzz_target;
use std::str;

use winnow::prelude::*;
use winnow::{
    branch::alt,
    bytes::tag,
    character::{digit1 as digit, space0 as space},
    multi::fold_many0,
    sequence::{delimited, terminated},
};

use std::cell::RefCell;

thread_local! {
    pub static LEVEL: RefCell<u32> = RefCell::new(0);
}

fn reset() {
    LEVEL.with(|l| {
        *l.borrow_mut() = 0;
    });
}

fn incr(i: &str) -> IResult<&str, ()> {
    LEVEL.with(|l| {
        *l.borrow_mut() += 1;

        // limit the number of recursions, the fuzzer keeps running into them
        if *l.borrow() >= 8192 {
            Err(winnow::error::ErrMode::Cut(winnow::error::Error::new(
                i,
                winnow::error::ErrorKind::Many,
            )))
        } else {
            Ok((i, ()))
        }
    })
}

fn decr() {
    LEVEL.with(|l| {
        *l.borrow_mut() -= 1;
    });
}

fn parens(i: &str) -> IResult<&str, i64> {
    delimited(
        space,
        delimited(terminated("(", incr), expr, tag(")").map(|_| decr())),
        space,
    )(i)
}

fn factor(i: &str) -> IResult<&str, i64> {
    alt((delimited(space, digit, space).parse_to(), parens))(i)
}

fn term(i: &str) -> IResult<&str, i64> {
    incr(i)?;
    let (i, init) = factor(i).map_err(|e| {
        decr();
        e
    })?;

    let res = fold_many0(
        alt((('*', factor), ('/', factor.verify(|i| *i != 0)))),
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
    )(i);

    decr();
    res
}

fn expr(i: &str) -> IResult<&str, i64> {
    incr(i)?;
    let (i, init) = term(i).map_err(|e| {
        decr();
        e
    })?;

    let res = fold_many0(
        (alt(('+', '-')), term),
        || init,
        |acc, (op, val): (char, i64)| {
            if op == '+' {
                acc.saturating_add(val)
            } else {
                acc.saturating_sub(val)
            }
        },
    )(i);

    decr();
    res
}

fuzz_target!(|data: &[u8]| {
    reset();
    // fuzzed code goes here
    let _ = match str::from_utf8(data) {
        Ok(v) => {
            //println!("v: {}", v);
            factor(v)
        }
        Err(_) => factor("2"),
    };
});
