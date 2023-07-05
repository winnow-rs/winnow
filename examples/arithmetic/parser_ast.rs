use std::fmt;
use std::fmt::{Debug, Display, Formatter};

use std::str::FromStr;

use winnow::prelude::*;
use winnow::{
    ascii::{digit1 as digit, multispace0 as multispace},
    combinator::alt,
    combinator::repeat,
    combinator::{delimited, preceded},
};

#[derive(Debug)]
pub enum Expr {
    Value(i64),
    Add(Box<Expr>, Box<Expr>),
    Sub(Box<Expr>, Box<Expr>),
    Mul(Box<Expr>, Box<Expr>),
    Div(Box<Expr>, Box<Expr>),
    Paren(Box<Expr>),
}

#[derive(Debug)]
pub enum Oper {
    Add,
    Sub,
    Mul,
    Div,
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
    let initial = term(i)?;
    let remainder = repeat(
        0..,
        alt((
            |i: &mut &str| {
                let add = preceded("+", term).parse_next(i)?;
                Ok((Oper::Add, add))
            },
            |i: &mut &str| {
                let sub = preceded("-", term).parse_next(i)?;
                Ok((Oper::Sub, sub))
            },
        )),
    )
    .parse_next(i)?;

    Ok(fold_exprs(initial, remainder))
}

fn term(i: &mut &str) -> PResult<Expr> {
    let initial = factor(i)?;
    let remainder = repeat(
        0..,
        alt((
            |i: &mut &str| {
                let mul = preceded("*", factor).parse_next(i)?;
                Ok((Oper::Mul, mul))
            },
            |i: &mut &str| {
                let div = preceded("/", factor).parse_next(i)?;
                Ok((Oper::Div, div))
            },
        )),
    )
    .parse_next(i)?;

    Ok(fold_exprs(initial, remainder))
}

fn factor(i: &mut &str) -> PResult<Expr> {
    alt((
        delimited(multispace, digit, multispace)
            .try_map(FromStr::from_str)
            .map(Expr::Value),
        parens,
    ))
    .parse_next(i)
}

fn parens(i: &mut &str) -> PResult<Expr> {
    delimited(
        multispace,
        delimited("(", expr.map(|e| Expr::Paren(Box::new(e))), ")"),
        multispace,
    )
    .parse_next(i)
}

fn fold_exprs(initial: Expr, remainder: Vec<(Oper, Expr)>) -> Expr {
    remainder.into_iter().fold(initial, |acc, pair| {
        let (oper, expr) = pair;
        match oper {
            Oper::Add => Expr::Add(Box::new(acc), Box::new(expr)),
            Oper::Sub => Expr::Sub(Box::new(acc), Box::new(expr)),
            Oper::Mul => Expr::Mul(Box::new(acc), Box::new(expr)),
            Oper::Div => Expr::Div(Box::new(acc), Box::new(expr)),
        }
    })
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
