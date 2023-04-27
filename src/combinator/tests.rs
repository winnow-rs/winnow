use super::*;

use crate::binary::u16;
use crate::binary::u8;
use crate::binary::Endianness;
use crate::error::ErrMode;
use crate::error::Error;
use crate::error::ErrorKind;
use crate::error::Needed;
use crate::error::ParseError;
use crate::multi::count;
use crate::token::take;
use crate::IResult;
use crate::Parser;
use crate::Partial;

macro_rules! assert_parse(
  ($left: expr, $right: expr) => {
    let res: $crate::IResult<_, _, Error<_>> = $left;
    assert_eq!(res, $right);
  };
);

#[test]
fn eof_on_slices() {
    let not_over: &[u8] = &b"Hello, world!"[..];
    let is_over: &[u8] = &b""[..];

    let res_not_over = eof(not_over);
    assert_parse!(
        res_not_over,
        Err(ErrMode::Backtrack(error_position!(
            not_over,
            ErrorKind::Eof
        )))
    );

    let res_over = eof(is_over);
    assert_parse!(res_over, Ok((is_over, is_over)));
}

#[test]
fn eof_on_strs() {
    let not_over: &str = "Hello, world!";
    let is_over: &str = "";

    let res_not_over = eof(not_over);
    assert_parse!(
        res_not_over,
        Err(ErrMode::Backtrack(error_position!(
            not_over,
            ErrorKind::Eof
        )))
    );

    let res_over = eof(is_over);
    assert_parse!(res_over, Ok((is_over, is_over)));
}

#[test]
fn rest_on_slices() {
    let input: &[u8] = &b"Hello, world!"[..];
    let empty: &[u8] = &b""[..];
    assert_parse!(rest(input), Ok((empty, input)));
}

#[test]
fn rest_on_strs() {
    let input: &str = "Hello, world!";
    let empty: &str = "";
    assert_parse!(rest(input), Ok((empty, input)));
}

#[test]
fn rest_len_on_slices() {
    let input: &[u8] = &b"Hello, world!"[..];
    assert_parse!(rest_len(input), Ok((input, input.len())));
}

use crate::lib::std::convert::From;
impl From<u32> for CustomError {
    fn from(_: u32) -> Self {
        CustomError
    }
}

impl<I> ParseError<I> for CustomError {
    fn from_error_kind(_: I, _: ErrorKind) -> Self {
        CustomError
    }

    fn append(self, _: I, _: ErrorKind) -> Self {
        CustomError
    }
}

struct CustomError;
#[allow(dead_code)]
fn custom_error(input: &[u8]) -> IResult<&[u8], &[u8], CustomError> {
    //fix_error!(input, CustomError<_>, alphanumeric)
    crate::character::alphanumeric1(input)
}

#[test]
fn test_parser_flat_map() {
    let input: &[u8] = &[3, 100, 101, 102, 103, 104][..];
    assert_parse!(
        u8.flat_map(take).parse_next(input),
        Ok((&[103, 104][..], &[100, 101, 102][..]))
    );
}

#[allow(dead_code)]
fn test_closure_compiles_195(input: &[u8]) -> IResult<&[u8], ()> {
    u8.flat_map(|num| count(u16(Endianness::Big), num as usize))
        .parse_next(input)
}

#[test]
fn test_parser_verify_map() {
    let input: &[u8] = &[50][..];
    assert_parse!(
        u8.verify_map(|u| if u < 20 { Some(u) } else { None })
            .parse_next(input),
        Err(ErrMode::Backtrack(Error {
            input: &[50][..],
            kind: ErrorKind::Verify
        }))
    );
    assert_parse!(
        u8.verify_map(|u| if u > 20 { Some(u) } else { None })
            .parse_next(input),
        Ok((&[][..], 50))
    );
}

#[test]
fn test_parser_map_parser() {
    let input: &[u8] = &[100, 101, 102, 103, 104][..];
    assert_parse!(
        take(4usize).and_then(take(2usize)).parse_next(input),
        Ok((&[104][..], &[100, 101][..]))
    );
}

#[test]
#[cfg(feature = "std")]
fn test_parser_into() {
    use crate::error::Error;
    use crate::token::take;

    let mut parser = take::<_, _, Error<_>>(3u8).output_into();
    let result: IResult<&[u8], Vec<u8>> = parser.parse_next(&b"abcdefg"[..]);

    assert_eq!(result, Ok((&b"defg"[..], vec![97, 98, 99])));
}

#[test]
fn opt_test() {
    fn opt_abcd(i: Partial<&[u8]>) -> IResult<Partial<&[u8]>, Option<&[u8]>> {
        opt("abcd").parse_next(i)
    }

    let a = &b"abcdef"[..];
    let b = &b"bcdefg"[..];
    let c = &b"ab"[..];
    assert_eq!(
        opt_abcd(Partial::new(a)),
        Ok((Partial::new(&b"ef"[..]), Some(&b"abcd"[..])))
    );
    assert_eq!(
        opt_abcd(Partial::new(b)),
        Ok((Partial::new(&b"bcdefg"[..]), None))
    );
    assert_eq!(
        opt_abcd(Partial::new(c)),
        Err(ErrMode::Incomplete(Needed::new(2)))
    );
}

#[test]
fn peek_test() {
    fn peek_tag(i: Partial<&[u8]>) -> IResult<Partial<&[u8]>, &[u8]> {
        peek("abcd").parse_next(i)
    }

    assert_eq!(
        peek_tag(Partial::new(&b"abcdef"[..])),
        Ok((Partial::new(&b"abcdef"[..]), &b"abcd"[..]))
    );
    assert_eq!(
        peek_tag(Partial::new(&b"ab"[..])),
        Err(ErrMode::Incomplete(Needed::new(2)))
    );
    assert_eq!(
        peek_tag(Partial::new(&b"xxx"[..])),
        Err(ErrMode::Backtrack(error_position!(
            Partial::new(&b"xxx"[..]),
            ErrorKind::Tag
        )))
    );
}

#[test]
fn not_test() {
    fn not_aaa(i: Partial<&[u8]>) -> IResult<Partial<&[u8]>, ()> {
        not("aaa").parse_next(i)
    }

    assert_eq!(
        not_aaa(Partial::new(&b"aaa"[..])),
        Err(ErrMode::Backtrack(error_position!(
            Partial::new(&b"aaa"[..]),
            ErrorKind::Not
        )))
    );
    assert_eq!(
        not_aaa(Partial::new(&b"aa"[..])),
        Err(ErrMode::Incomplete(Needed::new(1)))
    );
    assert_eq!(
        not_aaa(Partial::new(&b"abcd"[..])),
        Ok((Partial::new(&b"abcd"[..]), ()))
    );
}

#[test]
fn test_parser_verify() {
    use crate::token::take;

    fn test(i: Partial<&[u8]>) -> IResult<Partial<&[u8]>, &[u8]> {
        take(5u8)
            .verify(|slice: &[u8]| slice[0] == b'a')
            .parse_next(i)
    }
    assert_eq!(
        test(Partial::new(&b"bcd"[..])),
        Err(ErrMode::Incomplete(Needed::new(2)))
    );
    assert_eq!(
        test(Partial::new(&b"bcdefg"[..])),
        Err(ErrMode::Backtrack(error_position!(
            Partial::new(&b"bcdefg"[..]),
            ErrorKind::Verify
        )))
    );
    assert_eq!(
        test(Partial::new(&b"abcdefg"[..])),
        Ok((Partial::new(&b"fg"[..]), &b"abcde"[..]))
    );
}

#[test]
#[allow(unused)]
fn test_parser_verify_ref() {
    use crate::token::take;

    let mut parser1 = take(3u8).verify(|s: &[u8]| s == &b"abc"[..]);

    assert_eq!(
        parser1.parse_next(&b"abcd"[..]),
        Ok((&b"d"[..], &b"abc"[..]))
    );
    assert_eq!(
        parser1.parse_next(&b"defg"[..]),
        Err(ErrMode::Backtrack(Error {
            input: &b"defg"[..],
            kind: ErrorKind::Verify
        }))
    );

    fn parser2(i: &[u8]) -> IResult<&[u8], u32> {
        crate::binary::be_u32
            .verify(|val: &u32| *val < 3)
            .parse_next(i)
    }
}

#[test]
#[cfg(feature = "alloc")]
fn test_parser_verify_alloc() {
    use crate::token::take;
    let mut parser1 = take(3u8)
        .map(|s: &[u8]| s.to_vec())
        .verify(|s: &[u8]| s == &b"abc"[..]);

    assert_eq!(
        parser1.parse_next(&b"abcd"[..]),
        Ok((&b"d"[..], b"abc".to_vec()))
    );
    assert_eq!(
        parser1.parse_next(&b"defg"[..]),
        Err(ErrMode::Backtrack(Error {
            input: &b"defg"[..],
            kind: ErrorKind::Verify
        }))
    );
}

#[test]
fn fail_test() {
    let a = "string";
    let b = "another string";

    assert_eq!(
        fail::<_, &str, _>(a),
        Err(ErrMode::Backtrack(Error {
            input: a,
            kind: ErrorKind::Fail
        }))
    );
    assert_eq!(
        fail::<_, &str, _>(b),
        Err(ErrMode::Backtrack(Error {
            input: b,
            kind: ErrorKind::Fail
        }))
    );
}

#[test]
fn complete() {
    fn err_test(i: &[u8]) -> IResult<&[u8], &[u8]> {
        let (i, _) = "ijkl".parse_next(i)?;
        "mnop".parse_next(i)
    }
    let a = &b"ijklmn"[..];

    let res_a = err_test(a);
    assert_eq!(
        res_a,
        Err(ErrMode::Backtrack(error_position!(
            &b"mn"[..],
            ErrorKind::Tag
        )))
    );
}

#[test]
fn separated_pair_test() {
    #[allow(clippy::type_complexity)]
    fn sep_pair_abc_def(i: Partial<&[u8]>) -> IResult<Partial<&[u8]>, (&[u8], &[u8])> {
        separated_pair("abc", ",", "def").parse_next(i)
    }

    assert_eq!(
        sep_pair_abc_def(Partial::new(&b"abc,defghijkl"[..])),
        Ok((Partial::new(&b"ghijkl"[..]), (&b"abc"[..], &b"def"[..])))
    );
    assert_eq!(
        sep_pair_abc_def(Partial::new(&b"ab"[..])),
        Err(ErrMode::Incomplete(Needed::new(1)))
    );
    assert_eq!(
        sep_pair_abc_def(Partial::new(&b"abc,d"[..])),
        Err(ErrMode::Incomplete(Needed::new(2)))
    );
    assert_eq!(
        sep_pair_abc_def(Partial::new(&b"xxx"[..])),
        Err(ErrMode::Backtrack(error_position!(
            Partial::new(&b"xxx"[..]),
            ErrorKind::Tag
        )))
    );
    assert_eq!(
        sep_pair_abc_def(Partial::new(&b"xxx,def"[..])),
        Err(ErrMode::Backtrack(error_position!(
            Partial::new(&b"xxx,def"[..]),
            ErrorKind::Tag
        )))
    );
    assert_eq!(
        sep_pair_abc_def(Partial::new(&b"abc,xxx"[..])),
        Err(ErrMode::Backtrack(error_position!(
            Partial::new(&b"xxx"[..]),
            ErrorKind::Tag
        )))
    );
}

#[test]
fn preceded_test() {
    fn preceded_abcd_efgh(i: Partial<&[u8]>) -> IResult<Partial<&[u8]>, &[u8]> {
        preceded("abcd", "efgh").parse_next(i)
    }

    assert_eq!(
        preceded_abcd_efgh(Partial::new(&b"abcdefghijkl"[..])),
        Ok((Partial::new(&b"ijkl"[..]), &b"efgh"[..]))
    );
    assert_eq!(
        preceded_abcd_efgh(Partial::new(&b"ab"[..])),
        Err(ErrMode::Incomplete(Needed::new(2)))
    );
    assert_eq!(
        preceded_abcd_efgh(Partial::new(&b"abcde"[..])),
        Err(ErrMode::Incomplete(Needed::new(3)))
    );
    assert_eq!(
        preceded_abcd_efgh(Partial::new(&b"xxx"[..])),
        Err(ErrMode::Backtrack(error_position!(
            Partial::new(&b"xxx"[..]),
            ErrorKind::Tag
        )))
    );
    assert_eq!(
        preceded_abcd_efgh(Partial::new(&b"xxxxdef"[..])),
        Err(ErrMode::Backtrack(error_position!(
            Partial::new(&b"xxxxdef"[..]),
            ErrorKind::Tag
        )))
    );
    assert_eq!(
        preceded_abcd_efgh(Partial::new(&b"abcdxxx"[..])),
        Err(ErrMode::Backtrack(error_position!(
            Partial::new(&b"xxx"[..]),
            ErrorKind::Tag
        )))
    );
}

#[test]
fn terminated_test() {
    fn terminated_abcd_efgh(i: Partial<&[u8]>) -> IResult<Partial<&[u8]>, &[u8]> {
        terminated("abcd", "efgh").parse_next(i)
    }

    assert_eq!(
        terminated_abcd_efgh(Partial::new(&b"abcdefghijkl"[..])),
        Ok((Partial::new(&b"ijkl"[..]), &b"abcd"[..]))
    );
    assert_eq!(
        terminated_abcd_efgh(Partial::new(&b"ab"[..])),
        Err(ErrMode::Incomplete(Needed::new(2)))
    );
    assert_eq!(
        terminated_abcd_efgh(Partial::new(&b"abcde"[..])),
        Err(ErrMode::Incomplete(Needed::new(3)))
    );
    assert_eq!(
        terminated_abcd_efgh(Partial::new(&b"xxx"[..])),
        Err(ErrMode::Backtrack(error_position!(
            Partial::new(&b"xxx"[..]),
            ErrorKind::Tag
        )))
    );
    assert_eq!(
        terminated_abcd_efgh(Partial::new(&b"xxxxdef"[..])),
        Err(ErrMode::Backtrack(error_position!(
            Partial::new(&b"xxxxdef"[..]),
            ErrorKind::Tag
        )))
    );
    assert_eq!(
        terminated_abcd_efgh(Partial::new(&b"abcdxxxx"[..])),
        Err(ErrMode::Backtrack(error_position!(
            Partial::new(&b"xxxx"[..]),
            ErrorKind::Tag
        )))
    );
}

#[test]
fn delimited_test() {
    fn delimited_abc_def_ghi(i: Partial<&[u8]>) -> IResult<Partial<&[u8]>, &[u8]> {
        delimited("abc", "def", "ghi").parse_next(i)
    }

    assert_eq!(
        delimited_abc_def_ghi(Partial::new(&b"abcdefghijkl"[..])),
        Ok((Partial::new(&b"jkl"[..]), &b"def"[..]))
    );
    assert_eq!(
        delimited_abc_def_ghi(Partial::new(&b"ab"[..])),
        Err(ErrMode::Incomplete(Needed::new(1)))
    );
    assert_eq!(
        delimited_abc_def_ghi(Partial::new(&b"abcde"[..])),
        Err(ErrMode::Incomplete(Needed::new(1)))
    );
    assert_eq!(
        delimited_abc_def_ghi(Partial::new(&b"abcdefgh"[..])),
        Err(ErrMode::Incomplete(Needed::new(1)))
    );
    assert_eq!(
        delimited_abc_def_ghi(Partial::new(&b"xxx"[..])),
        Err(ErrMode::Backtrack(error_position!(
            Partial::new(&b"xxx"[..]),
            ErrorKind::Tag
        )))
    );
    assert_eq!(
        delimited_abc_def_ghi(Partial::new(&b"xxxdefghi"[..])),
        Err(ErrMode::Backtrack(error_position!(
            Partial::new(&b"xxxdefghi"[..]),
            ErrorKind::Tag
        ),))
    );
    assert_eq!(
        delimited_abc_def_ghi(Partial::new(&b"abcxxxghi"[..])),
        Err(ErrMode::Backtrack(error_position!(
            Partial::new(&b"xxxghi"[..]),
            ErrorKind::Tag
        )))
    );
    assert_eq!(
        delimited_abc_def_ghi(Partial::new(&b"abcdefxxx"[..])),
        Err(ErrMode::Backtrack(error_position!(
            Partial::new(&b"xxx"[..]),
            ErrorKind::Tag
        )))
    );
}

#[cfg(feature = "alloc")]
#[test]
fn alt_test() {
    #[cfg(feature = "alloc")]
    use crate::{
        error::ParseError,
        lib::std::{
            fmt::Debug,
            string::{String, ToString},
        },
    };

    #[cfg(feature = "alloc")]
    #[derive(Debug, Clone, Eq, PartialEq)]
    pub struct ErrorStr(String);

    #[cfg(feature = "alloc")]
    impl From<u32> for ErrorStr {
        fn from(i: u32) -> Self {
            ErrorStr(format!("custom error code: {}", i))
        }
    }

    #[cfg(feature = "alloc")]
    impl<'a> From<&'a str> for ErrorStr {
        fn from(i: &'a str) -> Self {
            ErrorStr(format!("custom error message: {}", i))
        }
    }

    #[cfg(feature = "alloc")]
    impl<I: Debug> ParseError<I> for ErrorStr {
        fn from_error_kind(input: I, kind: ErrorKind) -> Self {
            ErrorStr(format!("custom error message: ({:?}, {:?})", input, kind))
        }

        fn append(self, input: I, kind: ErrorKind) -> Self {
            ErrorStr(format!(
                "custom error message: ({:?}, {:?}) - {:?}",
                input, kind, self
            ))
        }
    }

    fn work(input: &[u8]) -> IResult<&[u8], &[u8], ErrorStr> {
        Ok((&b""[..], input))
    }

    #[allow(unused_variables)]
    fn dont_work(input: &[u8]) -> IResult<&[u8], &[u8], ErrorStr> {
        Err(ErrMode::Backtrack(ErrorStr("abcd".to_string())))
    }

    fn work2(input: &[u8]) -> IResult<&[u8], &[u8], ErrorStr> {
        Ok((input, &b""[..]))
    }

    fn alt1(i: &[u8]) -> IResult<&[u8], &[u8], ErrorStr> {
        alt((dont_work, dont_work)).parse_next(i)
    }
    fn alt2(i: &[u8]) -> IResult<&[u8], &[u8], ErrorStr> {
        alt((dont_work, work)).parse_next(i)
    }
    fn alt3(i: &[u8]) -> IResult<&[u8], &[u8], ErrorStr> {
        alt((dont_work, dont_work, work2, dont_work)).parse_next(i)
    }
    //named!(alt1, alt!(dont_work | dont_work));
    //named!(alt2, alt!(dont_work | work));
    //named!(alt3, alt!(dont_work | dont_work | work2 | dont_work));

    let a = &b"abcd"[..];
    assert_eq!(
        alt1(a),
        Err(ErrMode::Backtrack(error_node_position!(
            a,
            ErrorKind::Alt,
            ErrorStr("abcd".to_string())
        )))
    );
    assert_eq!(alt2(a), Ok((&b""[..], a)));
    assert_eq!(alt3(a), Ok((a, &b""[..])));

    fn alt4(i: &[u8]) -> IResult<&[u8], &[u8]> {
        alt(("abcd", "efgh")).parse_next(i)
    }
    let b = &b"efgh"[..];
    assert_eq!(alt4(a), Ok((&b""[..], a)));
    assert_eq!(alt4(b), Ok((&b""[..], b)));
}

#[test]
fn alt_incomplete() {
    fn alt1(i: Partial<&[u8]>) -> IResult<Partial<&[u8]>, &[u8]> {
        alt(("a", "bc", "def")).parse_next(i)
    }

    let a = &b""[..];
    assert_eq!(
        alt1(Partial::new(a)),
        Err(ErrMode::Incomplete(Needed::new(1)))
    );
    let a = &b"b"[..];
    assert_eq!(
        alt1(Partial::new(a)),
        Err(ErrMode::Incomplete(Needed::new(1)))
    );
    let a = &b"bcd"[..];
    assert_eq!(
        alt1(Partial::new(a)),
        Ok((Partial::new(&b"d"[..]), &b"bc"[..]))
    );
    let a = &b"cde"[..];
    assert_eq!(
        alt1(Partial::new(a)),
        Err(ErrMode::Backtrack(error_position!(
            Partial::new(a),
            ErrorKind::Tag
        )))
    );
    let a = &b"de"[..];
    assert_eq!(
        alt1(Partial::new(a)),
        Err(ErrMode::Incomplete(Needed::new(1)))
    );
    let a = &b"defg"[..];
    assert_eq!(
        alt1(Partial::new(a)),
        Ok((Partial::new(&b"g"[..]), &b"def"[..]))
    );
}

#[test]
fn permutation_test() {
    #[allow(clippy::type_complexity)]
    fn perm(i: Partial<&[u8]>) -> IResult<Partial<&[u8]>, (&[u8], &[u8], &[u8])> {
        permutation(("abcd", "efg", "hi")).parse_next(i)
    }

    let expected = (&b"abcd"[..], &b"efg"[..], &b"hi"[..]);

    let a = &b"abcdefghijk"[..];
    assert_eq!(
        perm(Partial::new(a)),
        Ok((Partial::new(&b"jk"[..]), expected))
    );
    let b = &b"efgabcdhijk"[..];
    assert_eq!(
        perm(Partial::new(b)),
        Ok((Partial::new(&b"jk"[..]), expected))
    );
    let c = &b"hiefgabcdjk"[..];
    assert_eq!(
        perm(Partial::new(c)),
        Ok((Partial::new(&b"jk"[..]), expected))
    );

    let d = &b"efgxyzabcdefghi"[..];
    assert_eq!(
        perm(Partial::new(d)),
        Err(ErrMode::Backtrack(error_node_position!(
            Partial::new(&b"efgxyzabcdefghi"[..]),
            ErrorKind::Alt,
            error_position!(Partial::new(&b"xyzabcdefghi"[..]), ErrorKind::Tag)
        )))
    );

    let e = &b"efgabc"[..];
    assert_eq!(
        perm(Partial::new(e)),
        Err(ErrMode::Incomplete(Needed::new(1)))
    );
}
