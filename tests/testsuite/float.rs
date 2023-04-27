use winnow::branch::alt;
use winnow::character::digit1 as digit;
use winnow::combinator::delimited;
use winnow::combinator::opt;
use winnow::prelude::*;
use winnow::IResult;

use std::str;
use std::str::FromStr;

fn unsigned_float(i: &[u8]) -> IResult<&[u8], f32> {
    let float_bytes = alt((
        delimited(digit, ".", opt(digit)),
        delimited(opt(digit), ".", digit),
    ))
    .recognize();
    let float_str = float_bytes.try_map(str::from_utf8);
    float_str.try_map(FromStr::from_str).parse_next(i)
}

fn float(i: &[u8]) -> IResult<&[u8], f32> {
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
    assert_eq!(unsigned_float(&b"123.456;"[..]), Ok((&b";"[..], 123.456)));
    assert_eq!(unsigned_float(&b"0.123;"[..]), Ok((&b";"[..], 0.123)));
    assert_eq!(unsigned_float(&b"123.0;"[..]), Ok((&b";"[..], 123.0)));
    assert_eq!(unsigned_float(&b"123.;"[..]), Ok((&b";"[..], 123.0)));
    assert_eq!(unsigned_float(&b".123;"[..]), Ok((&b";"[..], 0.123)));
}

#[test]
fn float_test() {
    assert_eq!(float(&b"123.456;"[..]), Ok((&b";"[..], 123.456)));
    assert_eq!(float(&b"+123.456;"[..]), Ok((&b";"[..], 123.456)));
    assert_eq!(float(&b"-123.456;"[..]), Ok((&b";"[..], -123.456)));
}
