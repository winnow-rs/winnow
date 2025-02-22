use std::fmt;
use std::fmt::{Debug, Display, Formatter};

use std::str::FromStr;

use winnow::prelude::*;
use winnow::Result;
use winnow::{
    ascii::{digit1 as digits, multispace0 as multispaces},
    combinator::alt,
    combinator::dispatch,
    combinator::fail,
    combinator::peek,
    combinator::repeat,
    combinator::{delimited, preceded, terminated},
    error::ContextError,
    stream::TokenSlice,
    token::any,
    token::literal,
    token::one_of,
};

/// Lex and parse
#[allow(dead_code)]
pub(crate) fn expr2(i: &mut &str) -> Result<Expr> {
    let tokens = tokens.parse_next(i)?;
    let mut tokens = Tokens::new(&tokens);
    expr.parse_next(&mut tokens)
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Token {
    Value(i64),
    Oper(Oper),
    OpenParen,
    CloseParen,
}

impl<'i> Parser<Tokens<'i>, &'i Token, ContextError> for Token {
    fn parse_next(&mut self, input: &mut Tokens<'i>) -> Result<&'i Token> {
        literal(*self).parse_next(input).map(|t| &t[0])
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Oper {
    Add,
    Sub,
    Mul,
    Div,
}

impl winnow::stream::ContainsToken<&'_ Token> for Token {
    #[inline(always)]
    fn contains_token(&self, token: &'_ Token) -> bool {
        self == token
    }
}

impl winnow::stream::ContainsToken<&'_ Token> for &'_ [Token] {
    #[inline]
    fn contains_token(&self, token: &'_ Token) -> bool {
        self.iter().any(|t| t == token)
    }
}

impl<const LEN: usize> winnow::stream::ContainsToken<&'_ Token> for &'_ [Token; LEN] {
    #[inline]
    fn contains_token(&self, token: &'_ Token) -> bool {
        self.iter().any(|t| t == token)
    }
}

impl<const LEN: usize> winnow::stream::ContainsToken<&'_ Token> for [Token; LEN] {
    #[inline]
    fn contains_token(&self, token: &'_ Token) -> bool {
        self.iter().any(|t| t == token)
    }
}

/// Lex tokens
///
/// See [`expr`] to parse the tokens
pub(crate) fn tokens(i: &mut &str) -> Result<Vec<Token>> {
    preceded(multispaces, repeat(1.., terminated(token, multispaces))).parse_next(i)
}

fn token(i: &mut &str) -> Result<Token> {
    dispatch! {peek(any);
        '0'..='9' => digits.try_map(FromStr::from_str).map(Token::Value),
        '(' => '('.value(Token::OpenParen),
        ')' => ')'.value(Token::CloseParen),
        '+' => '+'.value(Token::Oper(Oper::Add)),
        '-' => '-'.value(Token::Oper(Oper::Sub)),
        '*' => '*'.value(Token::Oper(Oper::Mul)),
        '/' => '/'.value(Token::Oper(Oper::Div)),
        _ => fail,
    }
    .parse_next(i)
}

#[derive(Debug, Clone)]
pub(crate) enum Expr {
    Value(i64),
    Add(Box<Expr>, Box<Expr>),
    Sub(Box<Expr>, Box<Expr>),
    Mul(Box<Expr>, Box<Expr>),
    Div(Box<Expr>, Box<Expr>),
    Paren(Box<Expr>),
}

impl Expr {
    pub(crate) fn eval(&self) -> i64 {
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
            Value(val) => write!(format, "{val}"),
            Add(ref left, ref right) => write!(format, "{left} + {right}"),
            Sub(ref left, ref right) => write!(format, "{left} - {right}"),
            Mul(ref left, ref right) => write!(format, "{left} * {right}"),
            Div(ref left, ref right) => write!(format, "{left} / {right}"),
            Paren(ref expr) => write!(format, "({expr})"),
        }
    }
}

pub(crate) type Tokens<'i> = TokenSlice<'i, Token>;

/// Parse the tokens lexed in [`tokens`]
pub(crate) fn expr(i: &mut Tokens<'_>) -> Result<Expr> {
    let init = term.parse_next(i)?;

    repeat(
        0..,
        (
            one_of([Token::Oper(Oper::Add), Token::Oper(Oper::Sub)]),
            term,
        ),
    )
    .fold(
        move || init.clone(),
        |acc, (op, val): (&Token, Expr)| {
            if *op == Token::Oper(Oper::Add) {
                Expr::Add(Box::new(acc), Box::new(val))
            } else {
                Expr::Sub(Box::new(acc), Box::new(val))
            }
        },
    )
    .parse_next(i)
}

pub(crate) fn term(i: &mut Tokens<'_>) -> Result<Expr> {
    let init = factor.parse_next(i)?;

    repeat(
        0..,
        (
            one_of([Token::Oper(Oper::Mul), Token::Oper(Oper::Div)]),
            factor,
        ),
    )
    .fold(
        move || init.clone(),
        |acc, (op, val): (&Token, Expr)| {
            if *op == Token::Oper(Oper::Mul) {
                Expr::Mul(Box::new(acc), Box::new(val))
            } else {
                Expr::Div(Box::new(acc), Box::new(val))
            }
        },
    )
    .parse_next(i)
}

pub(crate) fn factor(i: &mut Tokens<'_>) -> Result<Expr> {
    alt((
        one_of(|t: &_| matches!(t, Token::Value(_))).map(|t: &_| match t {
            Token::Value(v) => Expr::Value(*v),
            _ => unreachable!(),
        }),
        parens,
    ))
    .parse_next(i)
}

fn parens(i: &mut Tokens<'_>) -> Result<Expr> {
    delimited(Token::OpenParen, expr, Token::CloseParen)
        .map(|e| Expr::Paren(Box::new(e)))
        .parse_next(i)
}
