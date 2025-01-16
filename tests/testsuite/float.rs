use std::str;
use std::str::FromStr;

use snapbox::str;

use winnow::ascii::digit1 as digit;
use winnow::combinator::alt;
use winnow::combinator::delimited;
use winnow::combinator::opt;
use winnow::error::InputError;
use winnow::prelude::*;

fn unsigned_float<'i>(i: &mut &'i [u8]) -> PResult<f32, InputError<&'i [u8]>> {
    let float_bytes = alt((
        delimited(digit, ".", opt(digit)),
        delimited(opt(digit), ".", digit),
    ))
    .take();
    let float_str = float_bytes.try_map(str::from_utf8);
    float_str.try_map(FromStr::from_str).parse_next(i)
}

fn float<'i>(i: &mut &'i [u8]) -> PResult<f32, InputError<&'i [u8]>> {
    (opt(alt(("+", "-"))), unsigned_float)
        .map(|(sign, value)| {
            sign.and_then(|s| if s[0] == b'-' { Some(-1f32) } else { None })
                .unwrap_or(1f32)
                * value
        })
        .parse_next(i)
}

#[test]
fn unsigned_float_test() {
    assert_parse!(unsigned_float.parse_peek(&b"123.456;"[..]), str![[r#"
Ok(
    (
        [
            59,
        ],
        123.456,
    ),
)

"#]]);
    assert_parse!(unsigned_float.parse_peek(&b"0.123;"[..]), str![[r#"
Ok(
    (
        [
            59,
        ],
        0.123,
    ),
)

"#]]);
    assert_parse!(unsigned_float.parse_peek(&b"123.0;"[..]), str![[r#"
Ok(
    (
        [
            59,
        ],
        123.0,
    ),
)

"#]]);
    assert_parse!(unsigned_float.parse_peek(&b"123.;"[..]), str![[r#"
Ok(
    (
        [
            59,
        ],
        123.0,
    ),
)

"#]]);
    assert_parse!(unsigned_float.parse_peek(&b".123;"[..]), str![[r#"
Ok(
    (
        [
            59,
        ],
        0.123,
    ),
)

"#]]);
}

#[test]
fn float_test() {
    assert_parse!(float.parse_peek(&b"123.456;"[..]), str![[r#"
Ok(
    (
        [
            59,
        ],
        123.456,
    ),
)

"#]]);
    assert_parse!(float.parse_peek(&b"+123.456;"[..]), str![[r#"
Ok(
    (
        [
            59,
        ],
        123.456,
    ),
)

"#]]);
    assert_parse!(float.parse_peek(&b"-123.456;"[..]), str![[r#"
Ok(
    (
        [
            59,
        ],
        -123.456,
    ),
)

"#]]);
}
