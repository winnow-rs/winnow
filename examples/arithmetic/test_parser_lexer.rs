use snapbox::assert_data_eq;
use snapbox::prelude::*;
use snapbox::str;
use winnow::prelude::*;

use crate::parser_lexer::*;

#[test]
fn lex_test() {
    let input = "3";
    let expected = str![[r#"
Ok(
    (
        "",
        [
            Token {
                kind: Value,
                raw: "3",
            },
        ],
    ),
)

"#]];
    assert_data_eq!(tokens.parse_peek(input).to_debug(), expected);

    let input = "  24     ";
    let expected = str![[r#"
Ok(
    (
        "",
        [
            Token {
                kind: Value,
                raw: "24",
            },
        ],
    ),
)

"#]];
    assert_data_eq!(tokens.parse_peek(input).to_debug(), expected);

    let input = " 12 *2 /  3";
    let expected = str![[r#"
Ok(
    (
        "",
        [
            Token {
                kind: Value,
                raw: "12",
            },
            Token {
                kind: Oper(
                    Mul,
                ),
                raw: "*",
            },
            Token {
                kind: Value,
                raw: "2",
            },
            Token {
                kind: Oper(
                    Div,
                ),
                raw: "/",
            },
            Token {
                kind: Value,
                raw: "3",
            },
        ],
    ),
)

"#]];
    assert_data_eq!(tokens.parse_peek(input).to_debug(), expected);

    let input = "  2*2 / ( 5 - 1) + 3";
    let expected = str![[r#"
Ok(
    (
        "",
        [
            Token {
                kind: Value,
                raw: "2",
            },
            Token {
                kind: Oper(
                    Mul,
                ),
                raw: "*",
            },
            Token {
                kind: Value,
                raw: "2",
            },
            Token {
                kind: Oper(
                    Div,
                ),
                raw: "/",
            },
            Token {
                kind: OpenParen,
                raw: "(",
            },
            Token {
                kind: Value,
                raw: "5",
            },
            Token {
                kind: Oper(
                    Sub,
                ),
                raw: "-",
            },
            Token {
                kind: Value,
                raw: "1",
            },
            Token {
                kind: CloseParen,
                raw: ")",
            },
            Token {
                kind: Oper(
                    Add,
                ),
                raw: "+",
            },
            Token {
                kind: Value,
                raw: "3",
            },
        ],
    ),
)

"#]];
    assert_data_eq!(tokens.parse_peek(input).to_debug(), expected);
}

#[test]
fn factor_test() {
    let input = "3";
    let expected = str![[r#"
Ok(
    (
        TokenSlice {
            initial: [
                Token {
                    kind: Value,
                    raw: "3",
                },
            ],
            input: [],
        },
        Value(
            3,
        ),
    ),
)

"#]];
    let input = tokens.parse(input).unwrap();
    let input = Tokens::new(&input);
    assert_data_eq!(factor.parse_peek(input).to_debug(), expected);

    let input = " 12";
    let expected = str![[r#"
Ok(
    (
        TokenSlice {
            initial: [
                Token {
                    kind: Value,
                    raw: "12",
                },
            ],
            input: [],
        },
        Value(
            12,
        ),
    ),
)

"#]];
    let input = tokens.parse(input).unwrap();
    let input = Tokens::new(&input);
    assert_data_eq!(factor.parse_peek(input).to_debug(), expected);

    let input = "537 ";
    let expected = str![[r#"
Ok(
    (
        TokenSlice {
            initial: [
                Token {
                    kind: Value,
                    raw: "537",
                },
            ],
            input: [],
        },
        Value(
            537,
        ),
    ),
)

"#]];
    let input = tokens.parse(input).unwrap();
    let input = Tokens::new(&input);
    assert_data_eq!(factor.parse_peek(input).to_debug(), expected);

    let input = "  24     ";
    let expected = str![[r#"
Ok(
    (
        TokenSlice {
            initial: [
                Token {
                    kind: Value,
                    raw: "24",
                },
            ],
            input: [],
        },
        Value(
            24,
        ),
    ),
)

"#]];
    let input = tokens.parse(input).unwrap();
    let input = Tokens::new(&input);
    assert_data_eq!(factor.parse_peek(input).to_debug(), expected);
}

#[test]
fn term_test() {
    let input = " 12 *2 /  3";
    let expected = str![[r#"
Ok(
    (
        TokenSlice {
            initial: [
                Token {
                    kind: Value,
                    raw: "12",
                },
                Token {
                    kind: Oper(
                        Mul,
                    ),
                    raw: "*",
                },
                Token {
                    kind: Value,
                    raw: "2",
                },
                Token {
                    kind: Oper(
                        Div,
                    ),
                    raw: "/",
                },
                Token {
                    kind: Value,
                    raw: "3",
                },
            ],
            input: [],
        },
        Div(
            Mul(
                Value(
                    12,
                ),
                Value(
                    2,
                ),
            ),
            Value(
                3,
            ),
        ),
    ),
)

"#]];
    let input = tokens.parse(input).unwrap();
    let input = Tokens::new(&input);
    assert_data_eq!(term.parse_peek(input).to_debug(), expected);

    let input = " 12 *2 /  3";
    let expected = str![[r#"
Ok(
    (
        TokenSlice {
            initial: [
                Token {
                    kind: Value,
                    raw: "12",
                },
                Token {
                    kind: Oper(
                        Mul,
                    ),
                    raw: "*",
                },
                Token {
                    kind: Value,
                    raw: "2",
                },
                Token {
                    kind: Oper(
                        Div,
                    ),
                    raw: "/",
                },
                Token {
                    kind: Value,
                    raw: "3",
                },
            ],
            input: [],
        },
        Div(
            Mul(
                Value(
                    12,
                ),
                Value(
                    2,
                ),
            ),
            Value(
                3,
            ),
        ),
    ),
)

"#]];
    let input = tokens.parse(input).unwrap();
    let input = Tokens::new(&input);
    assert_data_eq!(term.parse_peek(input).to_debug(), expected);

    let input = " 2* 3  *2 *2 /  3";
    let expected = str![[r#"
Ok(
    (
        TokenSlice {
            initial: [
                Token {
                    kind: Value,
                    raw: "2",
                },
                Token {
                    kind: Oper(
                        Mul,
                    ),
                    raw: "*",
                },
                Token {
                    kind: Value,
                    raw: "3",
                },
                Token {
                    kind: Oper(
                        Mul,
                    ),
                    raw: "*",
                },
                Token {
                    kind: Value,
                    raw: "2",
                },
                Token {
                    kind: Oper(
                        Mul,
                    ),
                    raw: "*",
                },
                Token {
                    kind: Value,
                    raw: "2",
                },
                Token {
                    kind: Oper(
                        Div,
                    ),
                    raw: "/",
                },
                Token {
                    kind: Value,
                    raw: "3",
                },
            ],
            input: [],
        },
        Div(
            Mul(
                Mul(
                    Mul(
                        Value(
                            2,
                        ),
                        Value(
                            3,
                        ),
                    ),
                    Value(
                        2,
                    ),
                ),
                Value(
                    2,
                ),
            ),
            Value(
                3,
            ),
        ),
    ),
)

"#]];
    let input = tokens.parse(input).unwrap();
    let input = Tokens::new(&input);
    assert_data_eq!(term.parse_peek(input).to_debug(), expected);

    let input = " 48 /  3/2";
    let expected = str![[r#"
Ok(
    (
        TokenSlice {
            initial: [
                Token {
                    kind: Value,
                    raw: "48",
                },
                Token {
                    kind: Oper(
                        Div,
                    ),
                    raw: "/",
                },
                Token {
                    kind: Value,
                    raw: "3",
                },
                Token {
                    kind: Oper(
                        Div,
                    ),
                    raw: "/",
                },
                Token {
                    kind: Value,
                    raw: "2",
                },
            ],
            input: [],
        },
        Div(
            Div(
                Value(
                    48,
                ),
                Value(
                    3,
                ),
            ),
            Value(
                2,
            ),
        ),
    ),
)

"#]];
    let input = tokens.parse(input).unwrap();
    let input = Tokens::new(&input);
    assert_data_eq!(term.parse_peek(input).to_debug(), expected);
}

#[test]
fn expr_test() {
    let input = " 1 +  2 ";
    let expected = str![[r#"
Ok(
    Add(
        Value(
            1,
        ),
        Value(
            2,
        ),
    ),
)

"#]];
    let input = tokens.parse(input).unwrap();
    let input = Tokens::new(&input);
    assert_data_eq!(expr.parse(input).to_debug(), expected);

    let input = " 12 + 6 - 4+  3";
    let expected = str![[r#"
Ok(
    Add(
        Sub(
            Add(
                Value(
                    12,
                ),
                Value(
                    6,
                ),
            ),
            Value(
                4,
            ),
        ),
        Value(
            3,
        ),
    ),
)

"#]];
    let input = tokens.parse(input).unwrap();
    let input = Tokens::new(&input);
    assert_data_eq!(expr.parse(input).to_debug(), expected);

    let input = " 1 + 2*3 + 4";
    let expected = str![[r#"
Ok(
    Add(
        Add(
            Value(
                1,
            ),
            Mul(
                Value(
                    2,
                ),
                Value(
                    3,
                ),
            ),
        ),
        Value(
            4,
        ),
    ),
)

"#]];
    let input = tokens.parse(input).unwrap();
    let input = Tokens::new(&input);
    assert_data_eq!(expr.parse(input).to_debug(), expected);
}

#[test]
fn parens_test() {
    let input = " (  2 )";
    let expected = str![[r#"
Ok(
    Paren(
        Value(
            2,
        ),
    ),
)

"#]];
    let input = tokens.parse(input).unwrap();
    let input = Tokens::new(&input);
    assert_data_eq!(expr.parse(input).to_debug(), expected);

    let input = " 2* (  3 + 4 ) ";
    let expected = str![[r#"
Ok(
    Mul(
        Value(
            2,
        ),
        Paren(
            Add(
                Value(
                    3,
                ),
                Value(
                    4,
                ),
            ),
        ),
    ),
)

"#]];
    let input = tokens.parse(input).unwrap();
    let input = Tokens::new(&input);
    assert_data_eq!(expr.parse(input).to_debug(), expected);

    let input = "  2*2 / ( 5 - 1) + 3";
    let expected = str![[r#"
Ok(
    Add(
        Div(
            Mul(
                Value(
                    2,
                ),
                Value(
                    2,
                ),
            ),
            Paren(
                Sub(
                    Value(
                        5,
                    ),
                    Value(
                        1,
                    ),
                ),
            ),
        ),
        Value(
            3,
        ),
    ),
)

"#]];
    let input = tokens.parse(input).unwrap();
    let input = Tokens::new(&input);
    assert_data_eq!(expr.parse(input).to_debug(), expected);
}
