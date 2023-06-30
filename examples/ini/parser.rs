use std::collections::HashMap;
use std::str;

use winnow::prelude::*;
use winnow::{
    ascii::{alphanumeric1 as alphanumeric, multispace0 as multispace, space0 as space},
    combinator::opt,
    combinator::repeat,
    combinator::{delimited, separated_pair, terminated},
    token::take_while,
    unpeek,
};

pub type Stream<'i> = &'i [u8];

pub fn categories(i: Stream<'_>) -> IResult<Stream<'_>, HashMap<&str, HashMap<&str, &str>>> {
    repeat(
        0..,
        separated_pair(
            unpeek(category),
            opt(multispace),
            repeat(0.., terminated(unpeek(key_value), opt(multispace))),
        ),
    )
    .parse_peek(i)
}

fn category(i: Stream<'_>) -> IResult<Stream<'_>, &str> {
    delimited('[', take_while(0.., |c| c != b']'), ']')
        .try_map(str::from_utf8)
        .parse_peek(i)
}

pub fn key_value(i: Stream<'_>) -> IResult<Stream<'_>, (&str, &str)> {
    let (i, key) = alphanumeric.try_map(str::from_utf8).parse_peek(i)?;
    let (i, _) = (opt(space), '=', opt(space)).parse_peek(i)?;
    let (i, val) = take_while(0.., |c| c != b'\n' && c != b';')
        .try_map(str::from_utf8)
        .parse_peek(i)?;
    let (i, _) = opt((';', take_while(0.., |c| c != b'\n'))).parse_peek(i)?;
    Ok((i, (key, val)))
}

#[test]
fn parse_category_test() {
    let ini_file = &b"[category]

parameter=value
key = value2"[..];

    let ini_without_category = &b"\n\nparameter=value
key = value2"[..];

    let res = category(ini_file);
    println!("{:?}", res);
    match res {
        Ok((i, o)) => println!("i: {:?} | o: {:?}", str::from_utf8(i), o),
        _ => println!("error"),
    }

    assert_eq!(res, Ok((ini_without_category, "category")));
}

#[test]
fn parse_key_value_test() {
    let ini_file = &b"parameter=value
key = value2"[..];

    let ini_without_key_value = &b"\nkey = value2"[..];

    let res = key_value(ini_file);
    println!("{:?}", res);
    match res {
        Ok((i, (o1, o2))) => println!("i: {:?} | o: ({:?},{:?})", str::from_utf8(i), o1, o2),
        _ => println!("error"),
    }

    assert_eq!(res, Ok((ini_without_key_value, ("parameter", "value"))));
}

#[test]
fn parse_key_value_with_space_test() {
    let ini_file = &b"parameter = value
key = value2"[..];

    let ini_without_key_value = &b"\nkey = value2"[..];

    let res = key_value(ini_file);
    println!("{:?}", res);
    match res {
        Ok((i, (o1, o2))) => println!("i: {:?} | o: ({:?},{:?})", str::from_utf8(i), o1, o2),
        _ => println!("error"),
    }

    assert_eq!(res, Ok((ini_without_key_value, ("parameter", "value"))));
}

#[test]
fn parse_key_value_with_comment_test() {
    let ini_file = &b"parameter=value;abc
key = value2"[..];

    let ini_without_key_value = &b"\nkey = value2"[..];

    let res = key_value(ini_file);
    println!("{:?}", res);
    match res {
        Ok((i, (o1, o2))) => println!("i: {:?} | o: ({:?},{:?})", str::from_utf8(i), o1, o2),
        _ => println!("error"),
    }

    assert_eq!(res, Ok((ini_without_key_value, ("parameter", "value"))));
}

#[test]
fn parse_multiple_categories_test() {
    let ini_file = &b"[abcd]

parameter=value;abc

key = value2

[category]
parameter3=value3
key4 = value4
"[..];

    let ini_after_parser = &b""[..];

    let res = categories(ini_file);
    //println!("{:?}", res);
    match res {
        Ok((i, ref o)) => println!("i: {:?} | o: {:?}", str::from_utf8(i), o),
        _ => println!("error"),
    }

    let mut expected_1: HashMap<&str, &str> = HashMap::new();
    expected_1.insert("parameter", "value");
    expected_1.insert("key", "value2");
    let mut expected_2: HashMap<&str, &str> = HashMap::new();
    expected_2.insert("parameter3", "value3");
    expected_2.insert("key4", "value4");
    let mut expected_h: HashMap<&str, HashMap<&str, &str>> = HashMap::new();
    expected_h.insert("abcd", expected_1);
    expected_h.insert("category", expected_2);
    assert_eq!(res, Ok((ini_after_parser, expected_h)));
}
