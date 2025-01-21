#![allow(dead_code)]
// #![allow(unused_variables)]

use std::str;

use winnow::combinator::delimited;
use winnow::combinator::repeat;
use winnow::error::InputError;
use winnow::prelude::*;
use winnow::token::take_till;

use crate::TestResult;

fn atom<'a>(_tomb: &mut ()) -> impl Parser<&'a [u8], String, InputError<&'a [u8]>> {
    take_till(1.., [' ', '\t', '\r', '\n'])
        .try_map(str::from_utf8)
        .map(ToString::to_string)
}

// FIXME: should we support the use case of borrowing data mutably in a parser?
fn list<'a>(i: &mut &'a [u8], tomb: &mut ()) -> TestResult<&'a [u8], String> {
    delimited(
        '(',
        repeat(0.., atom(tomb)).fold(String::new, |mut acc: String, next: String| {
            acc.push_str(next.as_str());
            acc
        }),
        ')',
    )
    .parse_next(i)
}
