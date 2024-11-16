use core::cell::RefCell;

use crate::{
    combinator::{alt, fail, opt, trace},
    error::{ErrMode, ParserError},
    stream::{Stream, StreamIsPartial},
    PResult, Parser,
};

/// An adapter for the [`Parser`] trait to enable its use in the [`precedence`] parser.
pub trait PrecedenceParserExt<I, E> {
    /// Specifies that the parser is a `unary` `prefix` operator within a [`precedence`] parser.
    ///
    /// In most languages, operators such negation: `-`, `not` or `!`, dereferencing: `*`, etc. are prefix unary operators.
    ///
    /// The argument `fold` is a fold function that defines how to combine the operator and operand into a new expression.
    /// It must have the following signature:
    /// ```ignore
    /// impl Fn(O) -> O
    /// ```
    #[inline(always)]
    fn prefix<F, O>(self, fold: F) -> Prefix<Operator<F, Self>>
    where
        F: UnaryOp<O>,
        Self: Sized,
    {
        Prefix(Operator::new(self, fold))
    }
    /// Specifies that the parser is a `unary` `postfix` operator within a [`precedence`] parser.
    ///
    /// Operators like the factorial `!` are postfix unary operators.
    ///
    /// The argument `fold` is a fold function that defines how to combine the operator and operand into a new
    /// expression. It must have the following signature:
    /// ```ignore
    /// impl Fn(O) -> O
    /// ```
    #[inline(always)]
    fn postfix<F, O>(self, fold: F) -> Postfix<Operator<F, Self>>
    where
        F: UnaryOp<O>,
        Self: Sized,
    {
        Postfix(Operator::new(self, fold))
    }
    /// Specifies that the parser is a `binary` `infix` operator within a [`precedence`] parser.
    ///
    /// Operators like factorial `+`, `-`, `*`, `/` are infix binary operators.
    ///
    /// The argument is a fold function that defines how to combine the operator and two operands into a new
    /// expression. It must have the following signature:
    /// ```ignore
    /// impl Fn(O, O) -> O
    /// ```
    #[inline(always)]
    fn infix<F, O>(self, fold: F) -> Infix<Operator<F, Self>>
    where
        F: BinaryOp<O>,
        Self: Sized,
    {
        Infix(Operator::new(self, fold))
    }
}

impl<I, E, T: Parser<I, usize, E>> PrecedenceParserExt<I, E> for T where I: Stream {}

/// `NewType` that indicates this type is a prefix operator a [`precedence`] parser.
/// See: [`PrecedenceParserExt::prefix`]
///
/// Can hold and arbitrary parser, such as a tuple of multiple operator parsers: `(Operator<...>, Operator<...>)`
pub struct Prefix<T>(T);

/// `NewType` that indicates this type is a postfix operator a [`precedence`] parser.
/// See: [`PrecedenceParserExt::postfix`]
pub struct Postfix<T>(T);

/// `NewType` that indicates this type is a infix operator within a [`precedence`] parser.
/// See: [`PrecedenceParserExt::infix`]
pub struct Infix<T>(T);

/// Implementation of the operator parser for the [`precedence`] parser.
pub struct Operator<OperatorFunc, OperatorParser> {
    // We use two different `ReffCell`s to enable mutable borrowing within the recursion
    // while holding a reference to the predicate `op`:
    // ```
    //      let lhs = ...;
    //      let op: &ReffCell<dyn BinaryOp<...>> = operator.parse_next(i); // calls `operator.parser.borrow_mut().parse_next(i)`
    //      let rhs = recursion(&operator);
    //      let result = op.borrow_mut().fold_binary(lhs, rhs);
    // ```
    op: RefCell<OperatorFunc>,
    parser: RefCell<OperatorParser>,
}

impl<OperatorFunc, OperatorParser> Operator<OperatorFunc, OperatorParser> {
    /// Creates a new [`Operator`] from a parser and a predicate
    #[inline(always)]
    pub fn new(parser: OperatorParser, predicate: OperatorFunc) -> Self {
        Self {
            op: RefCell::new(predicate),
            parser: RefCell::new(parser),
        }
    }
}

/// Type-erased unary predicate that folds an expression into a new expression.
/// Useful for supporting not only closures but also arbitrary types as operator predicates within the [`precedence`] parser.
pub trait UnaryOp<O> {
    /// Invokes the [`UnaryOp`] predicate.
    fn fold_unary(&mut self, o: O) -> O;
}
/// Type-erased binary predicate that folds two expressions into a new expression similar to
/// [`UnaryOp`] within the [`precedence`] parser.
pub trait BinaryOp<O> {
    /// Invokes the [`BinaryOp`] predicate.
    fn fold_binary(&mut self, lhs: O, rhs: O) -> O;
}

impl<O, F> UnaryOp<O> for F
where
    F: Fn(O) -> O,
{
    #[inline(always)]
    fn fold_unary(&mut self, o: O) -> O {
        (self)(o)
    }
}
impl<O, F> BinaryOp<O> for F
where
    F: Fn(O, O) -> O,
{
    #[inline(always)]
    fn fold_binary(&mut self, lhs: O, rhs: O) -> O {
        (self)(lhs, rhs)
    }
}

impl<'s, UO, O, I, P, E> Parser<I, (&'s RefCell<dyn UnaryOp<O>>, usize), E> for &'s Operator<UO, P>
where
    UO: UnaryOp<O> + 'static,
    I: Stream + StreamIsPartial,
    P: Parser<I, usize, E>,
    E: ParserError<I>,
{
    #[inline(always)]
    fn parse_next(
        &mut self,
        input: &mut I,
    ) -> PResult<(&'s RefCell<dyn UnaryOp<O> + 'static>, usize), E> {
        let power = self.parser.borrow_mut().parse_next(input)?;
        Ok((&self.op, power))
    }
}
impl<'s, BO, O, I, P, E> Parser<I, (&'s RefCell<dyn BinaryOp<O>>, usize), E> for &'s Operator<BO, P>
where
    BO: BinaryOp<O> + 'static,
    I: Stream + StreamIsPartial,
    P: Parser<I, usize, E>,
    E: ParserError<I>,
{
    #[inline(always)]
    fn parse_next(
        &mut self,
        input: &mut I,
    ) -> PResult<(&'s RefCell<dyn BinaryOp<O> + 'static>, usize), E> {
        let power = self.parser.borrow_mut().parse_next(input)?;
        Ok((&self.op, power))
    }
}

/// Ability to request a parser of the specified affix from the [`impl Parser`](Parser) object.
pub trait AsPrecedence<I: Stream, Operand: 'static, E: ParserError<I>> {
    /// Interprets a parser as a [`PrecedenceParserExt::prefix`] parser that returns an `unary
    /// predicate` [`UnaryOp`] and a `binding power` as its parsing result.
    #[inline(always)]
    fn as_prefix(&self) -> impl Parser<I, (&RefCell<dyn UnaryOp<Operand>>, usize), E> {
        fail
    }
    /// Interprets a parser as a [`PrecedenceParserExt::postfix`] parser that returns an `unary
    /// predicate` [`UnaryOp`] and a `binding power` as its parsing result.
    #[inline(always)]
    fn as_postfix(&self) -> impl Parser<I, (&RefCell<dyn UnaryOp<Operand>>, usize), E> {
        fail
    }
    /// Interprets a parser as a [`PrecedenceParserExt::infix`] parser that returns a `binary
    /// predicate` [`BinaryOp`] and a `binding power` as its parsing result.
    #[inline(always)]
    fn as_infix(&self) -> impl Parser<I, (&RefCell<dyn BinaryOp<Operand>>, usize), E> {
        fail
    }
}

impl<'s, F, O, I, P, E> Parser<I, (&'s RefCell<dyn UnaryOp<O>>, usize), E>
    for &'s Prefix<Operator<F, P>>
where
    F: UnaryOp<O> + 'static,
    I: Stream,
    P: Parser<I, usize, E>,
    E: ParserError<I>,
{
    #[inline(always)]
    fn parse_next(
        &mut self,
        input: &mut I,
    ) -> PResult<(&'s RefCell<dyn UnaryOp<O> + 'static>, usize), E> {
        let power = self.0.parser.borrow_mut().parse_next(input)?;
        Ok((&self.0.op, power))
    }
}

impl<F, O: 'static, I, P, E> AsPrecedence<I, O, E> for Prefix<Operator<F, P>>
where
    F: UnaryOp<O> + 'static,
    I: Stream + StreamIsPartial,
    P: Parser<I, usize, E>,
    E: ParserError<I>,
{
    #[inline(always)]
    fn as_prefix(&self) -> impl Parser<I, (&RefCell<dyn UnaryOp<O>>, usize), E> {
        &self.0
    }
}

impl<F, O: 'static, I, P, E> AsPrecedence<I, O, E> for Postfix<Operator<F, P>>
where
    F: UnaryOp<O> + 'static,
    I: Stream + StreamIsPartial,
    P: Parser<I, usize, E>,
    E: ParserError<I>,
{
    #[inline(always)]
    fn as_postfix(&self) -> impl Parser<I, (&RefCell<dyn UnaryOp<O>>, usize), E> {
        &self.0
    }
}
impl<F, O: 'static, I, P, E> AsPrecedence<I, O, E> for Infix<Operator<F, P>>
where
    F: BinaryOp<O> + 'static,
    I: Stream + StreamIsPartial,
    P: Parser<I, usize, E>,
    E: ParserError<I>,
{
    #[inline(always)]
    fn as_infix(&self) -> impl Parser<I, (&RefCell<dyn BinaryOp<O>>, usize), E> {
        &self.0
    }
}

macro_rules! impl_parser_for_tuple {
    () => {};
    ($head:ident $($X:ident)*) => {
        impl_parser_for_tuple!($($X)*);
        impl_parser_for_tuple!(~ $head $($X)*);
    };
    (~ $($X:ident)+) => {

        #[allow(unused_variables, non_snake_case)]
        impl<I, O: 'static, E, $($X),*> AsPrecedence<I, O, E> for ($($X,)*)
        where
            I: Stream + StreamIsPartial,
            E: ParserError<I>,
            $($X: AsPrecedence<I, O, E>),*
        {
            #[inline(always)]
            fn as_prefix<'s>(
                &'s self,
            ) -> impl Parser<I, (&'s RefCell<dyn UnaryOp<O>>, usize), E> {
                Prefix(self)
            }
            #[inline(always)]
            fn as_infix<'s>(
                &'s self,
            ) -> impl Parser<I, (&'s RefCell<dyn BinaryOp<O>>, usize), E> {
                Infix(self)
            }
            #[inline(always)]
            fn as_postfix<'s>(
                &'s self,
            ) -> impl Parser<I, (&'s RefCell<dyn UnaryOp<O>>, usize), E> {
                Postfix(self)
            }
        }

        #[allow(unused_variables, non_snake_case)]
        impl<'s, I, O: 'static, E, $($X),*> Parser<I, (&'s RefCell<dyn UnaryOp<O>>, usize), E>
            for Prefix<&'s ($($X,)*)>
        where
            I: Stream + StreamIsPartial,
            E: ParserError<I>,
            $($X: AsPrecedence<I, O, E>),*

        {
            #[inline(always)]
            fn parse_next(&mut self, input: &mut I) -> PResult<(&'s RefCell<dyn UnaryOp<O>>, usize), E> {
                let ($($X,)*) = self.0;
                alt(($($X.as_prefix(),)*)).parse_next(input)
            }
        }
        #[allow(unused_variables, non_snake_case)]
        impl<'s, I, O: 'static, E, $($X),*> Parser<I, (&'s RefCell<dyn UnaryOp<O>>, usize), E>
            for Postfix<&'s ($($X,)*)>
        where
            I: Stream + StreamIsPartial,
            E: ParserError<I>,
            $($X: AsPrecedence<I, O, E>),*
        {
            #[inline(always)]
            fn parse_next(&mut self, input: &mut I) -> PResult<(&'s RefCell<dyn UnaryOp<O>>, usize), E> {
                let ($($X,)*) = self.0;
                alt(($($X.as_postfix(),)*)).parse_next(input)
            }
        }
        #[allow(unused_variables, non_snake_case)]
        impl<'s, I, O: 'static, E, $($X),*> Parser<I, (&'s RefCell<dyn BinaryOp<O>>, usize), E>
            for Infix<&'s ($($X,)*)>
        where
            I: Stream + StreamIsPartial,
            E: ParserError<I>,
            $($X: AsPrecedence<I, O, E>),*
        {
            #[inline(always)]
            fn parse_next(&mut self, input: &mut I) -> PResult<(&'s RefCell<dyn BinaryOp<O>>, usize), E> {
                let ($($X,)*) = self.0;
                alt(($($X.as_infix(),)*)).parse_next(input)
            }
        }

    };
}

impl_parser_for_tuple!(P1 P2 P3 P4 P5 P6 P7 P8 P9 P10 P11 P12 P13 P14 P15 P16 P17 P18 P19 P20 P21);

/// Constructs an expression parser from an operand parser and operator parsers to parse an
/// arbitrary expression separated by `prefix`, `postfix`, and `infix` operators of various precedence.
///
/// This technique is powerful and recommended for parsing expressions.
///
/// The implementation uses [Pratt parsing](https://en.wikipedia.org/wiki/Operator-precedence_parser#Pratt_parsing).
/// This algorithm is similar to the [Shunting Yard](https://en.wikipedia.org/wiki/Shunting_yard_algorithm) algorithm
/// in that both are linear, both use precedence and binding power, and both serve the same purpose.
/// However, the `Shunting Yard` algorithm additionally uses `left` and `right` associativity,
/// while `Pratt` parsing only relies on binding power.
#[doc(alias = "pratt")]
#[doc(alias = "separated")]
#[doc(alias = "shunting_yard")]
#[doc(alias = "precedence_climbing")]
#[inline(always)]
pub fn precedence<I, ParseOperand, Operators, Operand: 'static, E>(
    mut parse_operand: ParseOperand,
    ops: Operators,
) -> impl Parser<I, Operand, E>
where
    Operators: AsPrecedence<I, Operand, E>,
    ParseOperand: Parser<I, Operand, E>,
    I: Stream + StreamIsPartial,
    E: ParserError<I>,
{
    trace("precedence", move |i: &mut I| {
        let result = precedence_impl(i, &mut parse_operand, &ops, 0);
        result
    })
}

// recursive function
fn precedence_impl<I, ParseOperand, Operators, Operand: 'static, E>(
    i: &mut I,
    parse_operand: &mut ParseOperand,
    ops: &Operators,
    start_power: usize,
) -> PResult<Operand, E>
where
    I: Stream + StreamIsPartial,
    Operators: AsPrecedence<I, Operand, E>,
    ParseOperand: Parser<I, Operand, E>,
    E: ParserError<I>,
{
    let operand = trace("operand", opt(parse_operand.by_ref())).parse_next(i)?;
    let mut operand = if let Some(operand) = operand {
        operand
    } else {
        // Prefix unary operators
        let len = i.eof_offset();
        let (fold_prefix, power) = trace("prefix", ops.as_prefix()).parse_next(i)?;
        // infinite loop check: the parser must always consume
        if i.eof_offset() == len {
            return Err(ErrMode::assert(i, "`prefix` parsers must always consume"));
        }
        let operand = precedence_impl(i, parse_operand, ops, power)?;
        fold_prefix.borrow_mut().fold_unary(operand)
    };

    'parse: while i.eof_offset() > 0 {
        // Postfix unary operators
        let start = i.checkpoint();
        let len = i.eof_offset();
        if let Some((fold_postfix, power)) =
            trace("postfix", opt(ops.as_postfix())).parse_next(i)?
        {
            // infinite loop check: the parser must always consume
            if i.eof_offset() == len {
                return Err(ErrMode::assert(i, "`postfix` parsers must always consume"));
            }
            if power < start_power {
                i.reset(&start);
                break;
            }
            operand = fold_postfix.borrow_mut().fold_unary(operand);

            continue 'parse;
        }

        // Infix binary operators
        let start = i.checkpoint();
        let len = i.eof_offset();
        if let Some((fold_infix, power)) = trace("infix", opt(ops.as_infix())).parse_next(i)? {
            // infinite loop check: the parser must always consume
            if i.eof_offset() == len {
                return Err(ErrMode::assert(i, "`infix` parsers must always consume"));
            }
            if power < start_power {
                i.reset(&start);
                break;
            }
            let rhs = precedence_impl(i, parse_operand, ops, power)?;
            operand = fold_infix.borrow_mut().fold_binary(operand, rhs);

            continue 'parse;
        }

        break 'parse;
    }

    Ok(operand)
}

#[cfg(test)]
mod tests {
    use crate::ascii::{digit1, space0};
    use crate::combinator::delimited;
    use crate::error::ContextError;

    use super::*;

    fn factorial(x: i32) -> i32 {
        if x == 0 {
            1
        } else {
            x * factorial(x - 1)
        }
    }
    fn parser<'i>() -> impl Parser<&'i str, i32, ContextError> {
        move |i: &mut &str| {
            precedence(
                delimited(space0, digit1.try_map(|d: &str| d.parse::<i32>()), space0),
                (
                    "-".value(2).prefix(|a: i32| -a),
                    "+".value(2).prefix(|a| a),
                    "!".value(2).postfix(factorial),
                    "+".value(0).infix(|a, b| a + b),
                    "-".value(0).infix(|a, b| a + b),
                    "*".value(1).infix(|a, b| a * b),
                    "/".value(1).infix(|a, b| a / b),
                ),
            )
            .parse_next(i)
        }
    }

    #[test]
    fn test_precedence() {
        assert_eq!(parser().parse("-3!+-3 *  4"), Ok(-18));
        assert_eq!(parser().parse("+2 + 3 *  4"), Ok(14));
        assert_eq!(parser().parse("2 * 3+4"), Ok(10));
    }
    #[test]
    fn test_unary() {
        assert_eq!(parser().parse("-2"), Ok(-2));
        assert_eq!(parser().parse("4!"), Ok(24));
        assert_eq!(parser().parse("2 + 4!"), Ok(26));
        assert_eq!(parser().parse("-2 + 2"), Ok(0));
    }
}
