#![cfg(feature = "alloc")]

use winnow::{
    ascii::{alphanumeric1 as alphanumeric, line_ending as eol},
    combinator::repeat,
    combinator::terminated,
    prelude::*,
};

pub(crate) fn end_of_line<'i>(input: &mut &'i str) -> PResult<&'i str> {
    if input.is_empty() {
        Ok(*input)
    } else {
        eol.parse_next(input)
    }
}

pub(crate) fn read_line<'i>(input: &mut &'i str) -> PResult<&'i str> {
    terminated(alphanumeric, end_of_line).parse_next(input)
}

pub(crate) fn read_lines<'i>(input: &mut &'i str) -> PResult<Vec<&'i str>> {
    repeat(0.., read_line).parse_next(input)
}

#[cfg(feature = "alloc")]
#[test]
fn read_lines_test() {
    let res = Ok(("", vec!["Duck", "Dog", "Cow"]));

    assert_eq!(read_lines.parse_peek("Duck\nDog\nCow\n"), res);
    assert_eq!(read_lines.parse_peek("Duck\nDog\nCow"), res);
}
