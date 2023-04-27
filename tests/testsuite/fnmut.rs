#![cfg(feature = "alloc")]

use winnow::combinator::many0;
use winnow::Parser;

#[test]
#[cfg(feature = "std")]
fn parse() {
    let mut counter = 0;

    let res = {
        let mut parser = many0::<_, _, Vec<_>, (), _>(|i| {
            counter += 1;
            "abc".parse_next(i)
        });

        parser.parse_next("abcabcabcabc").unwrap()
    };

    println!("res: {:?}", res);
    assert_eq!(counter, 5);
}

#[test]
fn accumulate() {
    let mut v = Vec::new();

    let (_, count) = {
        let mut parser = many0::<_, _, usize, (), _>(|i| {
            let (i, o) = "abc".parse_next(i)?;
            v.push(o);
            Ok((i, ()))
        });
        parser.parse_next("abcabcabcabc").unwrap()
    };

    println!("v: {:?}", v);
    assert_eq!(count, 4);
    assert_eq!(v.len(), 4);
}
