use winnow::prelude::*;

use crate::parser::*;

#[test]
fn factor_test() {
    let input = "3";
    let expected = Ok(("", 3));
    assert_eq!(factor.parse_peek(input), expected);

    let input = " 12";
    let expected = Ok(("", 12));
    assert_eq!(factor.parse_peek(input), expected);

    let input = "537 ";
    let expected = Ok(("", 537));
    assert_eq!(factor.parse_peek(input), expected);

    let input = "  24     ";
    let expected = Ok(("", 24));
    assert_eq!(factor.parse_peek(input), expected);
}

#[test]
fn term_test() {
    let input = " 12 *2 /  3";
    let expected = Ok(("", 8));
    assert_eq!(term.parse_peek(input), expected);

    let input = " 12 *2 /  3";
    let expected = Ok(("", 8));
    assert_eq!(term.parse_peek(input), expected);

    let input = " 2* 3  *2 *2 /  3";
    let expected = Ok(("", 8));
    assert_eq!(term.parse_peek(input), expected);

    let input = " 48 /  3/2";
    let expected = Ok(("", 8));
    assert_eq!(term.parse_peek(input), expected);
}

#[test]
fn expr_test() {
    let input = " 1 +  2 ";
    let expected = Ok(("", 3));
    assert_eq!(expr.parse_peek(input), expected);

    let input = " 12 + 6 - 4+  3";
    let expected = Ok(("", 17));
    assert_eq!(expr.parse_peek(input), expected);

    let input = " 1 + 2*3 + 4";
    let expected = Ok(("", 11));
    assert_eq!(expr.parse_peek(input), expected);
}

#[test]
fn parens_test() {
    let input = " (  2 )";
    let expected = Ok(("", 2));
    assert_eq!(expr.parse_peek(input), expected);

    let input = " 2* (  3 + 4 ) ";
    let expected = Ok(("", 14));
    assert_eq!(expr.parse_peek(input), expected);

    let input = "  2*2 / ( 5 - 1) + 3";
    let expected = Ok(("", 4));
    assert_eq!(expr.parse_peek(input), expected);
}
