use snapbox::prelude::*;
use snapbox::str;

use winnow::ascii::Caseless;
use winnow::prelude::*;
use winnow::token::{take, take_till, take_until, take_while};
use winnow::Partial;
#[cfg(feature = "alloc")]
use winnow::{combinator::alt, combinator::repeat, token::literal};

use crate::TestResult;

#[test]
fn literal_succeed_str() {
    const INPUT: &str = "Hello World!";
    fn test<'i>(input: &mut &'i str) -> TestResult<&'i str, &'i str> {
        "Hello".parse_next(input)
    }

    assert_parse!(
        test.parse_peek(INPUT),
        str![[r#"
Ok(
    (
        " World!",
        "Hello",
    ),
)

"#]]
        .raw()
    );
}

#[test]
fn literal_incomplete_str() {
    const INPUT: &str = "Hello";

    assert_parse!(
        "Hello World!".parse_peek(INPUT),
        str![[r#"
Err(
    Backtrack(
        InputError {
            input: "Hello",
            kind: Fail,
        },
    ),
)

"#]]
        .raw()
    );
}

#[test]
fn literal_error_str() {
    const INPUT: &str = "Hello World!";

    assert_parse!(
        "Random".parse_peek(INPUT),
        str![[r#"
Err(
    Backtrack(
        InputError {
            input: "Hello World!",
            kind: Fail,
        },
    ),
)

"#]]
        .raw()
    );
}

#[cfg(feature = "alloc")]
#[test]
fn literal_case_insensitive_str() {
    fn test<'i>(input: &mut &'i str) -> TestResult<&'i str, &'i str> {
        literal(Caseless("ABcd")).parse_next(input)
    }
    assert_parse!(
        test.parse_peek("aBCdefgh"),
        str![[r#"
Ok(
    (
        "efgh",
        "aBCd",
    ),
)

"#]]
        .raw()
    );
    assert_parse!(
        test.parse_peek("abcdefgh"),
        str![[r#"
Ok(
    (
        "efgh",
        "abcd",
    ),
)

"#]]
        .raw()
    );
    assert_parse!(
        test.parse_peek("ABCDefgh"),
        str![[r#"
Ok(
    (
        "efgh",
        "ABCD",
    ),
)

"#]]
        .raw()
    );
}

#[test]
fn take_succeed_str() {
    const INPUT: &str = "βèƒôřèÂßÇáƒƭèř";

    assert_parse!(
        take(9_usize).parse_peek(INPUT),
        str![[r#"
Ok(
    (
        "áƒƭèř",
        "βèƒôřèÂßÇ",
    ),
)

"#]]
        .raw()
    );
}

#[test]
fn take_incomplete_str() {
    use winnow::token::take;

    const INPUT: &str = "βèƒôřèÂßÇá";

    assert_parse!(
        take(13_usize).parse_peek(Partial::new(INPUT)),
        str![[r#"
Err(
    Incomplete(
        Unknown,
    ),
)

"#]]
        .raw()
    );
}

#[test]
fn take_until_succeed_str() {
    const INPUT: &str = "βèƒôřèÂßÇ∂áƒƭèř";
    const FIND: &str = "ÂßÇ∂";

    assert_parse!(
        take_until(0.., FIND).parse_peek(INPUT),
        str![[r#"
Ok(
    (
        "ÂßÇ∂áƒƭèř",
        "βèƒôřè",
    ),
)

"#]]
        .raw()
    );
}

#[test]
fn take_until_incomplete_str() {
    use winnow::token::take_until;

    const INPUT: &str = "βèƒôřè";
    const FIND: &str = "βèƒôřèÂßÇ";

    assert_parse!(
        take_until(0.., FIND).parse_peek(Partial::new(INPUT)),
        str![[r#"
Err(
    Incomplete(
        Unknown,
    ),
)

"#]]
        .raw()
    );
}

#[test]
fn take_until_error_str() {
    use winnow::token::take_until;

    const INPUT: &str = "βèƒôřèÂßÇáƒƭèř";
    const FIND: &str = "Ráñδô₥";

    assert_parse!(
        take_until(0.., FIND).parse_peek(Partial::new(INPUT)),
        str![[r#"
Err(
    Incomplete(
        Unknown,
    ),
)

"#]]
        .raw()
    );
}

fn is_alphabetic(c: char) -> bool {
    (c as u8 >= 0x41 && c as u8 <= 0x5A) || (c as u8 >= 0x61 && c as u8 <= 0x7A)
}

#[test]
fn take_while_str() {
    use winnow::token::take_while;

    fn f<'i>(input: &mut Partial<&'i str>) -> TestResult<Partial<&'i str>, &'i str> {
        take_while(0.., is_alphabetic).parse_next(input)
    }
    let a = "";
    let b = "abcd";
    let c = "abcd123";
    let d = "123";

    assert_parse!(
        f.parse_peek(Partial::new(a)),
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
        f.parse_peek(Partial::new(b)),
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
        f.parse_peek(Partial::new(c)),
        str![[r#"
Ok(
    (
        Partial {
            input: "123",
            partial: true,
        },
        "abcd",
    ),
)

"#]]
        .raw()
    );
    assert_parse!(
        f.parse_peek(Partial::new(d)),
        str![[r#"
Ok(
    (
        Partial {
            input: "123",
            partial: true,
        },
        "",
    ),
)

"#]]
        .raw()
    );
}

#[test]
fn take_while_succeed_none_str() {
    use winnow::token::take_while;

    const INPUT: &str = "βèƒôřèÂßÇáƒƭèř";
    fn while_s(c: char) -> bool {
        c == '9'
    }
    fn test<'i>(input: &mut &'i str) -> TestResult<&'i str, &'i str> {
        take_while(0.., while_s).parse_next(input)
    }
    assert_parse!(
        test.parse_peek(INPUT),
        str![[r#"
Ok(
    (
        "βèƒôřèÂßÇáƒƭèř",
        "",
    ),
)

"#]]
        .raw()
    );
}

#[test]
fn take_while_succeed_some_str() {
    use winnow::token::take_while;

    const INPUT: &str = "βèƒôřèÂßÇáƒƭèř";
    fn while_s(c: char) -> bool {
        matches!(c, 'β' | 'è' | 'ƒ' | 'ô' | 'ř' | 'Â' | 'ß' | 'Ç')
    }
    fn test<'i>(input: &mut &'i str) -> TestResult<&'i str, &'i str> {
        take_while(0.., while_s).parse_next(input)
    }
    assert_parse!(
        test.parse_peek(INPUT),
        str![[r#"
Ok(
    (
        "áƒƭèř",
        "βèƒôřèÂßÇ",
    ),
)

"#]]
        .raw()
    );
}

#[test]
fn test_take_while1_str() {
    fn f<'i>(input: &mut Partial<&'i str>) -> TestResult<Partial<&'i str>, &'i str> {
        take_while(1.., is_alphabetic).parse_next(input)
    }
    let a = "";
    let b = "abcd";
    let c = "abcd123";
    let d = "123";

    assert_parse!(
        f.parse_peek(Partial::new(a)),
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
        f.parse_peek(Partial::new(b)),
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
        f.parse_peek(Partial::new(c)),
        str![[r#"
Ok(
    (
        Partial {
            input: "123",
            partial: true,
        },
        "abcd",
    ),
)

"#]]
        .raw()
    );
    assert_parse!(
        f.parse_peek(Partial::new(d)),
        str![[r#"
Err(
    Backtrack(
        InputError {
            input: Partial {
                input: "123",
                partial: true,
            },
            kind: Fail,
        },
    ),
)

"#]]
        .raw()
    );
}

#[test]
fn take_while1_fn_succeed_str() {
    use winnow::token::take_while;

    const INPUT: &str = "βèƒôřèÂßÇáƒƭèř";
    fn while1_s(c: char) -> bool {
        matches!(c, 'β' | 'è' | 'ƒ' | 'ô' | 'ř' | 'Â' | 'ß' | 'Ç')
    }
    fn test<'i>(input: &mut &'i str) -> TestResult<&'i str, &'i str> {
        take_while(1.., while1_s).parse_next(input)
    }
    assert_parse!(
        test.parse_peek(INPUT),
        str![[r#"
Ok(
    (
        "áƒƭèř",
        "βèƒôřèÂßÇ",
    ),
)

"#]]
        .raw()
    );
}

#[test]
fn take_while1_set_succeed_str() {
    const INPUT: &str = "βèƒôřèÂßÇáƒƭèř";
    const MATCH: &[char] = &['β', 'è', 'ƒ', 'ô', 'ř', 'è', 'Â', 'ß', 'Ç'];
    fn test<'i>(input: &mut &'i str) -> TestResult<&'i str, &'i str> {
        take_while(1.., MATCH).parse_next(input)
    }
    assert_parse!(
        test.parse_peek(INPUT),
        str![[r#"
Ok(
    (
        "áƒƭèř",
        "βèƒôřèÂßÇ",
    ),
)

"#]]
        .raw()
    );
}

#[test]
fn take_while1_fn_fail_str() {
    use winnow::token::take_while;

    const INPUT: &str = "βèƒôřèÂßÇáƒƭèř";
    fn while1_s(c: char) -> bool {
        c == '9'
    }
    fn test<'i>(input: &mut &'i str) -> TestResult<&'i str, &'i str> {
        take_while(1.., while1_s).parse_next(input)
    }
    assert_parse!(
        test.parse_peek(INPUT),
        str![[r#"
Err(
    Backtrack(
        InputError {
            input: "βèƒôřèÂßÇáƒƭèř",
            kind: Fail,
        },
    ),
)

"#]]
        .raw()
    );
}

#[test]
fn take_while1_set_fail_str() {
    const INPUT: &str = "βèƒôřèÂßÇáƒƭèř";
    const MATCH: &[char] = &['Û', 'ñ', 'ℓ', 'ú', 'ç', 'ƙ', '¥'];
    fn test<'i>(input: &mut &'i str) -> TestResult<&'i str, &'i str> {
        take_while(1.., MATCH).parse_next(input)
    }
    assert_parse!(
        test.parse_peek(INPUT),
        str![[r#"
Err(
    Backtrack(
        InputError {
            input: "βèƒôřèÂßÇáƒƭèř",
            kind: Fail,
        },
    ),
)

"#]]
        .raw()
    );
}

#[test]
fn take_till0_succeed_str() {
    const INPUT: &str = "βèƒôřèÂßÇáƒƭèř";
    fn till_s(c: char) -> bool {
        c == 'á'
    }
    fn test<'i>(input: &mut &'i str) -> TestResult<&'i str, &'i str> {
        take_till(0.., till_s).parse_next(input)
    }
    assert_parse!(
        test.parse_peek(INPUT),
        str![[r#"
Ok(
    (
        "áƒƭèř",
        "βèƒôřèÂßÇ",
    ),
)

"#]]
        .raw()
    );
}

#[test]
fn take_till1_succeed_str() {
    const INPUT: &str = "βèƒôřèÂßÇáƒƭèř";
    const AVOID: &[char] = &['£', 'ú', 'ç', 'ƙ', '¥', 'á'];
    fn test<'i>(input: &mut &'i str) -> TestResult<&'i str, &'i str> {
        take_till(1.., AVOID).parse_next(input)
    }
    assert_parse!(
        test.parse_peek(INPUT),
        str![[r#"
Ok(
    (
        "áƒƭèř",
        "βèƒôřèÂßÇ",
    ),
)

"#]]
        .raw()
    );
}

#[test]
fn take_till1_failed_str() {
    const INPUT: &str = "βèƒôřèÂßÇáƒƭèř";
    const AVOID: &[char] = &['β', 'ú', 'ç', 'ƙ', '¥'];
    fn test<'i>(input: &mut &'i str) -> TestResult<&'i str, &'i str> {
        take_till(1.., AVOID).parse_next(input)
    }
    assert_parse!(
        test.parse_peek(INPUT),
        str![[r#"
Err(
    Backtrack(
        InputError {
            input: "βèƒôřèÂßÇáƒƭèř",
            kind: Fail,
        },
    ),
)

"#]]
        .raw()
    );
}

#[test]
#[cfg(feature = "alloc")]
fn take_is_a_str() {
    use winnow::prelude::*;

    let a = "aabbab";
    let b = "ababcd";

    fn f<'i>(input: &mut &'i str) -> TestResult<&'i str, &'i str> {
        repeat::<_, _, (), _, _>(1.., alt(("a", "b")))
            .take()
            .parse_next(input)
    }

    assert_parse!(
        f.parse_peek(a),
        str![[r#"
Ok(
    (
        "",
        "aabbab",
    ),
)

"#]]
        .raw()
    );
    assert_parse!(
        f.parse_peek(b),
        str![[r#"
Ok(
    (
        "cd",
        "abab",
    ),
)

"#]]
        .raw()
    );
}

#[test]
fn utf8_indexing_str() {
    fn dot<'i>(input: &mut &'i str) -> TestResult<&'i str, &'i str> {
        ".".parse_next(input)
    }

    let _ = dot.parse_peek("點");
}
