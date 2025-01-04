use winnow::combinator::{cut_err, empty, fail, not, opt, peek, separated_pair, trace};
use winnow::error::ContextError;
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
    Pow(Box<Expr>, Box<Expr>),
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
    BitwiseNot(Box<Expr>),
}

// Parser definition

pub(crate) fn pratt_parser(i: &mut &str) -> PResult<Expr> {
    use winnow::combinator::precedence::{self, Assoc};
    // precedence is based on https://en.cppreference.com/w/c/language/operator_precedence
    // but specified in reverse order, because the `cppreference` table
    // uses `descending` precedence, but we need ascending one
    fn parser<'i>(start_power: i64) -> impl Parser<&'i str, Expr, ContextError> {
        move |i: &mut &str| {
            precedence::precedence(
            start_power,
            trace(
                "operand",
                delimited(
                    multispace0,
                    dispatch! {peek(any);
                        '(' => delimited('(',  parser(0).map(|e| Expr::Paren(Box::new(e))), cut_err(')')),
                        _ => alt((
                            identifier.map(|s| Expr::Name(s.into())),
                            digit1.parse_to::<i64>().map(Expr::Value)
                        )),
                    },
                    multispace0,
                ),
            ),
            trace(
                "prefix",
                delimited(
                    multispace0,
                    dispatch! {any;
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
                        '~' => empty.value((18, (|_: &mut _, a| Ok(Expr::BitwiseNot(Box::new(a)))) as _)),
                        _ => fail
                    },
                    multispace0,
                ),
            ),
            trace(
                "postfix",
                delimited(
                    multispace0,
                    alt((
                        dispatch! {any;
                            '!' => not('=').value((19, (|_: &mut _, a| Ok(Expr::Fac(Box::new(a)))) as _)),
                            '?' => empty.value((3, (|i: &mut &str, cond| {
                                let (left, right) = cut_err(separated_pair(parser(0), delimited(multispace0, ':', multispace0), parser(3))).parse_next(i)?;
                                Ok(Expr::Ternary(Box::new(cond), Box::new(left), Box::new(right)))
                            }) as _)),
                            '[' => empty.value((20, (|i: &mut &str, a| {
                                let index = delimited(multispace0, parser(0), (multispace0, cut_err(']'), multispace0)).parse_next(i)?;
                                Ok(Expr::Index(Box::new(a), Box::new(index)))
                            }) as _)),
                            '(' => empty.value((20, (|i: &mut &str, a| {
                                let args = delimited(multispace0, opt(parser(0)), (multispace0, cut_err(')'), multispace0)).parse_next(i)?;
                                Ok(Expr::FunctionCall(Box::new(a), args.map(Box::new)))
                            }) as _)),
                            _ => fail,
                        },
                        dispatch! {take(2usize);
                            "++" => empty.value((20, (|_: &mut _, a| Ok(Expr::PostIncr(Box::new(a)))) as _)),
                            "--" => empty.value((20, (|_: &mut _, a| Ok(Expr::PostDecr(Box::new(a)))) as _)),
                            _ => fail,
                        },
                    )),
                    multispace0,
                ),
            ),
            trace(
                "infix",
                alt((
                    dispatch! {any;
                        '*' => alt((
                            // **
                            "*".value((Assoc::Right(28), (|_: &mut _, a, b| Ok(Expr::Pow(Box::new(a), Box::new(b)))) as _)),
                            empty.value((Assoc::Left(16), (|_: &mut _, a, b| Ok(Expr::Mul(Box::new(a), Box::new(b)))) as _)),
                        )),
                        '/' => empty.value((Assoc::Left(16), (|_: &mut _, a, b| Ok(Expr::Div(Box::new(a), Box::new(b)))) as _)),
                        '%' => empty.value((Assoc::Left(16), (|_: &mut _, a, b| Ok(Expr::Rem(Box::new(a), Box::new(b)))) as _)),

                        '+' => empty.value((Assoc::Left(14), (|_: &mut _, a, b| Ok(Expr::Add(Box::new(a), Box::new(b)))) as _)),
                        '-' => alt((
                            dispatch!{take(2usize);
                                "ne" => empty.value((Assoc::Neither(10), (|_: &mut _, a, b| Ok(Expr::NotEq(Box::new(a), Box::new(b)))) as _)),
                                "eq" => empty.value((Assoc::Neither(10), (|_: &mut _, a, b| Ok(Expr::Eq(Box::new(a), Box::new(b)))) as _)),
                                "gt" => empty.value((Assoc::Neither(12), (|_: &mut _, a, b| Ok(Expr::Greater(Box::new(a), Box::new(b)))) as _)),
                                "ge" => empty.value((Assoc::Neither(12), (|_: &mut _, a, b| Ok(Expr::GreaterEqual(Box::new(a), Box::new(b)))) as _)),
                                "lt" => empty.value((Assoc::Neither(12), (|_: &mut _, a, b| Ok(Expr::Less(Box::new(a), Box::new(b)))) as _)),
                                "le" => empty.value((Assoc::Neither(12), (|_: &mut _, a, b| Ok(Expr::LessEqual(Box::new(a), Box::new(b)))) as _)),
                                _ => fail
                            },
                            '>'.value((Assoc::Left(20), (|_: &mut _, a, b| Ok(Expr::ArrowOp(Box::new(a), Box::new(b)))) as _)),
                            empty.value((Assoc::Left(14), (|_: &mut _, a, b| Ok(Expr::Sub(Box::new(a), Box::new(b)))) as _))
                        )),
                        '.' => empty.value((Assoc::Left(20), (|_: &mut _, a, b| Ok(Expr::Dot(Box::new(a), Box::new(b)))) as _)),
                        '&' => alt((
                            // &&
                            "&".value((Assoc::Left(6), (|_: &mut _, a, b| Ok(Expr::And(Box::new(a), Box::new(b)))) as _)  ),

                            empty.value((Assoc::Left(12), (|_: &mut _, a, b| Ok(Expr::BitAnd(Box::new(a), Box::new(b)))) as _)),
                        )),
                        '^' => empty.value((Assoc::Left(8), (|_: &mut _, a, b| Ok(Expr::BitXor(Box::new(a), Box::new(b)))) as _)),
                        '=' => alt((
                            // ==
                            "=".value((Assoc::Neither(10), (|_: &mut _, a, b| Ok(Expr::Eq(Box::new(a), Box::new(b)))) as _)),
                            empty.value((Assoc::Right(2), (|_: &mut _, a, b| Ok(Expr::Assign(Box::new(a), Box::new(b)))) as _))
                        )),

                        '>' => alt((
                            // >=
                            "=".value((Assoc::Neither(12), (|_: &mut _, a, b| Ok(Expr::GreaterEqual(Box::new(a), Box::new(b)))) as _)),
                            empty.value((Assoc::Neither(12), (|_: &mut _, a, b| Ok(Expr::Greater(Box::new(a), Box::new(b)))) as _))
                        )),
                        '<' => alt((
                            // <=
                            "=".value((Assoc::Neither(12), (|_: &mut _, a, b| Ok(Expr::LessEqual(Box::new(a), Box::new(b)))) as _)),
                            empty.value((Assoc::Neither(12), (|_: &mut _, a, b| Ok(Expr::Less(Box::new(a), Box::new(b)))) as _))
                        )),
                        ',' => empty.value((Assoc::Left(0), (|_: &mut _, a, b| Ok(Expr::Comma(Box::new(a), Box::new(b)))) as _)),
                        _ => fail
                    },
                    dispatch! {take(2usize);
                        "!=" => empty.value((Assoc::Neither(10), (|_: &mut _, a, b| Ok(Expr::NotEq(Box::new(a), Box::new(b)))) as _)),
                        "||" => empty.value((Assoc::Left(4), (|_: &mut _, a, b| Ok(Expr::Or(Box::new(a), Box::new(b)))) as _)),
                        _ => fail
                    },
                )),
            ),
        ).parse_next(i)
        }
    }
    parser(0).parse_next(i)
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
            Self::BitwiseNot(a) => unary_fmt!(a, "BIT_NOT"),
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
            Self::Pow(a, b) => binary_fmt!(a, b, "POW"),
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
            Self::Paren(a) => return a.fmt_delimited(f),
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
                write!(f, "call ")?;
                a.fmt_delimited(f)?;
                if let Some(b) = b {
                    write!(f, " ")?;
                    b.fmt_delimited(f)?;
                }
            }
            Self::ArrowOp(a, b) => binary!("->", a, b),
            Self::Dot(a, b) => binary!(".", a, b),
            Self::Addr(a) => unary!("&", a),
            Self::Deref(a) => unary!("*", a),
            Self::Neg(a) => unary!("-", a),
            Self::Fac(a) => unary!("!", a),
            Self::Not(a) => unary!("!", a),
            Self::BitwiseNot(a) => unary!("~", a),
            Self::PreIncr(a) => unary!("pre++", a),
            Self::PostIncr(a) => unary!("post++", a),
            Self::PreDecr(a) => unary!("pre--", a),
            Self::PostDecr(a) => unary!("post--", a),
            Self::Add(a, b) => binary!("+", a, b),
            Self::Sub(a, b) => binary!("-", a, b),
            Self::Mul(a, b) => binary!("*", a, b),
            Self::Div(a, b) => binary!("/", a, b),
            Self::Pow(a, b) => binary!("**", a, b),
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
    use winnow::error::ParseError;

    #[allow(dead_code)]
    // to invoke fmt_delimited()
    struct PrefixNotation(Expr);

    impl core::fmt::Display for PrefixNotation {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            self.0.fmt_delimited(f)
        }
    }

    #[allow(dead_code)]
    fn parse(i: &str) -> Result<String, ParseError<&str, winnow::error::ContextError>> {
        pratt_parser
            .parse(i)
            .map(|r| format!("{}", PrefixNotation(r)))
    }

    #[allow(dead_code)]
    fn parse_ok(i: &str, expect: &str) {
        assert_eq!(parse(i).unwrap(), expect);
    }

    #[test]
    fn op() {
        parse_ok("  1 ", "1");
    }

    #[test]
    fn neither() {
        assert!(parse("1 == 2 == 3").is_err());
        assert!(parse("1 -le 2 -gt 3").is_err());
        assert!(parse("1 < 2 < 3").is_err());
        assert!(parse("1 != 2 == 3").is_err());
    }

    #[test]
    fn equal() {
        parse_ok("x=3", "(= x 3)");
        parse_ok("x = 2*3", "(= x (* 2 3))");
        parse_ok("x = y", "(= x y)");
        parse_ok("a = b = 10", "(= a (= b 10))");
        parse_ok("x = ((y*4)-2)", "(= x (- (* y 4) 2))");
    }

    #[test]
    fn unary() {
        parse_ok("- - a", "(-(-a))");
        parse_ok("+ - a", "(-a)");
        parse_ok("++ -- a", "(pre++(pre--a))");
        parse_ok("a ++ --", "(post--(post++a))");
        parse_ok("!x", "(!x)");
        parse_ok("x--", "(post--x)");
        parse_ok("x[1]--", "(post--([] x 1))");
        parse_ok("--x", "(pre--x)");
        parse_ok("++x[1]", "(pre++([] x 1))");
        parse_ok("!x--", "(!(post--x))");
        parse_ok("~x++", "(~(post++x))");
        parse_ok("x++ - y++", "(- (post++x) (post++y))");
        parse_ok("++x - ++y", "(- (pre++x) (pre++y))");
        parse_ok("--1 * 2", "(* (pre--1) 2)");
        parse_ok("--f . g", "(pre--(. f g))");
    }

    #[test]
    fn same_precedence() {
        // left associative
        parse_ok("1 + 2 + 3", "(+ (+ 1 2) 3)");
        parse_ok("1 - 2 - 3", "(- (- 1 2) 3)");
        parse_ok("1 * 2 * 3", "(* (* 1 2) 3)");
        parse_ok("1 / 2 / 3", "(/ (/ 1 2) 3)");
        parse_ok("1 % 2 % 3", "(% (% 1 2) 3)");
        parse_ok("1 ^ 2 ^ 3", "(^ (^ 1 2) 3)");
        parse_ok("+-+1", "(-1)");
        parse_ok("f . g . h", "(. (. f g) h)");
        parse_ok("++--++1", "(pre++(pre--(pre++1)))");
        // right associative
        parse_ok("2 ** 3 ** 2", "(** 2 (** 3 2))");
    }

    #[test]
    fn different_precedence() {
        parse_ok("1 + 2 * 3", "(+ 1 (* 2 3))");
        parse_ok("1 + 2 * 3 - 4 / 5", "(- (+ 1 (* 2 3)) (/ 4 5))");
        parse_ok("a + b * c * d + e", "(+ (+ a (* (* b c) d)) e)");
        parse_ok("1 + ++2 * 3 * 5 + 6", "(+ (+ 1 (* (* (pre++2) 3) 5)) 6)");
        parse_ok("**3 + &1", "(+ (*(*3)) (&1))");
        parse_ok("x*y - y*z", "(- (* x y) (* y z))");
        parse_ok("x/y - y%z", "(- (/ x y) (% y z))");
        parse_ok("1<2 * 3", "(< 1 (* 2 3))");
        parse_ok(
            " 1 + 2 + f . g . h * 3 * 4",
            "(+ (+ 1 2) (* (* (. (. f g) h) 3) 4))",
        );
    }

    #[test]
    fn prefix_postfix_power() {
        // https://en.cppreference.com/w/c/language/operator_precedence
        // `post++` has `1`, `pre--` and `*` have 2
        parse_ok("--**3++", "(pre--(*(*(post++3))))");
        parse_ok("**--3++", "(*(*(pre--(post++3))))");
        parse_ok("&foo()[0]", "(&([] (call foo) 0))");
        parse_ok("-9!", "(-(!9))");
        parse_ok("f . g !", "(!(. f g))");
    }

    #[test]
    fn prefix_infix() {
        parse_ok("x - -y", "(- x (-y))");
        parse_ok("-1 * -2", "(* (-1) (-2))");
        parse_ok("-x * -y", "(* (-x) (-y))");
        parse_ok("x - -234", "(- x (-234))");
    }

    #[test]
    fn ternary() {
        parse_ok("a ? 2 + c : -2 * 2", "(? a (+ 2 c) (* (-2) 2))");
        parse_ok("a ? b : c ? d : e", "(? a b (? c d e))");
        parse_ok("2! > 1 ? 3 : 1", "(? (> (!2) 1) 3 1)");
        parse_ok(
            "2 > 1 ? 1 -ne 3 ? 4 : 5 : 1",
            "(? (> 2 1) (? (!= 1 3) 4 5) 1)",
        );
        parse_ok("a > b ? 0 : 1", "(? (> a b) 0 1)");
        parse_ok("a > b ? x+1 : y+1", "(? (> a b) (+ x 1) (+ y 1))");
        parse_ok(
            "1 ? true1 : 2 ? true2 : false",
            "(? 1 true1 (? 2 true2 false))",
        );
        parse_ok(
            "1 ?      true1 : (2 ? true2 : false)",
            "(? 1 true1 (? 2 true2 false))",
        );

        parse_ok(
            "1 ? (2 ? true : false1) : false2",
            "(? 1 (? 2 true false1) false2)",
        );
        parse_ok(
            "1 ? 2 ? true : false1 : false2",
            "(? 1 (? 2 true false1) false2)",
        );
    }

    #[test]
    fn comma() {
        parse_ok("x=1,y=2,z=3", "(, (, (= x 1) (= y 2)) (= z 3))");
        parse_ok("a, b, c", "(, (, a b) c)");
        parse_ok("(a, b, c)", "(, (, a b) c)");
        parse_ok("f(a, b, c), d", "(, (call f (, (, a b) c)) d)");
        parse_ok("(a, b, c), d", "(, (, (, a b) c) d)");
    }

    #[test]
    fn comma_ternary() {
        parse_ok("x ? 1 : 2, y ? 3 : 4", "(, (? x 1 2) (? y 3 4))");
        // Comma expressions can be inside
        parse_ok("a , b ? c, d : e, f", "(, (, a (? b (, c d) e)) f)");
        parse_ok("a = 0 ? b : c = d", "(= a (= (? 0 b c) d))");
    }

    #[test]
    fn braces() {
        parse_ok("4*(2+3)", "(* 4 (+ 2 3))");
        parse_ok("(2+3)*4", "(* (+ 2 3) 4)");
        parse_ok("(((0)))", "0");
    }

    #[test]
    fn logical() {
        parse_ok("a && b || c && d", "(|| (&& a b) (&& c d))");
        parse_ok("!a && !b", "(&& (!a) (!b))");
        parse_ok("a != b && c == d", "(&& (!= a b) (== c d))");
    }

    #[test]
    fn array() {
        parse_ok("x[1,2]", "([] x (, 1 2))");
        parse_ok("x[1]", "([] x 1)");
        parse_ok("x[a+b]", "([] x (+ a b))");
        parse_ok("c = pal[i*8]", "(= c ([] pal (* i 8)))");
        parse_ok("f[x] = 1", "(= ([] f x) 1)");
        parse_ok("x[0][1]", "([] ([] x 0) 1)");
    }

    #[test]
    fn function_call() {
        parse_ok("a()", "(call a)");
        parse_ok("a(+1)", "(call a 1)");
        parse_ok("a()+1", "(+ (call a) 1)");
        parse_ok("f(a, b, c)", "(call f (, (, a b) c))");
        parse_ok("print(x)", "(call print x)");
        parse_ok(
            "x = y(2)*3 + y(4)*5",
            "(= x (+ (* (call y 2) 3) (* (call y 4) 5)))",
        );
        parse_ok("x(1,2)+y(3,4)", "(+ (call x (, 1 2)) (call y (, 3 4)))");
        parse_ok("x(a,b,c[d])", "(call x (, (, a b) ([] c d)))");
        parse_ok(
            "x(1,2)*j+y(3,4)*k+z(5,6)*l",
            "(+ (+ (* (call x (, 1 2)) j) (* (call y (, 3 4)) k)) (* (call z (, 5 6)) l))",
        );
        parse_ok("print(test(2,3))", "(call print (call test (, 2 3)))");
        parse_ok("min(255,n*2)", "(call min (, 255 (* n 2)))");
    }

    #[test]
    fn member_access() {
        parse_ok("a.b", "(. a b)");
        parse_ok("a.b.c", "(. (. a b) c)");
        parse_ok("a->b", "(-> a b)");
        parse_ok("++a->b", "(pre++(-> a b))");
        parse_ok("a++ ->b", "(-> (post++a) b)");
        parse_ok("a.(x)", "(. a x)");
        parse_ok("a.(x+3)", "(. a (+ x 3))");
    }

    #[test]
    fn errors() {
        assert!(parse("x + a b").is_err());
        assert!(parse("x[a b]").is_err());
        assert!(parse("x[a)]").is_err());
        assert!(parse("x(a])").is_err());
        assert!(parse("[a + b]").is_err());
        assert!(parse("[a b]").is_err());
        assert!(parse("+").is_err());
        assert!(parse("a +").is_err());
        assert!(parse("<=").is_err());
        assert!(parse("<= - a + b").is_err());
        assert!(parse("a b").is_err());
        assert!(parse("a + b @").is_err());
        assert!(parse("a + b )").is_err());
        assert!(parse("( a + b").is_err());
        assert!(parse("( a + b) c").is_err());
        assert!(parse("f ( a + b ) c").is_err());
        assert!(parse("@ a + b").is_err());
        assert!(parse("a @ b").is_err());
        assert!(parse("(a @ b)").is_err());
        assert!(parse(")").is_err());
    }
}
