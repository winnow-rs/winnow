use winnow::prelude::*;

use crate::parser_ast::*;

#[test]
fn factor_test() {
    let input = "3";
    let expected = Ok(("", String::from("Value(3)")));
    assert_eq!(factor.map(|e| format!("{e:?}")).parse_peek(input), expected);

    let input = " 12";
    let expected = Ok(("", String::from("Value(12)")));
    assert_eq!(factor.map(|e| format!("{e:?}")).parse_peek(input), expected);

    let input = "537 ";
    let expected = Ok(("", String::from("Value(537)")));
    assert_eq!(factor.map(|e| format!("{e:?}")).parse_peek(input), expected);

    let input = "  24     ";
    let expected = Ok(("", String::from("Value(24)")));
    assert_eq!(factor.map(|e| format!("{e:?}")).parse_peek(input), expected);
}

#[test]
fn term_test() {
    let input = " 12 *2 /  3";
    let expected = Ok(("", String::from("Div(Mul(Value(12), Value(2)), Value(3))")));
    assert_eq!(term.map(|e| format!("{e:?}")).parse_peek(input), expected);

    let input = " 12 *2 /  3";
    let expected = Ok(("", String::from("Div(Mul(Value(12), Value(2)), Value(3))")));
    assert_eq!(term.map(|e| format!("{e:?}")).parse_peek(input), expected);

    let input = " 2* 3  *2 *2 /  3";
    let expected = Ok((
        "",
        String::from("Div(Mul(Mul(Mul(Value(2), Value(3)), Value(2)), Value(2)), Value(3))"),
    ));
    assert_eq!(term.map(|e| format!("{e:?}")).parse_peek(input), expected);

    let input = " 48 /  3/2";
    let expected = Ok(("", String::from("Div(Div(Value(48), Value(3)), Value(2))")));
    assert_eq!(term.map(|e| format!("{e:?}")).parse_peek(input), expected);
}

#[test]
fn expr_test() {
    let input = " 1 +  2 ";
    let expected = Ok(("", String::from("Add(Value(1), Value(2))")));
    assert_eq!(expr.map(|e| format!("{e:?}")).parse_peek(input), expected);

    let input = " 12 + 6 - 4+  3";
    let expected = Ok((
        "",
        String::from("Add(Sub(Add(Value(12), Value(6)), Value(4)), Value(3))"),
    ));
    assert_eq!(expr.map(|e| format!("{e:?}")).parse_peek(input), expected);

    let input = " 1 + 2*3 + 4";
    let expected = Ok((
        "",
        String::from("Add(Add(Value(1), Mul(Value(2), Value(3))), Value(4))"),
    ));
    assert_eq!(expr.map(|e| format!("{e:?}")).parse_peek(input), expected);
}

#[test]
fn parens_test() {
    let input = " (  2 )";
    let expected = Ok(("", String::from("Paren(Value(2))")));
    assert_eq!(expr.map(|e| format!("{e:?}")).parse_peek(input), expected);

    let input = " 2* (  3 + 4 ) ";
    let expected = Ok((
        "",
        String::from("Mul(Value(2), Paren(Add(Value(3), Value(4))))"),
    ));
    assert_eq!(expr.map(|e| format!("{e:?}")).parse_peek(input), expected);

    let input = "  2*2 / ( 5 - 1) + 3";
    let expected = Ok((
        "",
        String::from("Add(Div(Mul(Value(2), Value(2)), Paren(Sub(Value(5), Value(1)))), Value(3))"),
    ));
    assert_eq!(expr.map(|e| format!("{e:?}")).parse_peek(input), expected);
}
