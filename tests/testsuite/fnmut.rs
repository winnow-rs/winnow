#![cfg(feature = "alloc")]

use winnow::combinator::repeat;
use winnow::unpeek;
use winnow::Parser;

#[test]
#[cfg(feature = "std")]
fn parse() {
    let mut counter = 0;

    let res = {
        let mut parser = repeat::<_, _, Vec<_>, (), _>(
            0..,
            unpeek(|i| {
                counter += 1;
                "abc".parse_peek(i)
            }),
        );

        parser.parse_peek("abcabcabcabc").unwrap()
    };

    println!("res: {:?}", res);
    assert_eq!(counter, 5);
}

#[test]
fn accumulate() {
    let mut v = Vec::new();

    let (_, count) = {
        let mut parser = repeat::<_, _, usize, (), _>(
            0..,
            unpeek(|i| {
                let (i, o) = "abc".parse_peek(i)?;
                v.push(o);
                Ok((i, ()))
            }),
        );
        parser.parse_peek("abcabcabcabc").unwrap()
    };

    println!("v: {:?}", v);
    assert_eq!(count, 4);
    assert_eq!(v.len(), 4);
}
