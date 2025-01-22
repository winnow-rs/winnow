#![cfg(feature = "alloc")]
#![allow(dead_code)]
#![allow(clippy::redundant_closure)]

use snapbox::prelude::*;
use snapbox::str;

use winnow::prelude::*;
use winnow::Partial;
use winnow::{error::ErrMode, error::InputError, error::Needed};

use crate::TestResult;

#[allow(dead_code)]
struct Range {
    start: char,
    end: char,
}

pub(crate) fn take_char(input: &mut &[u8]) -> ModalResult<char> {
    if !input.is_empty() {
        Ok(input.next_token().unwrap() as char)
    } else {
        Err(ErrMode::Incomplete(Needed::new(1)))
    }
}

#[cfg(feature = "std")]
mod parse_int {
    use crate::TestResult;
    use snapbox::prelude::*;
    use snapbox::str;
    use std::str;
    use winnow::prelude::*;
    use winnow::Partial;
    use winnow::{
        ascii::{digit1 as digit, space1 as space},
        combinator::opt,
        combinator::repeat,
    };

    fn parse_ints<'i>(input: &mut Partial<&'i [u8]>) -> TestResult<Partial<&'i [u8]>, Vec<i32>> {
        repeat(0.., spaces_or_int).parse_next(input)
    }

    fn spaces_or_int<'i>(input: &mut Partial<&'i [u8]>) -> TestResult<Partial<&'i [u8]>, i32> {
        let _ = opt(space.complete_err()).parse_next(input)?;
        let res = digit
            .complete_err()
            .map(|x| {
                println!("x: {x:?}");
                let result = str::from_utf8(x).unwrap();
                println!("Result: {result}");
                println!("int is empty?: {}", x.is_empty());
                match result.parse() {
                    Ok(i) => i,
                    Err(e) => panic!("UH OH! NOT A DIGIT! {e:?}"),
                }
            })
            .parse_next(input)?;

        Ok(res)
    }

    #[test]
    fn issue_142() {
        let subject = parse_ints.parse_peek(Partial::new(&b"12 34 5689a"[..]));
        assert_parse!(
            subject,
            str![[r#"
Ok(
    (
        Partial {
            input: [
                97,
            ],
            partial: true,
        },
        [
            12,
            34,
            5689,
        ],
    ),
)

"#]]
            .raw()
        );

        let subject = parse_ints.parse_peek(Partial::new(&b"12 34 5689 "[..]));
        assert_parse!(
            subject,
            str![[r#"
Ok(
    (
        Partial {
            input: [
                32,
            ],
            partial: true,
        },
        [
            12,
            34,
            5689,
        ],
    ),
)

"#]]
            .raw()
        );
    }
}

#[test]
fn usize_length_bytes_issue() {
    use winnow::binary::be_u16;
    use winnow::binary::length_take;
    #[allow(clippy::type_complexity)]
    let _: ModalResult<(Partial<&[u8]>, &[u8])> =
        length_take(be_u16).parse_peek(Partial::new(b"012346"));
}

#[test]
fn take_till0_issue() {
    use winnow::token::take_till;

    fn nothing<'i>(input: &mut Partial<&'i [u8]>) -> TestResult<Partial<&'i [u8]>, &'i [u8]> {
        take_till(0.., |_| true).parse_next(input)
    }

    assert_parse!(
        nothing.parse_peek(Partial::new(b"")),
        str![[r#"
Err(
    Incomplete(
        Size(
            1,
        ),
    ),
)

"#]]
        .raw()
    );
    assert_parse!(
        nothing.parse_peek(Partial::new(b"abc")),
        str![[r#"
Ok(
    (
        Partial {
            input: [
                97,
                98,
                99,
            ],
            partial: true,
        },
        [],
    ),
)

"#]]
        .raw()
    );
}

#[test]
fn issue_655() {
    use winnow::ascii::{line_ending, till_line_ending};
    fn twolines<'i>(
        input: &mut Partial<&'i str>,
    ) -> TestResult<Partial<&'i str>, (&'i str, &'i str)> {
        let l1 = till_line_ending.parse_next(input)?;
        let _ = line_ending.parse_next(input)?;
        let l2 = till_line_ending.parse_next(input)?;
        let _ = line_ending.parse_next(input)?;

        Ok((l1, l2))
    }

    assert_parse!(
        twolines.parse_peek(Partial::new("foo\nbar\n")),
        str![[r#"
Ok(
    (
        Partial {
            input: "",
            partial: true,
        },
        (
            "foo",
            "bar",
        ),
    ),
)

"#]]
        .raw()
    );
    assert_parse!(
        twolines.parse_peek(Partial::new("féo\nbar\n")),
        str![[r#"
Ok(
    (
        Partial {
            input: "",
            partial: true,
        },
        (
            "féo",
            "bar",
        ),
    ),
)

"#]]
        .raw()
    );
    assert_parse!(
        twolines.parse_peek(Partial::new("foé\nbar\n")),
        str![[r#"
Ok(
    (
        Partial {
            input: "",
            partial: true,
        },
        (
            "foé",
            "bar",
        ),
    ),
)

"#]]
        .raw()
    );
    assert_parse!(
        twolines.parse_peek(Partial::new("foé\r\nbar\n")),
        str![[r#"
Ok(
    (
        Partial {
            input: "",
            partial: true,
        },
        (
            "foé",
            "bar",
        ),
    ),
)

"#]]
        .raw()
    );
}

#[cfg(feature = "alloc")]
fn issue_717<'i>(input: &mut &'i [u8]) -> TestResult<&'i [u8], Vec<&'i [u8]>> {
    use winnow::combinator::separated;
    use winnow::token::{literal, take_till};

    separated(0.., take_till(1.., [0x0u8]), literal([0x0])).parse_next(input)
}

mod issue_647 {
    use super::*;
    use winnow::binary::be_f64;
    use winnow::combinator::separated;
    use winnow::token::literal;
    pub(crate) type Stream<'a> = Partial<&'a [u8]>;

    #[derive(PartialEq, Debug, Clone)]
    struct Data {
        c: f64,
        v: Vec<f64>,
    }

    #[allow(clippy::type_complexity)]
    fn list<'a>(input: &mut Stream<'a>, _cs: &f64) -> TestResult<Stream<'a>, Vec<f64>> {
        separated(0.., be_f64.complete_err(), literal(",").complete_err()).parse_next(input)
    }

    fn data<'i>(input: &mut Stream<'i>) -> TestResult<Stream<'i>, Data> {
        let c = be_f64.parse_next(input)?;
        let _ = "\n".parse_next(input)?;
        let v = list(input, &c)?;
        Ok(Data { c, v })
    }
}

#[test]
#[cfg_attr(debug_assertions, should_panic)]
fn issue_848_overflow_incomplete_bits_to_bytes() {
    fn take<'i>(input: &mut Partial<&'i [u8]>) -> TestResult<Partial<&'i [u8]>, &'i [u8]> {
        use winnow::token::take;
        take(0x2000000000000000_usize).parse_next(input)
    }
    fn parser<'i>(input: &mut Partial<&'i [u8]>) -> TestResult<Partial<&'i [u8]>, &'i [u8]> {
        use winnow::binary::bits::{bits, bytes};

        bits(bytes(take)).parse_next(input)
    }
    assert_parse!(parser.parse_peek(Partial::new(&b""[..])), str![]);
}

#[test]
fn issue_942() {
    pub(crate) fn parser<'i>(input: &mut &'i str) -> TestResult<&'i str, usize> {
        use winnow::combinator::repeat;
        repeat(1.., 'a'.context("char_a")).parse_next(input)
    }
    assert_parse!(
        parser.parse_peek("aaa"),
        str![[r#"
Ok(
    (
        "",
        3,
    ),
)

"#]]
        .raw()
    );
    assert_parse!(
        parser.parse_peek("bbb"),
        str![[r#"
Err(
    Backtrack(
        InputError {
            input: "bbb",
            kind: Literal,
        },
    ),
)

"#]]
        .raw()
    );
}

#[test]
#[cfg(feature = "std")]
fn issue_many_m_n_with_zeros() {
    use winnow::combinator::repeat;
    assert_parse!(
        repeat(0, 'a').map(|v: Vec<_>| v).parse_peek("aaa"),
        str![[r#"
Ok(
    (
        "aaa",
        [],
    ),
)

"#]]
        .raw()
    );
}

#[test]
fn issue_1231_bits_expect_fn_closure() {
    use winnow::binary::bits::{bits, take};
    pub(crate) fn example<'i>(input: &mut &'i [u8]) -> TestResult<&'i [u8], (u8, u8)> {
        bits::<_, _, InputError<_>, _, _>((take(1usize), take(1usize))).parse_next(input)
    }
    assert_parse!(
        example.parse_peek(&[0xff]),
        str![[r#"
Ok(
    (
        [],
        (
            1,
            1,
        ),
    ),
)

"#]]
        .raw()
    );
}

#[test]
fn issue_1282_findtoken_char() {
    use winnow::token::one_of;
    let mut parser = one_of(&['a', 'b', 'c'][..]);
    assert_parse!(
        parser.parse_peek("aaa"),
        str![[r#"
Ok(
    (
        "aa",
        'a',
    ),
)

"#]]
        .raw()
    );
}

#[test]
fn issue_x_looser_fill_bounds() {
    use winnow::{ascii::digit1, combinator::fill, combinator::terminated};

    fn fill_pair<'i>(input: &mut &'i [u8]) -> TestResult<&'i [u8], [&'i [u8]; 2]> {
        let mut buf = [&[][..], &[][..]];
        fill(terminated(digit1, ","), &mut buf).parse_next(input)?;
        Ok(buf)
    }

    assert_parse!(
        fill_pair.parse_peek(b"123,456,"),
        str![[r#"
Ok(
    (
        [],
        [
            [
                49,
                50,
                51,
            ],
            [
                52,
                53,
                54,
            ],
        ],
    ),
)

"#]]
        .raw()
    );
    assert_parse!(
        fill_pair.parse_peek(b"123,456,789"),
        str![[r#"
Ok(
    (
        [
            55,
            56,
            57,
        ],
        [
            [
                49,
                50,
                51,
            ],
            [
                52,
                53,
                54,
            ],
        ],
    ),
)

"#]]
        .raw()
    );
    assert_parse!(
        fill_pair.parse_peek(b"123,,"),
        str![[r#"
Err(
    Backtrack(
        InputError {
            input: [
                44,
            ],
            kind: Slice,
        },
    ),
)

"#]]
        .raw()
    );
}

#[cfg(feature = "std")]
fn issue_1459_clamp_capacity() {
    // shouldn't panic
    use winnow::combinator::repeat;
    let mut parser = repeat(usize::MAX..=usize::MAX, 'a').map(|v: Vec<_>| v);
    assert_parse!(parser.parse_peek("a"), str![]);

    // shouldn't panic
    let mut parser = repeat(usize::MAX, 'a').map(|v: Vec<_>| v);
    assert_parse!(parser.parse_peek("a"), str![]);
}

#[test]
fn issue_1617_count_parser_returning_zero_size() {
    use winnow::combinator::repeat;

    // previously, `repeat()` panicked if the parser had type `O = ()`
    // shouldn't panic
    assert_parse!(
        repeat(3, "abc".map(|_| ()))
            .map(|v: Vec<_>| v)
            .parse_peek("abcabcabcdef"),
        str![[r#"
Ok(
    (
        "def",
        [
            (),
            (),
            (),
        ],
    ),
)

"#]]
        .raw()
    );
}
