#![allow(dead_code)]
// #![allow(unused_variables)]

use std::str;

use winnow::combinator::delimited;
use winnow::combinator::fold_repeat;
use winnow::prelude::*;
use winnow::token::take_till1;
use winnow::IResult;

fn atom(_tomb: &mut ()) -> impl for<'a> FnMut(&'a [u8]) -> IResult<&'a [u8], String> {
    move |input| {
        take_till1([' ', '\t', '\r', '\n'])
            .try_map(str::from_utf8)
            .map(ToString::to_string)
            .parse_next(input)
    }
}

// FIXME: should we support the use case of borrowing data mutably in a parser?
fn list<'a>(i: &'a [u8], tomb: &mut ()) -> IResult<&'a [u8], String> {
    delimited(
        '(',
        fold_repeat(
            0..,
            atom(tomb),
            String::new,
            |mut acc: String, next: String| {
                acc.push_str(next.as_str());
                acc
            },
        ),
        ')',
    )
    .parse_next(i)
}
