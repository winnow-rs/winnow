use super::*;

#[cfg(feature = "std")]
use proptest::prelude::*;

use crate::bytes::tag;
use crate::error::ErrMode;
use crate::error::Error;
use crate::error::ErrorKind;
use crate::error::Needed;
use crate::multi::length_data;
use crate::sequence::delimited;
use crate::stream::AsChar;
use crate::IResult;
use crate::Parser;
use crate::Partial;

#[test]
fn complete_take_while_m_n_utf8_all_matching() {
    let result: IResult<&str, &str> = take_while_m_n(1, 4, |c: char| c.is_alphabetic())("øn");
    assert_eq!(result, Ok(("", "øn")));
}

#[test]
fn complete_take_while_m_n_utf8_all_matching_substring() {
    let result: IResult<&str, &str> = take_while_m_n(1, 1, |c: char| c.is_alphabetic())("øn");
    assert_eq!(result, Ok(("n", "ø")));
}

#[cfg(feature = "std")]
fn model_complete_take_while_m_n(
    m: usize,
    n: usize,
    valid: usize,
    input: &str,
) -> IResult<&str, &str> {
    if n < m {
        Err(crate::error::ErrMode::from_error_kind(
            input,
            crate::error::ErrorKind::TakeWhileMN,
        ))
    } else if m <= valid {
        let offset = n.min(valid);
        Ok((&input[offset..], &input[0..offset]))
    } else {
        Err(crate::error::ErrMode::from_error_kind(
            input,
            crate::error::ErrorKind::TakeWhileMN,
        ))
    }
}

#[cfg(feature = "std")]
proptest! {
  #[test]
  #[cfg_attr(miri, ignore)]  // See https://github.com/AltSysrq/proptest/issues/253
  fn complete_take_while_m_n_bounds(m in 0..20usize, n in 0..20usize, valid in 0..20usize, invalid in 0..20usize) {
      let input = format!("{:a<valid$}{:b<invalid$}", "", "", valid=valid, invalid=invalid);
      let expected = model_complete_take_while_m_n(m, n, valid, &input);
      let actual = take_while_m_n(m, n, |c: char| c == 'a')(input.as_str());
      assert_eq!(expected, actual);
  }
}

#[test]
fn partial_any_str() {
    use super::any;
    assert_eq!(
        any::<_, Error<Partial<&str>>>(Partial("Ә")),
        Ok((Partial(""), 'Ә'))
    );
}

#[test]
fn partial_one_of_test() {
    fn f(i: Partial<&[u8]>) -> IResult<Partial<&[u8]>, u8> {
        one_of("ab")(i)
    }

    let a = &b"abcd"[..];
    assert_eq!(f(Partial(a)), Ok((Partial(&b"bcd"[..]), b'a')));

    let b = &b"cde"[..];
    assert_eq!(
        f(Partial(b)),
        Err(ErrMode::Backtrack(error_position!(
            Partial(b),
            ErrorKind::OneOf
        )))
    );

    fn utf8(i: Partial<&str>) -> IResult<Partial<&str>, char> {
        one_of("+\u{FF0B}")(i)
    }

    assert!(utf8(Partial("+")).is_ok());
    assert!(utf8(Partial("\u{FF0B}")).is_ok());
}

#[test]
fn char_byteslice() {
    fn f(i: Partial<&[u8]>) -> IResult<Partial<&[u8]>, u8> {
        one_of('c')(i)
    }

    let a = &b"abcd"[..];
    assert_eq!(
        f(Partial(a)),
        Err(ErrMode::Backtrack(error_position!(
            Partial(a),
            ErrorKind::OneOf
        )))
    );

    let b = &b"cde"[..];
    assert_eq!(f(Partial(b)), Ok((Partial(&b"de"[..]), b'c')));
}

#[test]
fn char_str() {
    fn f(i: Partial<&str>) -> IResult<Partial<&str>, char> {
        one_of('c')(i)
    }

    let a = "abcd";
    assert_eq!(
        f(Partial(a)),
        Err(ErrMode::Backtrack(error_position!(
            Partial(a),
            ErrorKind::OneOf
        )))
    );

    let b = "cde";
    assert_eq!(f(Partial(b)), Ok((Partial("de"), 'c')));
}

#[test]
fn partial_none_of_test() {
    fn f(i: Partial<&[u8]>) -> IResult<Partial<&[u8]>, u8> {
        none_of("ab")(i)
    }

    let a = &b"abcd"[..];
    assert_eq!(
        f(Partial(a)),
        Err(ErrMode::Backtrack(error_position!(
            Partial(a),
            ErrorKind::NoneOf
        )))
    );

    let b = &b"cde"[..];
    assert_eq!(f(Partial(b)), Ok((Partial(&b"de"[..]), b'c')));
}

#[test]
fn partial_is_a() {
    fn a_or_b(i: Partial<&[u8]>) -> IResult<Partial<&[u8]>, &[u8]> {
        take_while1("ab")(i)
    }

    let a = Partial(&b"abcd"[..]);
    assert_eq!(a_or_b(a), Ok((Partial(&b"cd"[..]), &b"ab"[..])));

    let b = Partial(&b"bcde"[..]);
    assert_eq!(a_or_b(b), Ok((Partial(&b"cde"[..]), &b"b"[..])));

    let c = Partial(&b"cdef"[..]);
    assert_eq!(
        a_or_b(c),
        Err(ErrMode::Backtrack(error_position!(
            c,
            ErrorKind::TakeWhile1
        )))
    );

    let d = Partial(&b"bacdef"[..]);
    assert_eq!(a_or_b(d), Ok((Partial(&b"cdef"[..]), &b"ba"[..])));
}

#[test]
fn partial_is_not() {
    fn a_or_b(i: Partial<&[u8]>) -> IResult<Partial<&[u8]>, &[u8]> {
        take_till1("ab")(i)
    }

    let a = Partial(&b"cdab"[..]);
    assert_eq!(a_or_b(a), Ok((Partial(&b"ab"[..]), &b"cd"[..])));

    let b = Partial(&b"cbde"[..]);
    assert_eq!(a_or_b(b), Ok((Partial(&b"bde"[..]), &b"c"[..])));

    let c = Partial(&b"abab"[..]);
    assert_eq!(
        a_or_b(c),
        Err(ErrMode::Backtrack(error_position!(c, ErrorKind::TakeTill1)))
    );

    let d = Partial(&b"cdefba"[..]);
    assert_eq!(a_or_b(d), Ok((Partial(&b"ba"[..]), &b"cdef"[..])));

    let e = Partial(&b"e"[..]);
    assert_eq!(a_or_b(e), Err(ErrMode::Incomplete(Needed::new(1))));
}

#[test]
fn partial_take_until_incomplete() {
    fn y(i: Partial<&[u8]>) -> IResult<Partial<&[u8]>, &[u8]> {
        take_until0("end")(i)
    }
    assert_eq!(
        y(Partial(&b"nd"[..])),
        Err(ErrMode::Incomplete(Needed::Unknown))
    );
    assert_eq!(
        y(Partial(&b"123"[..])),
        Err(ErrMode::Incomplete(Needed::Unknown))
    );
    assert_eq!(
        y(Partial(&b"123en"[..])),
        Err(ErrMode::Incomplete(Needed::Unknown))
    );
}

#[test]
fn partial_take_until_incomplete_s() {
    fn ys(i: Partial<&str>) -> IResult<Partial<&str>, &str> {
        take_until0("end")(i)
    }
    assert_eq!(
        ys(Partial("123en")),
        Err(ErrMode::Incomplete(Needed::Unknown))
    );
}

#[test]
fn partial_recognize() {
    use crate::character::{
        alpha1 as alpha, alphanumeric1 as alphanumeric, digit1 as digit, hex_digit1 as hex_digit,
        multispace1 as multispace, oct_digit1 as oct_digit, space1 as space,
    };

    fn x(i: Partial<&[u8]>) -> IResult<Partial<&[u8]>, &[u8]> {
        delimited(tag("<!--"), take(5_usize), tag("-->"))
            .recognize()
            .parse_next(i)
    }
    let r = x(Partial(&b"<!-- abc --> aaa"[..]));
    assert_eq!(r, Ok((Partial(&b" aaa"[..]), &b"<!-- abc -->"[..])));

    let semicolon = &b";"[..];

    fn ya(i: Partial<&[u8]>) -> IResult<Partial<&[u8]>, &[u8]> {
        alpha.recognize().parse_next(i)
    }
    let ra = ya(Partial(&b"abc;"[..]));
    assert_eq!(ra, Ok((Partial(semicolon), &b"abc"[..])));

    fn yd(i: Partial<&[u8]>) -> IResult<Partial<&[u8]>, &[u8]> {
        digit.recognize().parse_next(i)
    }
    let rd = yd(Partial(&b"123;"[..]));
    assert_eq!(rd, Ok((Partial(semicolon), &b"123"[..])));

    fn yhd(i: Partial<&[u8]>) -> IResult<Partial<&[u8]>, &[u8]> {
        hex_digit.recognize().parse_next(i)
    }
    let rhd = yhd(Partial(&b"123abcDEF;"[..]));
    assert_eq!(rhd, Ok((Partial(semicolon), &b"123abcDEF"[..])));

    fn yod(i: Partial<&[u8]>) -> IResult<Partial<&[u8]>, &[u8]> {
        oct_digit.recognize().parse_next(i)
    }
    let rod = yod(Partial(&b"1234567;"[..]));
    assert_eq!(rod, Ok((Partial(semicolon), &b"1234567"[..])));

    fn yan(i: Partial<&[u8]>) -> IResult<Partial<&[u8]>, &[u8]> {
        alphanumeric.recognize().parse_next(i)
    }
    let ran = yan(Partial(&b"123abc;"[..]));
    assert_eq!(ran, Ok((Partial(semicolon), &b"123abc"[..])));

    fn ys(i: Partial<&[u8]>) -> IResult<Partial<&[u8]>, &[u8]> {
        space.recognize().parse_next(i)
    }
    let rs = ys(Partial(&b" \t;"[..]));
    assert_eq!(rs, Ok((Partial(semicolon), &b" \t"[..])));

    fn yms(i: Partial<&[u8]>) -> IResult<Partial<&[u8]>, &[u8]> {
        multispace.recognize().parse_next(i)
    }
    let rms = yms(Partial(&b" \t\r\n;"[..]));
    assert_eq!(rms, Ok((Partial(semicolon), &b" \t\r\n"[..])));
}

#[test]
fn partial_take_while0() {
    fn f(i: Partial<&[u8]>) -> IResult<Partial<&[u8]>, &[u8]> {
        take_while0(AsChar::is_alpha)(i)
    }
    let a = &b""[..];
    let b = &b"abcd"[..];
    let c = &b"abcd123"[..];
    let d = &b"123"[..];

    assert_eq!(f(Partial(a)), Err(ErrMode::Incomplete(Needed::new(1))));
    assert_eq!(f(Partial(b)), Err(ErrMode::Incomplete(Needed::new(1))));
    assert_eq!(f(Partial(c)), Ok((Partial(d), b)));
    assert_eq!(f(Partial(d)), Ok((Partial(d), a)));
}

#[test]
fn partial_take_while1() {
    fn f(i: Partial<&[u8]>) -> IResult<Partial<&[u8]>, &[u8]> {
        take_while1(AsChar::is_alpha)(i)
    }
    let a = &b""[..];
    let b = &b"abcd"[..];
    let c = &b"abcd123"[..];
    let d = &b"123"[..];

    assert_eq!(f(Partial(a)), Err(ErrMode::Incomplete(Needed::new(1))));
    assert_eq!(f(Partial(b)), Err(ErrMode::Incomplete(Needed::new(1))));
    assert_eq!(f(Partial(c)), Ok((Partial(&b"123"[..]), b)));
    assert_eq!(
        f(Partial(d)),
        Err(ErrMode::Backtrack(error_position!(
            Partial(d),
            ErrorKind::TakeWhile1
        )))
    );
}

#[test]
fn partial_take_while_m_n() {
    fn x(i: Partial<&[u8]>) -> IResult<Partial<&[u8]>, &[u8]> {
        take_while_m_n(2, 4, AsChar::is_alpha)(i)
    }
    let a = &b""[..];
    let b = &b"a"[..];
    let c = &b"abc"[..];
    let d = &b"abc123"[..];
    let e = &b"abcde"[..];
    let f = &b"123"[..];

    assert_eq!(x(Partial(a)), Err(ErrMode::Incomplete(Needed::new(2))));
    assert_eq!(x(Partial(b)), Err(ErrMode::Incomplete(Needed::new(1))));
    assert_eq!(x(Partial(c)), Err(ErrMode::Incomplete(Needed::new(1))));
    assert_eq!(x(Partial(d)), Ok((Partial(&b"123"[..]), c)));
    assert_eq!(x(Partial(e)), Ok((Partial(&b"e"[..]), &b"abcd"[..])));
    assert_eq!(
        x(Partial(f)),
        Err(ErrMode::Backtrack(error_position!(
            Partial(f),
            ErrorKind::TakeWhileMN
        )))
    );
}

#[test]
fn partial_take_till0() {
    fn f(i: Partial<&[u8]>) -> IResult<Partial<&[u8]>, &[u8]> {
        take_till0(AsChar::is_alpha)(i)
    }
    let a = &b""[..];
    let b = &b"abcd"[..];
    let c = &b"123abcd"[..];
    let d = &b"123"[..];

    assert_eq!(f(Partial(a)), Err(ErrMode::Incomplete(Needed::new(1))));
    assert_eq!(f(Partial(b)), Ok((Partial(&b"abcd"[..]), &b""[..])));
    assert_eq!(f(Partial(c)), Ok((Partial(&b"abcd"[..]), &b"123"[..])));
    assert_eq!(f(Partial(d)), Err(ErrMode::Incomplete(Needed::new(1))));
}

#[test]
fn partial_take_till1() {
    fn f(i: Partial<&[u8]>) -> IResult<Partial<&[u8]>, &[u8]> {
        take_till1(AsChar::is_alpha)(i)
    }
    let a = &b""[..];
    let b = &b"abcd"[..];
    let c = &b"123abcd"[..];
    let d = &b"123"[..];

    assert_eq!(f(Partial(a)), Err(ErrMode::Incomplete(Needed::new(1))));
    assert_eq!(
        f(Partial(b)),
        Err(ErrMode::Backtrack(error_position!(
            Partial(b),
            ErrorKind::TakeTill1
        )))
    );
    assert_eq!(f(Partial(c)), Ok((Partial(&b"abcd"[..]), &b"123"[..])));
    assert_eq!(f(Partial(d)), Err(ErrMode::Incomplete(Needed::new(1))));
}

#[test]
fn partial_take_while_utf8() {
    fn f(i: Partial<&str>) -> IResult<Partial<&str>, &str> {
        take_while0(|c| c != '點')(i)
    }

    assert_eq!(f(Partial("")), Err(ErrMode::Incomplete(Needed::new(1))));
    assert_eq!(f(Partial("abcd")), Err(ErrMode::Incomplete(Needed::new(1))));
    assert_eq!(f(Partial("abcd點")), Ok((Partial("點"), "abcd")));
    assert_eq!(f(Partial("abcd點a")), Ok((Partial("點a"), "abcd")));

    fn g(i: Partial<&str>) -> IResult<Partial<&str>, &str> {
        take_while0(|c| c == '點')(i)
    }

    assert_eq!(g(Partial("")), Err(ErrMode::Incomplete(Needed::new(1))));
    assert_eq!(g(Partial("點abcd")), Ok((Partial("abcd"), "點")));
    assert_eq!(g(Partial("點點點a")), Ok((Partial("a"), "點點點")));
}

#[test]
fn partial_take_till0_utf8() {
    fn f(i: Partial<&str>) -> IResult<Partial<&str>, &str> {
        take_till0(|c| c == '點')(i)
    }

    assert_eq!(f(Partial("")), Err(ErrMode::Incomplete(Needed::new(1))));
    assert_eq!(f(Partial("abcd")), Err(ErrMode::Incomplete(Needed::new(1))));
    assert_eq!(f(Partial("abcd點")), Ok((Partial("點"), "abcd")));
    assert_eq!(f(Partial("abcd點a")), Ok((Partial("點a"), "abcd")));

    fn g(i: Partial<&str>) -> IResult<Partial<&str>, &str> {
        take_till0(|c| c != '點')(i)
    }

    assert_eq!(g(Partial("")), Err(ErrMode::Incomplete(Needed::new(1))));
    assert_eq!(g(Partial("點abcd")), Ok((Partial("abcd"), "點")));
    assert_eq!(g(Partial("點點點a")), Ok((Partial("a"), "點點點")));
}

#[test]
fn partial_take_utf8() {
    fn f(i: Partial<&str>) -> IResult<Partial<&str>, &str> {
        take(3_usize)(i)
    }

    assert_eq!(f(Partial("")), Err(ErrMode::Incomplete(Needed::Unknown)));
    assert_eq!(f(Partial("ab")), Err(ErrMode::Incomplete(Needed::Unknown)));
    assert_eq!(f(Partial("點")), Err(ErrMode::Incomplete(Needed::Unknown)));
    assert_eq!(f(Partial("ab點cd")), Ok((Partial("cd"), "ab點")));
    assert_eq!(f(Partial("a點bcd")), Ok((Partial("cd"), "a點b")));
    assert_eq!(f(Partial("a點b")), Ok((Partial(""), "a點b")));

    fn g(i: Partial<&str>) -> IResult<Partial<&str>, &str> {
        take_while0(|c| c == '點')(i)
    }

    assert_eq!(g(Partial("")), Err(ErrMode::Incomplete(Needed::new(1))));
    assert_eq!(g(Partial("點abcd")), Ok((Partial("abcd"), "點")));
    assert_eq!(g(Partial("點點點a")), Ok((Partial("a"), "點點點")));
}

#[test]
fn partial_take_while_m_n_utf8_fixed() {
    fn parser(i: Partial<&str>) -> IResult<Partial<&str>, &str> {
        take_while_m_n(1, 1, |c| c == 'A' || c == '😃')(i)
    }
    assert_eq!(parser(Partial("A!")), Ok((Partial("!"), "A")));
    assert_eq!(parser(Partial("😃!")), Ok((Partial("!"), "😃")));
}

#[test]
fn partial_take_while_m_n_utf8_range() {
    fn parser(i: Partial<&str>) -> IResult<Partial<&str>, &str> {
        take_while_m_n(1, 2, |c| c == 'A' || c == '😃')(i)
    }
    assert_eq!(parser(Partial("A!")), Ok((Partial("!"), "A")));
    assert_eq!(parser(Partial("😃!")), Ok((Partial("!"), "😃")));
}

#[test]
fn partial_take_while_m_n_utf8_full_match_fixed() {
    fn parser(i: Partial<&str>) -> IResult<Partial<&str>, &str> {
        take_while_m_n(1, 1, |c: char| c.is_alphabetic())(i)
    }
    assert_eq!(parser(Partial("øn")), Ok((Partial("n"), "ø")));
}

#[test]
fn partial_take_while_m_n_utf8_full_match_range() {
    fn parser(i: Partial<&str>) -> IResult<Partial<&str>, &str> {
        take_while_m_n(1, 2, |c: char| c.is_alphabetic())(i)
    }
    assert_eq!(parser(Partial("øn")), Ok((Partial(""), "øn")));
}

#[test]
#[cfg(feature = "std")]
fn partial_recognize_take_while0() {
    fn x(i: Partial<&[u8]>) -> IResult<Partial<&[u8]>, &[u8]> {
        take_while0(AsChar::is_alphanum)(i)
    }
    fn y(i: Partial<&[u8]>) -> IResult<Partial<&[u8]>, &[u8]> {
        x.recognize().parse_next(i)
    }
    assert_eq!(
        x(Partial(&b"ab."[..])),
        Ok((Partial(&b"."[..]), &b"ab"[..]))
    );
    assert_eq!(
        y(Partial(&b"ab."[..])),
        Ok((Partial(&b"."[..]), &b"ab"[..]))
    );
}

#[test]
fn partial_length_bytes() {
    use crate::number::le_u8;

    fn x(i: Partial<&[u8]>) -> IResult<Partial<&[u8]>, &[u8]> {
        length_data(le_u8)(i)
    }
    assert_eq!(
        x(Partial(b"\x02..>>")),
        Ok((Partial(&b">>"[..]), &b".."[..]))
    );
    assert_eq!(x(Partial(b"\x02..")), Ok((Partial(&[][..]), &b".."[..])));
    assert_eq!(
        x(Partial(b"\x02.")),
        Err(ErrMode::Incomplete(Needed::new(1)))
    );
    assert_eq!(
        x(Partial(b"\x02")),
        Err(ErrMode::Incomplete(Needed::new(2)))
    );

    fn y(i: Partial<&[u8]>) -> IResult<Partial<&[u8]>, &[u8]> {
        let (i, _) = tag("magic")(i)?;
        length_data(le_u8)(i)
    }
    assert_eq!(
        y(Partial(b"magic\x02..>>")),
        Ok((Partial(&b">>"[..]), &b".."[..]))
    );
    assert_eq!(
        y(Partial(b"magic\x02..")),
        Ok((Partial(&[][..]), &b".."[..]))
    );
    assert_eq!(
        y(Partial(b"magic\x02.")),
        Err(ErrMode::Incomplete(Needed::new(1)))
    );
    assert_eq!(
        y(Partial(b"magic\x02")),
        Err(ErrMode::Incomplete(Needed::new(2)))
    );
}

#[cfg(feature = "alloc")]
#[test]
fn partial_case_insensitive() {
    fn test(i: Partial<&[u8]>) -> IResult<Partial<&[u8]>, &[u8]> {
        tag_no_case("ABcd")(i)
    }
    assert_eq!(
        test(Partial(&b"aBCdefgh"[..])),
        Ok((Partial(&b"efgh"[..]), &b"aBCd"[..]))
    );
    assert_eq!(
        test(Partial(&b"abcdefgh"[..])),
        Ok((Partial(&b"efgh"[..]), &b"abcd"[..]))
    );
    assert_eq!(
        test(Partial(&b"ABCDefgh"[..])),
        Ok((Partial(&b"efgh"[..]), &b"ABCD"[..]))
    );
    assert_eq!(
        test(Partial(&b"ab"[..])),
        Err(ErrMode::Incomplete(Needed::new(2)))
    );
    assert_eq!(
        test(Partial(&b"Hello"[..])),
        Err(ErrMode::Backtrack(error_position!(
            Partial(&b"Hello"[..]),
            ErrorKind::Tag
        )))
    );
    assert_eq!(
        test(Partial(&b"Hel"[..])),
        Err(ErrMode::Backtrack(error_position!(
            Partial(&b"Hel"[..]),
            ErrorKind::Tag
        )))
    );

    fn test2(i: Partial<&str>) -> IResult<Partial<&str>, &str> {
        tag_no_case("ABcd")(i)
    }
    assert_eq!(test2(Partial("aBCdefgh")), Ok((Partial("efgh"), "aBCd")));
    assert_eq!(test2(Partial("abcdefgh")), Ok((Partial("efgh"), "abcd")));
    assert_eq!(test2(Partial("ABCDefgh")), Ok((Partial("efgh"), "ABCD")));
    assert_eq!(
        test2(Partial("ab")),
        Err(ErrMode::Incomplete(Needed::new(2)))
    );
    assert_eq!(
        test2(Partial("Hello")),
        Err(ErrMode::Backtrack(error_position!(
            Partial("Hello"),
            ErrorKind::Tag
        )))
    );
    assert_eq!(
        test2(Partial("Hel")),
        Err(ErrMode::Backtrack(error_position!(
            Partial("Hel"),
            ErrorKind::Tag
        )))
    );
}

#[test]
fn partial_tag_fixed_size_array() {
    fn test(i: Partial<&[u8]>) -> IResult<Partial<&[u8]>, &[u8]> {
        tag([0x42])(i)
    }
    fn test2(i: Partial<&[u8]>) -> IResult<Partial<&[u8]>, &[u8]> {
        tag(&[0x42])(i)
    }
    let input = Partial(&[0x42, 0x00][..]);
    assert_eq!(test(input), Ok((Partial(&b"\x00"[..]), &b"\x42"[..])));
    assert_eq!(test2(input), Ok((Partial(&b"\x00"[..]), &b"\x42"[..])));
}
