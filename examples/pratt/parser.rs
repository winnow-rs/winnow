use winnow::combinator::{cut_err, empty, fail, opt, peek, preceded, separated_pair, trace};
use winnow::prelude::*;
use winnow::stream::AsChar as _;
use winnow::token::{any, take, take_while};
use winnow::{
    ascii::{digit1, multispace0},
    combinator::alt,
    combinator::delimited,
    dispatch,
    token::one_of,
};

pub(crate) enum Expr {
    Name(String),
    Value(i64),

    Assign(Box<Expr>, Box<Expr>),

    Addr(Box<Expr>),
    Deref(Box<Expr>),

    Dot(Box<Expr>, Box<Expr>),
    ArrowOp(Box<Expr>, Box<Expr>),
    Neg(Box<Expr>),
    Add(Box<Expr>, Box<Expr>),
    Sub(Box<Expr>, Box<Expr>),
    Mul(Box<Expr>, Box<Expr>),
    Div(Box<Expr>, Box<Expr>),
    Fac(Box<Expr>),

    PreIncr(Box<Expr>),
    PostIncr(Box<Expr>),
    PreDecr(Box<Expr>),
    PostDecr(Box<Expr>),

    And(Box<Expr>, Box<Expr>),
    Or(Box<Expr>, Box<Expr>),

    // `==`
    Eq(Box<Expr>, Box<Expr>),
    // `!=`
    NotEq(Box<Expr>, Box<Expr>),
    // `!`
    Not(Box<Expr>),
    Greater(Box<Expr>, Box<Expr>),
    GreaterEqual(Box<Expr>, Box<Expr>),
    Less(Box<Expr>, Box<Expr>),
    LessEqual(Box<Expr>, Box<Expr>),

    // A parenthesized expression.
    Paren(Box<Expr>),
    FunctionCall(Box<Expr>, Option<Box<Expr>>),
    Ternary(Box<Expr>, Box<Expr>, Box<Expr>),
    // foo[...]
    Index(Box<Expr>, Box<Expr>),
    // a, b
    Comma(Box<Expr>, Box<Expr>),

    // %
    Rem(Box<Expr>, Box<Expr>),
    BitXor(Box<Expr>, Box<Expr>),
    BitAnd(Box<Expr>, Box<Expr>),
}

// Parser definition

pub(crate) fn pratt_parser(i: &mut &str) -> PResult<Expr> {
    use winnow::combinator::precedence;
    // precedence is based on https://en.cppreference.com/w/c/language/operator_precedence
    // but specified in reverse order, because the `cppreference` table
    // uses `descending` precedence, but we need ascending one
    precedence::precedence(
        trace(
            "operand",
            delimited(multispace0, dispatch! {peek(any);
                '(' => delimited('(',  pratt_parser.map(|e| Expr::Paren(Box::new(e))), cut_err(')')),
                _ => alt((
                    identifier.map(|s| Expr::Name(s.into())),
                    digit1.parse_to::<i64>().map(Expr::Value)
                )),
            }, multispace0),
        ),
        trace(
            "prefix",
           delimited(multispace0, dispatch! {any;
                '+' => alt((
                    // ++
                    '+'.value((18, (|_: &mut _, a| Ok(Expr::PreIncr(Box::new(a)))) as _)),
                    empty.value((18, (|_: &mut _, a| Ok(a)) as _))
                )),
                '-' =>  alt((
                    // --
                    '-'.value((18, (|_: &mut _, a| Ok(Expr::PreDecr(Box::new(a)))) as _)),
                    empty.value((18, (|_: &mut _, a| Ok(Expr::Neg(Box::new(a)))) as _))
                )),
                '&' => empty.value((18, (|_: &mut _, a| Ok(Expr::Addr(Box::new(a)))) as _)),
                '*' => empty.value((18, (|_: &mut _, a| Ok(Expr::Deref(Box::new(a)))) as _)),
                '!' => empty.value((18, (|_: &mut _, a| Ok(Expr::Not(Box::new(a)))) as _)),
                _ => fail
            }
        , multispace0)),
        trace(
            "postfix",
            delimited(multispace0, alt((
                dispatch! {any;
                    '!' => empty.value((20, (|_: &mut _, a| Ok(Expr::Fac(Box::new(a)))) as _)),
                    '?' => empty.value((3, (|i: &mut &str, cond| {
                        let (left, right) = preceded(multispace0, cut_err(separated_pair(pratt_parser, delimited(multispace0, ':', multispace0), pratt_parser))).parse_next(i)?;
                        Ok(Expr::Ternary(Box::new(cond), Box::new(left), Box::new(right)))
                    }) as _)),
                    '[' => empty.value((20, (|i: &mut &str, a| {
                        let index = delimited(multispace0, pratt_parser, (multispace0, cut_err(']'), multispace0)).parse_next(i)?;
                        Ok(Expr::Index(Box::new(a), Box::new(index)))
                    }) as _)),
                    '(' => empty.value((20, (|i: &mut &str, a| {
                        let args = delimited(multispace0, opt(pratt_parser), (multispace0, cut_err(')'), multispace0)).parse_next(i)?;
                        Ok(Expr::FunctionCall(Box::new(a), args.map(Box::new)))
                    }) as _)),
                    _ => fail,
                },

                dispatch! {take(2usize);
                    "++" => empty.value((19, (|_: &mut _, a| Ok(Expr::PostIncr(Box::new(a)))) as _)),
                    "--" => empty.value((19, (|_: &mut _, a| Ok(Expr::PostDecr(Box::new(a)))) as _)),
                    _ => fail,
                },
            )), multispace0),
        ),
        trace(
            "infix",
            alt((
                dispatch! {any;
                   '*' => empty.value((16, 17, (|_: &mut _, a, b| Ok(Expr::Mul(Box::new(a), Box::new(b)))) as _)),
                   '/' => empty.value((16, 17, (|_: &mut _, a, b| Ok(Expr::Div(Box::new(a), Box::new(b)))) as _)),
                   '%' => empty.value((16, 17, (|_: &mut _, a, b| Ok(Expr::Rem(Box::new(a), Box::new(b)))) as _)),

                   '+' => empty.value((14, 15, (|_: &mut _, a, b| Ok(Expr::Add(Box::new(a), Box::new(b)))) as _)),
                   '-' => alt((
                        dispatch!{take(2usize);
                            "ne" => empty.value((10, 11, (|_: &mut _, a, b| Ok(Expr::NotEq(Box::new(a), Box::new(b)))) as _)),
                            "eq" => empty.value((10, 11, (|_: &mut _, a, b| Ok(Expr::Eq(Box::new(a), Box::new(b)))) as _)),
                            "gt" => empty.value((12, 13, (|_: &mut _, a, b| Ok(Expr::Greater(Box::new(a), Box::new(b)))) as _)),
                            "ge" => empty.value((12, 13, (|_: &mut _, a, b| Ok(Expr::GreaterEqual(Box::new(a), Box::new(b)))) as _)),
                            "lt" => empty.value((12, 13, (|_: &mut _, a, b| Ok(Expr::Less(Box::new(a), Box::new(b)))) as _)),
                            "le" => empty.value((12, 13, (|_: &mut _, a, b| Ok(Expr::LessEqual(Box::new(a), Box::new(b)))) as _)),
                        _ => fail
                        },
                        '>'.value((19, 20, (|_: &mut _, a, b| Ok(Expr::ArrowOp(Box::new(a), Box::new(b)))) as _)),
                        empty.value((14, 15, (|_: &mut _, a, b| Ok(Expr::Sub(Box::new(a), Box::new(b)))) as _))
                    )),
                   '.' => empty.value((19, 20, (|_: &mut _, a, b| Ok(Expr::Dot(Box::new(a), Box::new(b)))) as _)),
                   '&' => alt((
                        // &&
                        "&".value((6, 7, (|_: &mut _, a, b| Ok(Expr::And(Box::new(a), Box::new(b)))) as _)  ),

                        empty.value((12, 13, (|_: &mut _, a, b| Ok(Expr::BitAnd(Box::new(a), Box::new(b)))) as _)),
                    )),
                   '^' => empty.value((8, 9, (|_: &mut _, a, b| Ok(Expr::BitXor(Box::new(a), Box::new(b)))) as _)),
                   '=' => alt((
                        // ==
                        "=".value((10, 11, (|_: &mut _, a, b| Ok(Expr::Eq(Box::new(a), Box::new(b)))) as _)),
                        empty.value((1, 2, (|_: &mut _, a, b| Ok(Expr::Assign(Box::new(a), Box::new(b)))) as _))
                    )),

                   '>' => alt((
                        // >=
                        "=".value((12, 13, (|_: &mut _, a, b| Ok(Expr::GreaterEqual(Box::new(a), Box::new(b)))) as _)),
                        empty.value((12, 13, (|_: &mut _, a, b| Ok(Expr::Greater(Box::new(a), Box::new(b)))) as _))
                    )),
                   '<' => alt((
                        // <=
                        "=".value((12, 13, (|_: &mut _, a, b| Ok(Expr::LessEqual(Box::new(a), Box::new(b)))) as _)),
                        empty.value((12, 13, (|_: &mut _, a, b| Ok(Expr::Less(Box::new(a), Box::new(b)))) as _))
                    )),
                   ',' => empty.value((0, 1, (|_: &mut _, a, b| Ok(Expr::Comma(Box::new(a), Box::new(b)))) as _)),
                   _ => fail
                },
                dispatch! {take(2usize);
                   "!=" => empty.value((10, 11, (|_: &mut _, a, b| Ok(Expr::NotEq(Box::new(a), Box::new(b)))) as _)),
                   "||" => empty.value((4, 5, (|_: &mut _, a, b| Ok(Expr::Or(Box::new(a), Box::new(b)))) as _)),
                   _ => fail
                },
            )),
        ),
    )
    .parse_next(i)
}

fn identifier<'i>(i: &mut &'i str) -> PResult<&'i str> {
    trace(
        "identifier",
        (
            one_of(|c: char| c.is_alpha() || c == '_'),
            take_while(0.., |c: char| c.is_alphanum() || c == '_'),
        ),
    )
    .take()
    .parse_next(i)
}

impl Expr {
    fn fmt_ast_with_indent(
        &self,
        indent: u32,
        f: &mut core::fmt::Formatter<'_>,
    ) -> core::fmt::Result {
        for _ in 0..indent {
            write!(f, "  ")?;
        }
        macro_rules! binary_fmt {
            ($a:ident, $b:ident, $name:literal) => {{
                writeln!(f, $name)?;
                $a.fmt_ast_with_indent(indent + 1, f)?;
                $b.fmt_ast_with_indent(indent + 1, f)
            }};
        }
        macro_rules! unary_fmt {
            ($a:ident, $name:literal) => {{
                writeln!(f, $name)?;
                $a.fmt_ast_with_indent(indent + 1, f)
            }};
        }
        match self {
            Self::Name(name) => writeln!(f, "NAME {name}"),
            Self::Value(value) => writeln!(f, "VAL {value}"),
            Self::Addr(a) => unary_fmt!(a, "ADDR"),
            Self::Deref(a) => unary_fmt!(a, "DEREF"),
            Self::Neg(a) => unary_fmt!(a, "NEG"),
            Self::Fac(a) => unary_fmt!(a, "FAC"),
            Self::PreIncr(a) => unary_fmt!(a, "PRE_INCR"),
            Self::PostIncr(a) => unary_fmt!(a, "POST_INCR"),
            Self::PreDecr(a) => unary_fmt!(a, "PRE_DECR"),
            Self::PostDecr(a) => unary_fmt!(a, "POST_DECR"),
            Self::Not(a) => unary_fmt!(a, "NOT"),
            Self::Paren(a) => unary_fmt!(a, "PAREN"),
            Self::Assign(a, b) => binary_fmt!(a, b, "ASSIGN"),
            Self::ArrowOp(a, b) => binary_fmt!(a, b, "ARROW"),
            Self::Dot(a, b) => binary_fmt!(a, b, "ARROW"),
            Self::FunctionCall(a, b) => {
                writeln!(f, "CALL")?;
                a.fmt_ast_with_indent(indent + 1, f)?;
                if let Some(b) = b {
                    b.fmt_ast_with_indent(indent + 1, f)?;
                }
                Ok(())
            }
            Self::Add(a, b) => binary_fmt!(a, b, "ADD"),
            Self::Sub(a, b) => binary_fmt!(a, b, "SUB"),
            Self::Mul(a, b) => binary_fmt!(a, b, "MUL"),
            Self::Div(a, b) => binary_fmt!(a, b, "DIV"),
            Self::And(a, b) => binary_fmt!(a, b, "AND"),
            Self::Or(a, b) => binary_fmt!(a, b, "OR"),
            Self::Eq(a, b) => binary_fmt!(a, b, "EQ"),
            Self::NotEq(a, b) => binary_fmt!(a, b, "NEQ"),
            Self::Greater(a, b) => binary_fmt!(a, b, "GREATER"),
            Self::GreaterEqual(a, b) => binary_fmt!(a, b, "GTEQ"),
            Self::Less(a, b) => binary_fmt!(a, b, "LESS"),
            Self::LessEqual(a, b) => binary_fmt!(a, b, "LESSEQ"),
            Self::BitXor(a, b) => binary_fmt!(a, b, "BIT_XOR"),
            Self::Rem(a, b) => binary_fmt!(a, b, "REM"),
            Self::BitAnd(a, b) => binary_fmt!(a, b, "BIT_AND"),
            Self::Index(a, b) => binary_fmt!(a, b, "INDEX"),
            Self::Comma(a, b) => binary_fmt!(a, b, "COMMA"),
            Self::Ternary(cond, a, b) => {
                writeln!(f, "TERNARY")?;
                cond.fmt_ast_with_indent(indent + 1, f)?;
                a.fmt_ast_with_indent(indent + 2, f)?;
                b.fmt_ast_with_indent(indent + 2, f)
            }
        }
    }
    fn fmt_delimited(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Name(name) => return write!(f, "{name}"),
            Self::Value(value) => return write!(f, "{value}"),
            _ => (),
        }
        macro_rules! unary {
            ($op:literal, $a:ident) => {{
                write!(f, $op)?;
                $a.fmt_delimited(f)?;
            }};
        }
        macro_rules! binary {
            ($op:literal, $a:ident, $b:ident) => {{
                write!(f, "{} ", $op)?;
                $a.fmt_delimited(f)?;
                write!(f, " ")?;
                $b.fmt_delimited(f)?;
            }};
        }
        write!(f, "(")?;
        match self {
            Self::Assign(a, b) => binary!("=", a, b),
            Self::FunctionCall(a, b) => {
                a.fmt_delimited(f)?;
                if let Some(b) = b {
                    b.fmt_delimited(f)?;
                } else {
                    write!(f, "()")?;
                }
            }
            Self::ArrowOp(a, b) => binary!("->", a, b),
            Self::Dot(a, b) => binary!(".", a, b),
            Self::Addr(a) => unary!("&", a),
            Self::Deref(a) => unary!("*", a),
            Self::Neg(a) => unary!("-", a),
            Self::Fac(a) => unary!("!", a),
            Self::Not(a) => unary!("!", a),
            Self::PreIncr(a) => unary!("pre++", a),
            Self::PostIncr(a) => unary!("post++", a),
            Self::PreDecr(a) => unary!("pre--", a),
            Self::PostDecr(a) => unary!("post--", a),

            Self::Add(a, b) => binary!("+", a, b),
            Self::Sub(a, b) => binary!("-", a, b),
            Self::Mul(a, b) => binary!("*", a, b),
            Self::Div(a, b) => binary!("/", a, b),
            Self::And(a, b) => binary!("&&", a, b),
            Self::Or(a, b) => binary!("||", a, b),
            Self::Eq(a, b) => binary!("==", a, b),
            Self::NotEq(a, b) => binary!("!=", a, b),
            Self::Greater(a, b) => binary!(">", a, b),
            Self::GreaterEqual(a, b) => binary!(">=", a, b),
            Self::Less(a, b) => binary!("<", a, b),
            Self::LessEqual(a, b) => binary!("<=", a, b),
            Self::BitXor(a, b) => binary!("^", a, b),
            Self::Rem(a, b) => binary!("%", a, b),
            Self::BitAnd(a, b) => binary!("&", a, b),
            Self::Index(a, b) => binary!("[]", a, b),
            Self::Comma(a, b) => binary!(",", a, b),

            Self::Paren(a) => {
                write!(f, "(")?;
                a.fmt_delimited(f)?;
                write!(f, ")")?;
            }
            Self::Ternary(cond, a, b) => {
                write!(f, "? ")?;
                cond.fmt_delimited(f)?;
                write!(f, " ")?;
                a.fmt_delimited(f)?;
                write!(f, " ")?;
                b.fmt_delimited(f)?;
            }
            _ => unreachable!(),
        }

        write!(f, ")")
    }
}

impl core::fmt::Display for Expr {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.fmt_ast_with_indent(0, f)?;
        writeln!(f)?;
        self.fmt_delimited(f)
    }
}

#[cfg(test)]
mod test {
    #[allow(clippy::useless_attribute)]
    #[allow(unused_imports)] // its dead for benches
    use super::*;
    #[test]
    fn test_simple() {
        let r = pratt_parser
            // .parse("+- --!&**foo! + 3 * 4 - bar ^ e")
            // .parse("foo(a + b)")
            // .parse("1 + 2 * *4^7! + 6")
            .parse("foo(1 + 2 + 3) + bar() ? 1 : 2")
            .unwrap();
        println!("{r}");
    }
}
