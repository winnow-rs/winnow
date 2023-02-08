#![cfg(feature = "alloc")]
#![allow(dead_code)]
#![allow(clippy::redundant_closure)]

use winnow::prelude::*;
use winnow::Partial;
use winnow::{error::ErrMode, error::ErrorKind, error::Needed, IResult};

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
    use winnow::input::HexDisplay;
    use winnow::prelude::*;
    use winnow::Partial;
    use winnow::{
        character::{digit1 as digit, space1 as space},
        combinator::opt,
        multi::many0,
        IResult,
    };

    fn parse_ints(input: Partial<&[u8]>) -> IResult<Partial<&[u8]>, Vec<i32>> {
        many0(spaces_or_int)(input)
    }

    fn spaces_or_int(input: Partial<&[u8]>) -> IResult<Partial<&[u8]>, i32> {
        println!("{}", input.to_hex(8));
        let (i, _) = opt(space.complete())(input)?;
        let (i, res) = digit
            .complete()
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
            .parse_next(i)?;

        Ok((i, res))
    }

    #[test]
    fn issue_142() {
        let subject = parse_ints(Partial(&b"12 34 5689a"[..]));
        let expected = Ok((Partial(&b"a"[..]), vec![12, 34, 5689]));
        assert_eq!(subject, expected);

        let subject = parse_ints(Partial(&b"12 34 5689 "[..]));
        let expected = Ok((Partial(&b" "[..]), vec![12, 34, 5689]));
        assert_eq!(subject, expected);
    }
}

#[test]
fn usize_length_bytes_issue() {
    use winnow::multi::length_data;
    use winnow::number::be_u16;
    #[allow(clippy::type_complexity)]
    let _: IResult<Partial<&[u8]>, &[u8]> = length_data(be_u16)(Partial(b"012346"));
}

#[test]
fn take_till0_issue() {
    use winnow::bytes::take_till0;

    fn nothing(i: Partial<&[u8]>) -> IResult<Partial<&[u8]>, &[u8]> {
        take_till0(|_| true)(i)
    }

    assert_eq!(
        nothing(Partial(b"")),
        Err(ErrMode::Incomplete(Needed::new(1)))
    );
    assert_eq!(
        nothing(Partial(b"abc")),
        Ok((Partial(&b"abc"[..]), &b""[..]))
    );
}

#[test]
fn issue_655() {
    use winnow::character::{line_ending, not_line_ending};
    fn twolines(i: Partial<&str>) -> IResult<Partial<&str>, (&str, &str)> {
        let (i, l1) = not_line_ending(i)?;
        let (i, _) = line_ending(i)?;
        let (i, l2) = not_line_ending(i)?;
        let (i, _) = line_ending(i)?;

        Ok((i, (l1, l2)))
    }

    assert_eq!(
        twolines(Partial("foo\nbar\n")),
        Ok((Partial(""), ("foo", "bar")))
    );
    assert_eq!(
        twolines(Partial("féo\nbar\n")),
        Ok((Partial(""), ("féo", "bar")))
    );
    assert_eq!(
        twolines(Partial("foé\nbar\n")),
        Ok((Partial(""), ("foé", "bar")))
    );
    assert_eq!(
        twolines(Partial("foé\r\nbar\n")),
        Ok((Partial(""), ("foé", "bar")))
    );
}

#[cfg(feature = "alloc")]
fn issue_717(i: &[u8]) -> IResult<&[u8], Vec<&[u8]>> {
    use winnow::bytes::{tag, take_till1};
    use winnow::multi::separated0;

    separated0(take_till1([0x0u8]), tag([0x0]))(i)
}

mod issue_647 {
    use winnow::bytes::tag;
    use winnow::multi::separated0;
    use winnow::prelude::*;
    use winnow::{error::ErrMode, error::Error, number::be_f64, IResult};
    pub type Input<'a> = winnow::Partial<&'a [u8]>;

    #[derive(PartialEq, Debug, Clone)]
    struct Data {
        c: f64,
        v: Vec<f64>,
    }

    #[allow(clippy::type_complexity)]
    fn list<'a>(
        input: Input<'a>,
        _cs: &f64,
    ) -> Result<(Input<'a>, Vec<f64>), ErrMode<Error<Input<'a>>>> {
        separated0(be_f64.complete(), tag(",").complete())(input)
    }

    fn data(input: Input<'_>) -> IResult<Input<'_>, Data> {
        let (i, c) = be_f64(input)?;
        let (i, _) = tag("\n")(i)?;
        let (i, v) = list(i, &c)?;
        Ok((i, Data { c, v }))
    }
}

#[test]
fn issue_848_overflow_incomplete_bits_to_bytes() {
    fn take(i: Partial<&[u8]>) -> IResult<Partial<&[u8]>, &[u8]> {
        use winnow::bytes::take;
        take(0x2000000000000000_usize)(i)
    }
    fn parser(i: Partial<&[u8]>) -> IResult<Partial<&[u8]>, &[u8]> {
        use winnow::bits::{bits, bytes};

        bits(bytes(take))(i)
    }
    assert_eq!(
        parser(Partial(&b""[..])),
        Err(ErrMode::Cut(winnow::error::Error {
            input: Partial(&b""[..]),
            kind: ErrorKind::TooLarge
        }))
    );
}

#[test]
fn issue_942() {
    use winnow::error::{ContextError, ParseError};
    pub fn parser<'a, E: ParseError<&'a str> + ContextError<&'a str, &'static str>>(
        i: &'a str,
    ) -> IResult<&'a str, usize, E> {
        use winnow::{bytes::one_of, multi::many0};
        many0(one_of('a').context("char_a"))(i)
    }
    assert_eq!(parser::<()>("aaa"), Ok(("", 3)));
}

#[test]
#[cfg(feature = "std")]
fn issue_many_m_n_with_zeros() {
    use winnow::multi::many_m_n;
    let mut parser = many_m_n::<_, _, Vec<_>, (), _>(0, 0, 'a');
    assert_eq!(parser("aaa"), Ok(("aaa", vec![])));
}

#[test]
fn issue_1027_convert_error_panic_nonempty() {
    use winnow::error::{convert_error, VerboseError};

    let input = "a";

    let result: IResult<_, _, VerboseError<&str>> = ('a', 'b').parse_next(input);
    let err = match result.unwrap_err() {
        ErrMode::Backtrack(e) => e,
        _ => unreachable!(),
    };

    let msg = convert_error(input, err);
    assert_eq!(msg, "0: at line 1, in OneOf:\na\n ^\n\n",);
}

#[test]
fn issue_1231_bits_expect_fn_closure() {
    use winnow::bits::{bits, take};
    use winnow::error::Error;
    pub fn example(input: &[u8]) -> IResult<&[u8], (u8, u8)> {
        bits::<_, _, Error<_>, _, _>((take(1usize), take(1usize)))(input)
    }
    assert_eq!(example(&[0xff]), Ok((&b""[..], (1, 1))));
}

#[test]
fn issue_1282_findtoken_char() {
    use winnow::bytes::one_of;
    use winnow::error::Error;
    let parser = one_of::<_, _, Error<_>, false>(&['a', 'b', 'c'][..]);
    assert_eq!(parser("aaa"), Ok(("aa", 'a')));
}

#[test]
fn issue_x_looser_fill_bounds() {
    use winnow::{bytes::tag, character::digit1, multi::fill, sequence::terminated};

    fn fill_pair(i: &[u8]) -> IResult<&[u8], [&[u8]; 2]> {
        let mut buf = [&[][..], &[][..]];
        let (i, _) = fill(terminated(digit1, tag(",")), &mut buf)(i)?;
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
        Err(ErrMode::Backtrack(winnow::error::Error {
            input: &b","[..],
            kind: ErrorKind::Digit
        }))
    );
}

#[cfg(feature = "std")]
fn issue_1459_clamp_capacity() {
    // shouldn't panic
    use winnow::multi::many_m_n;
    let mut parser = many_m_n::<_, _, Vec<_>, (), _>(usize::MAX, usize::MAX, 'a');
    assert_eq!(parser("a"), Err(winnow::error::ErrMode::Backtrack(())));

    // shouldn't panic
    use winnow::multi::count;
    let mut parser = count::<_, _, Vec<_>, (), _>('a', usize::MAX);
    assert_eq!(parser("a"), Err(winnow::error::ErrMode::Backtrack(())));
}

#[test]
fn issue_1617_count_parser_returning_zero_size() {
    use winnow::{bytes::tag, error::Error, multi::count};

    // previously, `count()` panicked if the parser had type `O = ()`
    let parser = tag::<_, _, Error<&str>, false>("abc").map(|_| ());
    // shouldn't panic
    let result = count(parser, 3)("abcabcabcdef").expect("parsing should succeed");
    assert_eq!(result, ("def", vec![(), (), ()]));
}
