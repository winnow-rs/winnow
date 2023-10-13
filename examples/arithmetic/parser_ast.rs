use std::fmt;
use std::fmt::{Debug, Display, Formatter};

use std::str::FromStr;

use winnow::prelude::*;
use winnow::{
    ascii::{digit1 as digits, multispace0 as multispaces},
    combinator::alt,
    combinator::delimited,
    combinator::fold_repeat,
    token::one_of,
};

#[derive(Debug, Clone)]
pub enum Expr {
    Value(i64),
    Add(Box<Expr>, Box<Expr>),
    Sub(Box<Expr>, Box<Expr>),
    Mul(Box<Expr>, Box<Expr>),
    Div(Box<Expr>, Box<Expr>),
    Paren(Box<Expr>),
}

impl Expr {
    pub fn eval(&self) -> i64 {
        match self {
            Self::Value(v) => *v,
            Self::Add(lhs, rhs) => lhs.eval() + rhs.eval(),
            Self::Sub(lhs, rhs) => lhs.eval() - rhs.eval(),
            Self::Mul(lhs, rhs) => lhs.eval() * rhs.eval(),
            Self::Div(lhs, rhs) => lhs.eval() / rhs.eval(),
            Self::Paren(expr) => expr.eval(),
        }
    }
}

impl Display for Expr {
    fn fmt(&self, format: &mut Formatter<'_>) -> fmt::Result {
        use Expr::{Add, Div, Mul, Paren, Sub, Value};
        match *self {
            Value(val) => write!(format, "{}", val),
            Add(ref left, ref right) => write!(format, "{} + {}", left, right),
            Sub(ref left, ref right) => write!(format, "{} - {}", left, right),
            Mul(ref left, ref right) => write!(format, "{} * {}", left, right),
            Div(ref left, ref right) => write!(format, "{} / {}", left, right),
            Paren(ref expr) => write!(format, "({})", expr),
        }
    }
}

pub fn expr(i: &mut &str) -> PResult<Expr> {
    let init = term.parse_next(i)?;

    fold_repeat(
        0..,
        (one_of(['+', '-']), term),
        move || init.clone(),
        |acc, (op, val): (char, Expr)| {
            if op == '+' {
                Expr::Add(Box::new(acc), Box::new(val))
            } else {
                Expr::Sub(Box::new(acc), Box::new(val))
            }
        },
    )
    .parse_next(i)
}

fn term(i: &mut &str) -> PResult<Expr> {
    let init = factor.parse_next(i)?;

    fold_repeat(
        0..,
        (one_of(['*', '/']), factor),
        move || init.clone(),
        |acc, (op, val): (char, Expr)| {
            if op == '*' {
                Expr::Mul(Box::new(acc), Box::new(val))
            } else {
                Expr::Div(Box::new(acc), Box::new(val))
            }
        },
    )
    .parse_next(i)
}

fn factor(i: &mut &str) -> PResult<Expr> {
    delimited(
        multispaces,
        alt((digits.try_map(FromStr::from_str).map(Expr::Value), parens)),
        multispaces,
    )
    .parse_next(i)
}

fn parens(i: &mut &str) -> PResult<Expr> {
    delimited("(", expr, ")")
        .map(|e| Expr::Paren(Box::new(e)))
        .parse_next(i)
}

#[test]
fn factor_test() {
    assert_eq!(
        factor
            .parse_peek("  3  ")
            .map(|(i, x)| (i, format!("{:?}", x))),
        Ok(("", String::from("Value(3)")))
    );
}

#[test]
fn term_test() {
    assert_eq!(
        term.parse_peek(" 3 *  5   ")
            .map(|(i, x)| (i, format!("{:?}", x))),
        Ok(("", String::from("Mul(Value(3), Value(5))")))
    );
}

#[test]
fn expr_test() {
    assert_eq!(
        expr.parse_peek(" 1 + 2 *  3 ")
            .map(|(i, x)| (i, format!("{:?}", x))),
        Ok(("", String::from("Add(Value(1), Mul(Value(2), Value(3)))")))
    );
    assert_eq!(
        expr.parse_peek(" 1 + 2 *  3 / 4 - 5 ")
            .map(|(i, x)| (i, format!("{:?}", x))),
        Ok((
            "",
            String::from("Sub(Add(Value(1), Div(Mul(Value(2), Value(3)), Value(4))), Value(5))")
        ))
    );
    assert_eq!(
        expr.parse_peek(" 72 / 2 / 3 ")
            .map(|(i, x)| (i, format!("{:?}", x))),
        Ok(("", String::from("Div(Div(Value(72), Value(2)), Value(3))")))
    );
}

#[test]
fn parens_test() {
    assert_eq!(
        expr.parse_peek(" ( 1 + 2 ) *  3 ")
            .map(|(i, x)| (i, format!("{:?}", x))),
        Ok((
            "",
            String::from("Mul(Paren(Add(Value(1), Value(2))), Value(3))")
        ))
    );
}
