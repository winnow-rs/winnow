use std::str::FromStr;

use winnow::prelude::*;
use winnow::{
    ascii::{digit1 as digits, space0 as spaces},
    combinator::alt,
    combinator::delimited,
    combinator::fold_repeat,
    token::one_of,
};

// Parser definition

pub fn expr(i: &mut &str) -> PResult<i64> {
    let init = term.parse_next(i)?;

    fold_repeat(
        0..,
        (one_of(['+', '-']), term),
        move || init,
        |acc, (op, val): (char, i64)| {
            if op == '+' {
                acc + val
            } else {
                acc - val
            }
        },
    )
    .parse_next(i)
}

// We read an initial factor and for each time we find
// a * or / operator followed by another factor, we do
// the math by folding everything
fn term(i: &mut &str) -> PResult<i64> {
    let init = factor.parse_next(i)?;

    fold_repeat(
        0..,
        (one_of(['*', '/']), factor),
        move || init,
        |acc, (op, val): (char, i64)| {
            if op == '*' {
                acc * val
            } else {
                acc / val
            }
        },
    )
    .parse_next(i)
}

// We transform an integer string into a i64, ignoring surrounding whitespaces
// We look for a digit suite, and try to convert it.
// If either str::from_utf8 or FromStr::from_str fail,
// we fallback to the parens parser defined above
fn factor(i: &mut &str) -> PResult<i64> {
    delimited(
        spaces,
        alt((
            digits.try_map(FromStr::from_str),
            delimited('(', expr, ')'),
            parens,
        )),
        spaces,
    )
    .parse_next(i)
}

// We parse any expr surrounded by parens, ignoring all whitespaces around those
fn parens(i: &mut &str) -> PResult<i64> {
    delimited('(', expr, ')').parse_next(i)
}

#[test]
fn factor_test() {
    assert_eq!(factor.parse_peek("3"), Ok(("", 3)));
    assert_eq!(factor.parse_peek(" 12"), Ok(("", 12)));
    assert_eq!(factor.parse_peek("537  "), Ok(("", 537)));
    assert_eq!(factor.parse_peek("  24   "), Ok(("", 24)));
}

#[test]
fn term_test() {
    assert_eq!(term.parse_peek(" 12 *2 /  3"), Ok(("", 8)));
    assert_eq!(term.parse_peek(" 2* 3  *2 *2 /  3"), Ok(("", 8)));
    assert_eq!(term.parse_peek(" 48 /  3/2"), Ok(("", 8)));
}

#[test]
fn expr_test() {
    assert_eq!(expr.parse_peek(" 1 +  2 "), Ok(("", 3)));
    assert_eq!(expr.parse_peek(" 12 + 6 - 4+  3"), Ok(("", 17)));
    assert_eq!(expr.parse_peek(" 1 + 2*3 + 4"), Ok(("", 11)));
}

#[test]
fn parens_test() {
    assert_eq!(expr.parse_peek(" (  2 )"), Ok(("", 2)));
    assert_eq!(expr.parse_peek(" 2* (  3 + 4 ) "), Ok(("", 14)));
    assert_eq!(expr.parse_peek("  2*2 / ( 5 - 1) + 3"), Ok(("", 4)));
}
