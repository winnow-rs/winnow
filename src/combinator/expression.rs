use core::marker::PhantomData;

use crate::{
    combinator::{opt, trace},
    error::{ErrMode, ParserError},
    stream::{Stream, StreamIsPartial},
    PResult, Parser,
};

use super::{empty, fail};

/// Parses an expression based on operator precedence.
#[doc(alias = "pratt")]
#[doc(alias = "separated")]
#[doc(alias = "shunting_yard")]
#[doc(alias = "precedence_climbing")]
#[inline(always)]
pub fn expression<I, ParseOperand, O, E>(
    start_power: i64,
    parse_operand: ParseOperand,
) -> Expression<
    I,
    O,
    ParseOperand,
    impl Parser<I, Prefix<I, O, E>, E>,
    impl Parser<I, Postfix<I, O, E>, E>,
    impl Parser<I, Infix<I, O, E>, E>,
    E,
>
where
    I: Stream + StreamIsPartial,
    ParseOperand: Parser<I, O, E>,
    E: ParserError<I>,
{
    Expression {
        start_power,
        parse_operand,
        parse_prefix: fail,
        parse_postfix: fail,
        parse_infix: fail,
        i: Default::default(),
        o: Default::default(),
        e: Default::default(),
    }
}

pub struct Expression<I, O, ParseOperand, Pre, Post, Pix, E>
where
    I: Stream + StreamIsPartial,
    ParseOperand: Parser<I, O, E>,
    E: ParserError<I>,
{
    start_power: i64,
    parse_operand: ParseOperand,
    parse_prefix: Pre,
    parse_postfix: Post,
    parse_infix: Pix,
    i: PhantomData<I>,
    o: PhantomData<O>,
    e: PhantomData<E>,
}

impl<I, O, ParseOperand, Pre, Post, Pix, E> Expression<I, O, ParseOperand, Pre, Post, Pix, E>
where
    ParseOperand: Parser<I, O, E>,
    I: Stream + StreamIsPartial,
    E: ParserError<I>,
{
    #[inline(always)]
    pub fn prefix<NewParsePrefix>(
        self,
        parser: NewParsePrefix,
    ) -> Expression<I, O, ParseOperand, NewParsePrefix, Post, Pix, E>
    where
        NewParsePrefix: Parser<I, Prefix<I, O, E>, E>,
    {
        Expression {
            start_power: self.start_power,
            parse_operand: self.parse_operand,
            parse_prefix: parser,
            parse_postfix: self.parse_postfix,
            parse_infix: self.parse_infix,
            i: Default::default(),
            o: Default::default(),
            e: Default::default(),
        }
    }

    #[inline(always)]
    pub fn postfix<NewParsePostfix>(
        self,
        parser: NewParsePostfix,
    ) -> Expression<I, O, ParseOperand, Pre, NewParsePostfix, Pix, E>
    where
        NewParsePostfix: Parser<I, Postfix<I, O, E>, E>,
    {
        Expression {
            start_power: self.start_power,
            parse_operand: self.parse_operand,
            parse_prefix: self.parse_prefix,
            parse_postfix: parser,
            parse_infix: self.parse_infix,
            i: Default::default(),
            o: Default::default(),
            e: Default::default(),
        }
    }

    #[inline(always)]
    pub fn infix<NewParseInfix>(
        self,
        parser: NewParseInfix,
    ) -> Expression<I, O, ParseOperand, Pre, Post, NewParseInfix, E>
    where
        NewParseInfix: Parser<I, Infix<I, O, E>, E>,
    {
        Expression {
            start_power: self.start_power,
            parse_operand: self.parse_operand,
            parse_prefix: self.parse_prefix,
            parse_postfix: self.parse_postfix,
            parse_infix: parser,
            i: Default::default(),
            o: Default::default(),
            e: Default::default(),
        }
    }
}

impl<I, O, Pop, Pre, Post, Pix, E> Parser<I, O, E> for Expression<I, O, Pop, Pre, Post, Pix, E>
where
    I: Stream + StreamIsPartial,
    Pop: Parser<I, O, E>,
    Pix: Parser<I, Infix<I, O, E>, E>,
    Pre: Parser<I, Prefix<I, O, E>, E>,
    Post: Parser<I, Postfix<I, O, E>, E>,
    E: ParserError<I>,
{
    #[inline(always)]
    fn parse_next(&mut self, input: &mut I) -> PResult<O, E> {
        trace("expression", move |i: &mut I| {
            expression_impl(
                i,
                &mut self.parse_operand,
                &mut self.parse_prefix,
                &mut self.parse_postfix,
                &mut self.parse_infix,
                self.start_power,
            )
        })
        .parse_next(input)
    }
}

fn expression_impl<I, O, Pop, Pre, Post, Pix, E>(
    i: &mut I,
    parse_operand: &mut Pop,
    prefix: &mut Pre,
    postfix: &mut Post,
    infix: &mut Pix,
    min_power: i64,
) -> PResult<O, E>
where
    I: Stream + StreamIsPartial,
    Pop: Parser<I, O, E>,
    Pix: Parser<I, Infix<I, O, E>, E>,
    Pre: Parser<I, Prefix<I, O, E>, E>,
    Post: Parser<I, Postfix<I, O, E>, E>,
    E: ParserError<I>,
{
    let operand = opt(trace("operand", parse_operand.by_ref())).parse_next(i)?;
    let mut operand = if let Some(operand) = operand {
        operand
    } else {
        // Prefix unary operators
        let len = i.eof_offset();
        let Prefix(power, fold_prefix) = trace("prefix", prefix.by_ref()).parse_next(i)?;
        // infinite loop check: the parser must always consume
        if i.eof_offset() == len {
            return Err(ErrMode::assert(i, "`prefix` parsers must always consume"));
        }
        let operand = expression_impl(i, parse_operand, prefix, postfix, infix, power)?;
        fold_prefix(i, operand)?
    };

    // A variable to stop the `'parse` loop when `Assoc::Neither` with the same
    // precedence is encountered e.g. `a == b == c`. `Assoc::Neither` has similar
    // associativity rules as `Assoc::Left`, but we stop parsing when the next operator
    // is the same as the current one.
    let mut prev_op_is_neither = None;
    'parse: while i.eof_offset() > 0 {
        // Postfix unary operators
        let start = i.checkpoint();
        if let Some(Postfix(power, fold_postfix)) =
            opt(trace("postfix", postfix.by_ref())).parse_next(i)?
        {
            // control precedence over the prefix e.g.:
            // `--(i++)` or `(--i)++`
            if power < min_power {
                i.reset(&start);
                break 'parse;
            }
            operand = fold_postfix(i, operand)?;

            continue 'parse;
        }

        // Infix binary operators
        let start = i.checkpoint();
        let parse_result = opt(trace("infix", infix.by_ref())).parse_next(i)?;
        if let Some(infix_op) = parse_result {
            let mut is_neither = None;
            let (lpower, rpower, fold_infix) = match infix_op {
                Infix::Right(p, f) => (p, p - 1, f),
                Infix::Left(p, f) => (p, p + 1, f),
                Infix::Neither(p, f) => {
                    is_neither = Some(p);
                    (p, p + 1, f)
                }
            };
            if lpower < min_power
                // MSRV: `is_some_and`
                || match prev_op_is_neither {
                    None => false,
                    Some(p) => lpower == p,
                }
            {
                i.reset(&start);
                break 'parse;
            }
            prev_op_is_neither = is_neither;
            let rhs = expression_impl(i, parse_operand, prefix, postfix, infix, rpower)?;
            operand = fold_infix(i, operand, rhs)?;

            continue 'parse;
        }

        break 'parse;
    }

    Ok(operand)
}

pub struct Prefix<I, O, E>(i64, fn(&mut I, O) -> PResult<O, E>);

impl<I, O, E> Clone for Prefix<I, O, E> {
    #[inline(always)]
    fn clone(&self) -> Self {
        Prefix(self.0, self.1)
    }
}

impl<I: Stream, O, E: ParserError<I>> Parser<I, Prefix<I, O, E>, E> for Prefix<I, O, E> {
    #[inline(always)]
    fn parse_next(&mut self, input: &mut I) -> PResult<Prefix<I, O, E>, E> {
        empty.value(self.clone()).parse_next(input)
    }
}

pub struct Postfix<I, O, E>(i64, fn(&mut I, O) -> PResult<O, E>);

impl<I, O, E> Clone for Postfix<I, O, E> {
    #[inline(always)]
    fn clone(&self) -> Self {
        Postfix(self.0, self.1)
    }
}

impl<I: Stream, O, E: ParserError<I>> Parser<I, Postfix<I, O, E>, E>
    for (i64, fn(&mut I, O) -> PResult<O, E>)
{
    #[inline(always)]
    fn parse_next(&mut self, input: &mut I) -> PResult<Postfix<I, O, E>, E> {
        empty.value(Postfix(self.0, self.1)).parse_next(input)
    }
}

pub enum Infix<I, O, E> {
    Left(i64, fn(&mut I, O, O) -> PResult<O, E>),
    Right(i64, fn(&mut I, O, O) -> PResult<O, E>),
    Neither(i64, fn(&mut I, O, O) -> PResult<O, E>),
}

impl<I, O, E> Clone for Infix<I, O, E> {
    #[inline(always)]
    fn clone(&self) -> Self {
        match self {
            Infix::Left(p, f) => Infix::Left(*p, *f),
            Infix::Right(p, f) => Infix::Right(*p, *f),
            Infix::Neither(p, f) => Infix::Neither(*p, *f),
        }
    }
}

impl<I: Stream, O, E: ParserError<I>> Parser<I, Infix<I, O, E>, E> for Infix<I, O, E> {
    #[inline(always)]
    fn parse_next(&mut self, input: &mut I) -> PResult<Infix<I, O, E>, E> {
        empty.value(self.clone()).parse_next(input)
    }
}

#[cfg(test)]
mod tests {
    use crate::ascii::digit1;
    use crate::combinator::fail;
    use crate::dispatch;
    use crate::error::ContextError;
    use crate::token::any;

    use super::*;

    fn parser<'i>() -> impl Parser<&'i str, i32, ContextError> {
        move |i: &mut &str| {
            use Infix::*;
            expression(0, digit1.parse_to::<i32>())
                .prefix(dispatch! {any;
                    '+' => Prefix(12, |_: &mut _, a: i32| Ok(a)),
                    '-' => Prefix(12, |_: &mut _, a: i32| Ok(-a)),
                    _ => fail
                })
                .infix(dispatch! {any;
                   '+' => Left(5, |_: &mut _, a, b| Ok(a + b)),
                   '-' => Left(5, |_: &mut _, a, b| Ok(a - b)),
                   '*' => Left(7, |_: &mut _, a, b| Ok(a * b)),
                   '/' => Left(7, |_: &mut _, a, b| Ok(a / b)),
                   '%' => Left(7, |_: &mut _, a, b| Ok(a % b)),
                   '^' => Left(9, |_: &mut _, a, b| Ok(a ^ b)),
                   _ => fail
                })
                .parse_next(i)
        }
    }

    #[test]
    fn test_expression() {
        assert_eq!(parser().parse("-3+-3*4"), Ok(-15));
        assert_eq!(parser().parse("+2+3*4"), Ok(14));
        assert_eq!(parser().parse("2*3+4"), Ok(10));
    }
}
