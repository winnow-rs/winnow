#![cfg(feature = "alloc")]
#![allow(dead_code)]
#![allow(clippy::redundant_closure)]

use winnow::prelude::*;
use winnow::Partial;
use winnow::{error::ErrMode, error::ErrorKind, error::InputError, error::Needed, unpeek, IResult};

#[allow(dead_code)]
struct Range {
    start: char,
    end: char,
}

pub fn take_char(input: &[u8]) -> IResult<&[u8], char> {
    if !input.is_empty() {
        Ok((&input[1..], input[0] as char))
    } else {
        Err(ErrMode::Incomplete(Needed::new(1)))
    }
}

#[cfg(feature = "std")]
mod parse_int {
    use std::str;
    use winnow::prelude::*;
    use winnow::Partial;
    use winnow::{
        ascii::{digit1 as digit, space1 as space},
        combinator::opt,
        combinator::repeat,
        unpeek, IResult,
    };

    fn parse_ints(input: Partial<&[u8]>) -> IResult<Partial<&[u8]>, Vec<i32>> {
        repeat(0.., unpeek(spaces_or_int)).parse_peek(input)
    }

    fn spaces_or_int(input: Partial<&[u8]>) -> IResult<Partial<&[u8]>, i32> {
        let (i, _) = opt(space.complete_err()).parse_peek(input)?;
        let (i, res) = digit
            .complete_err()
            .map(|x| {
                println!("x: {:?}", x);
                let result = str::from_utf8(x).unwrap();
                println!("Result: {}", result);
                println!("int is empty?: {}", x.is_empty());
                match result.parse() {
                    Ok(i) => i,
                    Err(e) => panic!("UH OH! NOT A DIGIT! {:?}", e),
                }
            })
            .parse_peek(i)?;

        Ok((i, res))
    }

    #[test]
    fn issue_142() {
        let subject = parse_ints(Partial::new(&b"12 34 5689a"[..]));
        let expected = Ok((Partial::new(&b"a"[..]), vec![12, 34, 5689]));
        assert_eq!(subject, expected);

        let subject = parse_ints(Partial::new(&b"12 34 5689 "[..]));
        let expected = Ok((Partial::new(&b" "[..]), vec![12, 34, 5689]));
        assert_eq!(subject, expected);
    }
}

#[test]
fn usize_length_bytes_issue() {
    use winnow::binary::be_u16;
    use winnow::binary::length_data;
    #[allow(clippy::type_complexity)]
    let _: IResult<Partial<&[u8]>, &[u8]> = length_data(be_u16).parse_peek(Partial::new(b"012346"));
}

#[test]
fn take_till0_issue() {
    use winnow::token::take_till0;

    fn nothing(i: Partial<&[u8]>) -> IResult<Partial<&[u8]>, &[u8]> {
        take_till0(|_| true).parse_peek(i)
    }

    assert_eq!(
        nothing(Partial::new(b"")),
        Err(ErrMode::Incomplete(Needed::new(1)))
    );
    assert_eq!(
        nothing(Partial::new(b"abc")),
        Ok((Partial::new(&b"abc"[..]), &b""[..]))
    );
}

#[test]
fn issue_655() {
    use winnow::ascii::{line_ending, not_line_ending};
    fn twolines(i: Partial<&str>) -> IResult<Partial<&str>, (&str, &str)> {
        let (i, l1) = not_line_ending.parse_peek(i)?;
        let (i, _) = line_ending.parse_peek(i)?;
        let (i, l2) = not_line_ending.parse_peek(i)?;
        let (i, _) = line_ending.parse_peek(i)?;

        Ok((i, (l1, l2)))
    }

    assert_eq!(
        twolines(Partial::new("foo\nbar\n")),
        Ok((Partial::new(""), ("foo", "bar")))
    );
    assert_eq!(
        twolines(Partial::new("féo\nbar\n")),
        Ok((Partial::new(""), ("féo", "bar")))
    );
    assert_eq!(
        twolines(Partial::new("foé\nbar\n")),
        Ok((Partial::new(""), ("foé", "bar")))
    );
    assert_eq!(
        twolines(Partial::new("foé\r\nbar\n")),
        Ok((Partial::new(""), ("foé", "bar")))
    );
}

#[cfg(feature = "alloc")]
fn issue_717(i: &[u8]) -> IResult<&[u8], Vec<&[u8]>> {
    use winnow::combinator::separated0;
    use winnow::token::{tag, take_till1};

    separated0(take_till1([0x0u8]), tag([0x0])).parse_peek(i)
}

mod issue_647 {
    use super::*;
    use winnow::combinator::separated0;
    use winnow::token::tag;
    use winnow::{binary::be_f64, error::ErrMode, IResult};
    pub type Stream<'a> = winnow::Partial<&'a [u8]>;

    #[derive(PartialEq, Debug, Clone)]
    struct Data {
        c: f64,
        v: Vec<f64>,
    }

    #[allow(clippy::type_complexity)]
    fn list<'a>(
        input: Stream<'a>,
        _cs: &f64,
    ) -> Result<(Stream<'a>, Vec<f64>), ErrMode<InputError<Stream<'a>>>> {
        separated0(be_f64.complete_err(), tag(",").complete_err()).parse_peek(input)
    }

    fn data(input: Stream<'_>) -> IResult<Stream<'_>, Data> {
        let (i, c) = be_f64.parse_peek(input)?;
        let (i, _) = "\n".parse_peek(i)?;
        let (i, v) = list(i, &c)?;
        Ok((i, Data { c, v }))
    }
}

#[test]
#[cfg_attr(debug_assertions, should_panic)]
fn issue_848_overflow_incomplete_bits_to_bytes() {
    fn take(i: Partial<&[u8]>) -> IResult<Partial<&[u8]>, &[u8]> {
        use winnow::token::take;
        take(0x2000000000000000_usize).parse_peek(i)
    }
    fn parser(i: Partial<&[u8]>) -> IResult<Partial<&[u8]>, &[u8]> {
        use winnow::binary::bits::{bits, bytes};

        bits(bytes(unpeek(take))).parse_peek(i)
    }
    assert_eq!(
        parser(Partial::new(&b""[..])),
        Err(ErrMode::Cut(InputError::new(
            Partial::new(&b""[..]),
            ErrorKind::Assert
        )))
    );
}

#[test]
fn issue_942() {
    use winnow::error::{AddContext, ParserError};
    pub fn parser<'a, E: ParserError<&'a str> + AddContext<&'a str, &'static str>>(
        i: &'a str,
    ) -> IResult<&'a str, usize, E> {
        use winnow::combinator::repeat;
        repeat(0.., 'a'.context("char_a")).parse_peek(i)
    }
    assert_eq!(parser::<()>("aaa"), Ok(("", 3)));
}

#[test]
#[cfg(feature = "std")]
fn issue_many_m_n_with_zeros() {
    use winnow::combinator::repeat;
    let mut parser = repeat::<_, _, Vec<_>, (), _>(0, 'a');
    assert_eq!(parser.parse_peek("aaa"), Ok(("aaa", vec![])));
}

#[test]
fn issue_1027_convert_error_panic_nonempty() {
    use winnow::error::{convert_error, VerboseError};

    let input = "a";

    let result: IResult<_, _, VerboseError<&str>> = ('a', 'b').parse_peek(input);
    let err = match result.unwrap_err() {
        ErrMode::Backtrack(e) => e,
        _ => unreachable!(),
    };

    let msg = convert_error(input, err);
    assert_eq!(msg, "0: at line 1, in Token:\na\n ^\n\n",);
}

#[test]
fn issue_1231_bits_expect_fn_closure() {
    use winnow::binary::bits::{bits, take};
    pub fn example(input: &[u8]) -> IResult<&[u8], (u8, u8)> {
        bits::<_, _, InputError<_>, _, _>((take(1usize), take(1usize))).parse_peek(input)
    }
    assert_eq!(example(&[0xff]), Ok((&b""[..], (1, 1))));
}

#[test]
fn issue_1282_findtoken_char() {
    use winnow::token::one_of;
    let mut parser = one_of::<_, _, InputError<_>>(&['a', 'b', 'c'][..]);
    assert_eq!(parser.parse_peek("aaa"), Ok(("aa", 'a')));
}

#[test]
fn issue_x_looser_fill_bounds() {
    use winnow::{ascii::digit1, combinator::fill, combinator::terminated};

    fn fill_pair(i: &[u8]) -> IResult<&[u8], [&[u8]; 2]> {
        let mut buf = [&[][..], &[][..]];
        let (i, _) = fill(terminated(digit1, ","), &mut buf).parse_peek(i)?;
        Ok((i, buf))
    }

    assert_eq!(
        fill_pair(b"123,456,"),
        Ok((&b""[..], [&b"123"[..], &b"456"[..]]))
    );
    assert_eq!(
        fill_pair(b"123,456,789"),
        Ok((&b"789"[..], [&b"123"[..], &b"456"[..]]))
    );
    assert_eq!(
        fill_pair(b"123,,"),
        Err(ErrMode::Backtrack(InputError::new(
            &b","[..],
            ErrorKind::Slice
        )))
    );
}

#[cfg(feature = "std")]
fn issue_1459_clamp_capacity() {
    // shouldn't panic
    use winnow::combinator::repeat;
    let mut parser = repeat::<_, _, Vec<_>, (), _>(usize::MAX..=usize::MAX, 'a');
    assert_eq!(
        parser.parse_peek("a"),
        Err(winnow::error::ErrMode::Backtrack(()))
    );

    // shouldn't panic
    let mut parser = repeat::<_, _, Vec<_>, (), _>(usize::MAX, 'a');
    assert_eq!(
        parser.parse_peek("a"),
        Err(winnow::error::ErrMode::Backtrack(()))
    );
}

#[test]
fn issue_1617_count_parser_returning_zero_size() {
    use winnow::{combinator::repeat, token::tag};

    // previously, `repeat()` panicked if the parser had type `O = ()`
    let parser = tag::<_, _, InputError<&str>>("abc").map(|_| ());
    // shouldn't panic
    let result = repeat(3, parser)
        .parse_peek("abcabcabcdef")
        .expect("parsing should succeed");
    assert_eq!(result, ("def", vec![(), (), ()]));
}
