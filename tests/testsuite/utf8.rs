#[cfg(test)]
mod test {
    use winnow::Parser;
    use winnow::Partial;
    #[cfg(feature = "alloc")]
    use winnow::{combinator::alt, combinator::repeat, token::tag_no_case};
    use winnow::{
        error::ErrMode,
        error::{self, Error, ErrorKind},
        token::{take, take_till0, take_till1, take_until0, take_while1},
        IResult,
    };

    #[test]
    fn tag_succeed_str() {
        const INPUT: &str = "Hello World!";
        fn test(input: &str) -> IResult<&str, &str> {
            "Hello".parse_next(input)
        }

        match test(INPUT) {
            Ok((extra, output)) => {
                assert!(extra == " World!", "Parser `tag` consumed leftover input.");
                assert!(
                    output == "Hello",
                    "Parser `tag` doesn't return the tag it matched on success. \
           Expected `{}`, got `{}`.",
                    "Hello",
                    output
                );
            }
            other => panic!(
                "Parser `tag` didn't succeed when it should have. \
         Got `{:?}`.",
                other
            ),
        };
    }

    #[test]
    fn tag_incomplete_str() {
        const INPUT: &str = "Hello";

        let res: IResult<_, _, error::Error<_>> = "Hello World!".parse_next(Partial::new(INPUT));
        match res {
            Err(ErrMode::Incomplete(_)) => (),
            other => {
                panic!(
                    "Parser `tag` didn't require more input when it should have. \
           Got `{:?}`.",
                    other
                );
            }
        };
    }

    #[test]
    fn tag_error_str() {
        const INPUT: &str = "Hello World!";

        let res: IResult<_, _, error::Error<_>> = "Random".parse_next(INPUT);
        match res {
            Err(ErrMode::Backtrack(_)) => (),
            other => {
                panic!(
                    "Parser `tag` didn't fail when it should have. Got `{:?}`.`",
                    other
                );
            }
        };
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn tag_case_insensitive_str() {
        fn test(i: &str) -> IResult<&str, &str> {
            tag_no_case("ABcd").parse_next(i)
        }
        assert_eq!(test("aBCdefgh"), Ok(("efgh", "aBCd")));
        assert_eq!(test("abcdefgh"), Ok(("efgh", "abcd")));
        assert_eq!(test("ABCDefgh"), Ok(("efgh", "ABCD")));
    }

    #[test]
    fn take_succeed_str() {
        const INPUT: &str = "βèƒôřèÂßÇáƒƭèř";
        const CONSUMED: &str = "βèƒôřèÂßÇ";
        const LEFTOVER: &str = "áƒƭèř";

        let res: IResult<_, _, error::Error<_>> = take(9_usize).parse_next(INPUT);
        match res {
            Ok((extra, output)) => {
                assert!(
                    extra == LEFTOVER,
                    "Parser `take_s` consumed leftover input. Leftover `{}`.",
                    extra
                );
                assert!(
          output == CONSUMED,
          "Parser `take_s` doesn't return the string it consumed on success. Expected `{}`, got `{}`.",
          CONSUMED,
          output
        );
            }
            other => panic!(
                "Parser `take_s` didn't succeed when it should have. \
         Got `{:?}`.",
                other
            ),
        };
    }

    #[test]
    fn take_incomplete_str() {
        use winnow::token::take;

        const INPUT: &str = "βèƒôřèÂßÇá";

        let res: IResult<_, _, Error<_>> = take(13_usize).parse_next(Partial::new(INPUT));
        match res {
            Err(ErrMode::Incomplete(_)) => (),
            other => panic!(
                "Parser `take` didn't require more input when it should have. \
         Got `{:?}`.",
                other
            ),
        }
    }

    #[test]
    fn take_until_succeed_str() {
        const INPUT: &str = "βèƒôřèÂßÇ∂áƒƭèř";
        const FIND: &str = "ÂßÇ∂";
        const CONSUMED: &str = "βèƒôřè";
        const LEFTOVER: &str = "ÂßÇ∂áƒƭèř";

        let res: IResult<_, _, Error<_>> = take_until0(FIND).parse_next(INPUT);
        match res {
            Ok((extra, output)) => {
                assert!(
                    extra == LEFTOVER,
                    "Parser `take_until0`\
           consumed leftover input. Leftover `{}`.",
                    extra
                );
                assert!(
                    output == CONSUMED,
                    "Parser `take_until0`\
           doesn't return the string it consumed on success. Expected `{}`, got `{}`.",
                    CONSUMED,
                    output
                );
            }
            other => panic!(
                "Parser `take_until0` didn't succeed when it should have. \
         Got `{:?}`.",
                other
            ),
        };
    }

    #[test]
    fn take_until_incomplete_str() {
        use winnow::token::take_until0;

        const INPUT: &str = "βèƒôřè";
        const FIND: &str = "βèƒôřèÂßÇ";

        let res: IResult<_, _, Error<_>> = take_until0(FIND).parse_next(Partial::new(INPUT));
        match res {
            Err(ErrMode::Incomplete(_)) => (),
            other => panic!(
                "Parser `take_until0` didn't require more input when it should have. \
         Got `{:?}`.",
                other
            ),
        };
    }

    #[test]
    fn take_until_error_str() {
        use winnow::token::take_until0;

        const INPUT: &str = "βèƒôřèÂßÇáƒƭèř";
        const FIND: &str = "Ráñδô₥";

        let res: IResult<_, _, Error<_>> = take_until0(FIND).parse_next(Partial::new(INPUT));
        match res {
            Err(ErrMode::Incomplete(_)) => (),
            other => panic!(
                "Parser `take_until0` didn't fail when it should have. \
         Got `{:?}`.",
                other
            ),
        };
    }

    fn is_alphabetic(c: char) -> bool {
        (c as u8 >= 0x41 && c as u8 <= 0x5A) || (c as u8 >= 0x61 && c as u8 <= 0x7A)
    }

    #[test]
    fn take_while_str() {
        use winnow::error::Needed;

        use winnow::token::take_while0;

        fn f(i: Partial<&str>) -> IResult<Partial<&str>, &str> {
            take_while0(is_alphabetic).parse_next(i)
        }
        let a = "";
        let b = "abcd";
        let c = "abcd123";
        let d = "123";

        assert_eq!(f(Partial::new(a)), Err(ErrMode::Incomplete(Needed::new(1))));
        assert_eq!(f(Partial::new(b)), Err(ErrMode::Incomplete(Needed::new(1))));
        assert_eq!(f(Partial::new(c)), Ok((Partial::new(d), b)));
        assert_eq!(f(Partial::new(d)), Ok((Partial::new(d), a)));
    }

    #[test]
    fn take_while_succeed_none_str() {
        use winnow::token::take_while0;

        const INPUT: &str = "βèƒôřèÂßÇáƒƭèř";
        const CONSUMED: &str = "";
        const LEFTOVER: &str = "βèƒôřèÂßÇáƒƭèř";
        fn while_s(c: char) -> bool {
            c == '9'
        }
        fn test(input: &str) -> IResult<&str, &str> {
            take_while0(while_s).parse_next(input)
        }
        match test(INPUT) {
            Ok((extra, output)) => {
                assert!(
                    extra == LEFTOVER,
                    "Parser `take_while0` consumed leftover input."
                );
                assert!(
                    output == CONSUMED,
                    "Parser `take_while0` doesn't return the string it consumed on success. \
           Expected `{}`, got `{}`.",
                    CONSUMED,
                    output
                );
            }
            other => panic!(
                "Parser `take_while0` didn't succeed when it should have. \
         Got `{:?}`.",
                other
            ),
        };
    }

    #[test]
    fn take_while_succeed_some_str() {
        use winnow::token::take_while0;

        const INPUT: &str = "βèƒôřèÂßÇáƒƭèř";
        const CONSUMED: &str = "βèƒôřèÂßÇ";
        const LEFTOVER: &str = "áƒƭèř";
        fn while_s(c: char) -> bool {
            matches!(c, 'β' | 'è' | 'ƒ' | 'ô' | 'ř' | 'Â' | 'ß' | 'Ç')
        }
        fn test(input: &str) -> IResult<&str, &str> {
            take_while0(while_s).parse_next(input)
        }
        match test(INPUT) {
            Ok((extra, output)) => {
                assert!(
                    extra == LEFTOVER,
                    "Parser `take_while0` consumed leftover input."
                );
                assert!(
                    output == CONSUMED,
                    "Parser `take_while0` doesn't return the string it consumed on success. \
           Expected `{}`, got `{}`.",
                    CONSUMED,
                    output
                );
            }
            other => panic!(
                "Parser `take_while0` didn't succeed when it should have. \
         Got `{:?}`.",
                other
            ),
        };
    }

    #[test]
    fn test_take_while1_str() {
        use winnow::error::Needed;

        fn f(i: Partial<&str>) -> IResult<Partial<&str>, &str> {
            take_while1(is_alphabetic).parse_next(i)
        }
        let a = "";
        let b = "abcd";
        let c = "abcd123";
        let d = "123";

        assert_eq!(f(Partial::new(a)), Err(ErrMode::Incomplete(Needed::new(1))));
        assert_eq!(f(Partial::new(b)), Err(ErrMode::Incomplete(Needed::new(1))));
        assert_eq!(f(Partial::new(c)), Ok((Partial::new("123"), b)));
        assert_eq!(
            f(Partial::new(d)),
            Err(ErrMode::Backtrack(winnow::error::Error {
                input: Partial::new(d),
                kind: ErrorKind::Slice
            }))
        );
    }

    #[test]
    fn take_while1_fn_succeed_str() {
        use winnow::token::take_while1;

        const INPUT: &str = "βèƒôřèÂßÇáƒƭèř";
        const CONSUMED: &str = "βèƒôřèÂßÇ";
        const LEFTOVER: &str = "áƒƭèř";
        fn while1_s(c: char) -> bool {
            matches!(c, 'β' | 'è' | 'ƒ' | 'ô' | 'ř' | 'Â' | 'ß' | 'Ç')
        }
        fn test(input: &str) -> IResult<&str, &str> {
            take_while1(while1_s).parse_next(input)
        }
        match test(INPUT) {
            Ok((extra, output)) => {
                assert!(
                    extra == LEFTOVER,
                    "Parser `take_while1` consumed leftover input."
                );
                assert!(
                    output == CONSUMED,
                    "Parser `take_while1` doesn't return the string it consumed on success. \
           Expected `{}`, got `{}`.",
                    CONSUMED,
                    output
                );
            }
            other => panic!(
                "Parser `take_while1` didn't succeed when it should have. \
         Got `{:?}`.",
                other
            ),
        };
    }

    #[test]
    fn take_while1_set_succeed_str() {
        const INPUT: &str = "βèƒôřèÂßÇáƒƭèř";
        const MATCH: &str = "βèƒôřèÂßÇ";
        const CONSUMED: &str = "βèƒôřèÂßÇ";
        const LEFTOVER: &str = "áƒƭèř";
        fn test(input: &str) -> IResult<&str, &str> {
            take_while1(MATCH).parse_next(input)
        }
        match test(INPUT) {
            Ok((extra, output)) => {
                assert!(
                    extra == LEFTOVER,
                    "Parser `is_a` consumed leftover input. Leftover `{}`.",
                    extra
                );
                assert!(
          output == CONSUMED,
          "Parser `is_a` doesn't return the string it consumed on success. Expected `{}`, got `{}`.",
          CONSUMED,
          output
        );
            }
            other => panic!(
                "Parser `is_a` didn't succeed when it should have. \
         Got `{:?}`.",
                other
            ),
        };
    }

    #[test]
    fn take_while1_fn_fail_str() {
        use winnow::token::take_while1;

        const INPUT: &str = "βèƒôřèÂßÇáƒƭèř";
        fn while1_s(c: char) -> bool {
            c == '9'
        }
        fn test(input: &str) -> IResult<&str, &str> {
            take_while1(while1_s).parse_next(input)
        }
        match test(INPUT) {
            Err(ErrMode::Backtrack(_)) => (),
            other => panic!(
                "Parser `take_while1` didn't fail when it should have. \
         Got `{:?}`.",
                other
            ),
        };
    }

    #[test]
    fn take_while1_set_fail_str() {
        const INPUT: &str = "βèƒôřèÂßÇáƒƭèř";
        const MATCH: &str = "Ûñℓúçƙ¥";
        fn test(input: &str) -> IResult<&str, &str> {
            take_while1(MATCH).parse_next(input)
        }
        match test(INPUT) {
            Err(ErrMode::Backtrack(_)) => (),
            other => panic!(
                "Parser `is_a` didn't fail when it should have. Got `{:?}`.",
                other
            ),
        };
    }

    #[test]
    fn take_till0_succeed_str() {
        const INPUT: &str = "βèƒôřèÂßÇáƒƭèř";
        const CONSUMED: &str = "βèƒôřèÂßÇ";
        const LEFTOVER: &str = "áƒƭèř";
        fn till_s(c: char) -> bool {
            c == 'á'
        }
        fn test(input: &str) -> IResult<&str, &str> {
            take_till0(till_s).parse_next(input)
        }
        match test(INPUT) {
            Ok((extra, output)) => {
                assert!(
                    extra == LEFTOVER,
                    "Parser `take_till0` consumed leftover input."
                );
                assert!(
                    output == CONSUMED,
                    "Parser `take_till0` doesn't return the string it consumed on success. \
           Expected `{}`, got `{}`.",
                    CONSUMED,
                    output
                );
            }
            other => panic!(
                "Parser `take_till0` didn't succeed when it should have. \
         Got `{:?}`.",
                other
            ),
        };
    }

    #[test]
    fn take_till1_succeed_str() {
        const INPUT: &str = "βèƒôřèÂßÇáƒƭèř";
        const AVOID: &str = "£úçƙ¥á";
        const CONSUMED: &str = "βèƒôřèÂßÇ";
        const LEFTOVER: &str = "áƒƭèř";
        fn test(input: &str) -> IResult<&str, &str> {
            take_till1(AVOID).parse_next(input)
        }
        match test(INPUT) {
            Ok((extra, output)) => {
                assert!(
                    extra == LEFTOVER,
                    "Parser `take_till1` consumed leftover input. Leftover `{}`.",
                    extra
                );
                assert!(
          output == CONSUMED,
          "Parser `take_till1` doesn't return the string it consumed on success. Expected `{}`, got `{}`.",
          CONSUMED,
          output
        );
            }
            other => panic!(
                "Parser `take_till1` didn't succeed when it should have. \
         Got `{:?}`.",
                other
            ),
        };
    }

    #[test]
    fn take_till1_failed_str() {
        const INPUT: &str = "βèƒôřèÂßÇáƒƭèř";
        const AVOID: &str = "βúçƙ¥";
        fn test(input: &str) -> IResult<&str, &str> {
            take_till1(AVOID).parse_next(input)
        }
        match test(INPUT) {
            Err(ErrMode::Backtrack(_)) => (),
            other => panic!(
                "Parser `is_not` didn't fail when it should have. Got `{:?}`.",
                other
            ),
        };
    }

    #[test]
    #[cfg(feature = "alloc")]
    fn recognize_is_a_str() {
        use winnow::prelude::*;

        let a = "aabbab";
        let b = "ababcd";

        fn f(i: &str) -> IResult<&str, &str> {
            repeat::<_, _, (), _, _>(1.., alt(("a", "b")))
                .recognize()
                .parse_next(i)
        }

        assert_eq!(f(a), Ok((&a[6..], a)));
        assert_eq!(f(b), Ok((&b[4..], &b[..4])));
    }

    #[test]
    fn utf8_indexing_str() {
        fn dot(i: &str) -> IResult<&str, &str> {
            ".".parse_next(i)
        }

        let _ = dot("點");
    }
}
