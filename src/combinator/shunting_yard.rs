use crate::combinator::opt;
use crate::error::{ErrMode, ErrorKind, ParserError};
use crate::stream::{Stream, StreamIsPartial};
use crate::{PResult, Parser};

use super::precedence::Assoc;
use super::trace;

pub fn precedence<I, ParseOperand, ParseInfix, ParsePrefix, ParsePostfix, Operand, E>(
    start_precedence: i64,
    mut operand: ParseOperand,
    mut prefix: ParsePrefix,
    mut postfix: ParsePostfix,
    mut infix: ParseInfix,
) -> impl Parser<I, Operand, E>
where
    I: Stream + StreamIsPartial,
    ParseOperand: Parser<I, Operand, E>,
    ParseInfix: Parser<I, (Assoc, fn(&mut I, Operand, Operand) -> PResult<Operand, E>), E>,
    ParsePrefix: Parser<I, (i64, fn(&mut I, Operand) -> PResult<Operand, E>), E>,
    ParsePostfix: Parser<I, (i64, fn(&mut I, Operand) -> PResult<Operand, E>), E>,
    E: ParserError<I>,
{
    trace("precedence", move |i: &mut I| {
        let result = shunting_yard(
            start_precedence,
            i,
            operand.by_ref(),
            prefix.by_ref(),
            postfix.by_ref(),
            infix.by_ref(),
        )?;
        Ok(result)
    })
}

fn shunting_yard<I, ParseOperand, ParseInfix, ParsePrefix, ParsePostfix, Operand, E>(
    start_precedence: i64,
    i: &mut I,
    mut operand: ParseOperand,
    mut prefix: ParsePrefix,
    mut postfix: ParsePostfix,
    mut infix: ParseInfix,
) -> PResult<Operand, E>
where
    I: Stream + StreamIsPartial,
    ParseOperand: Parser<I, Operand, E>,
    ParseInfix: Parser<I, (Assoc, fn(&mut I, Operand, Operand) -> PResult<Operand, E>), E>,
    ParsePrefix: Parser<I, (i64, fn(&mut I, Operand) -> PResult<Operand, E>), E>,
    ParsePostfix: Parser<I, (i64, fn(&mut I, Operand) -> PResult<Operand, E>), E>,
    E: ParserError<I>,
{
    // a stack for computing the result
    let mut value_stack = Vec::<Operand>::new();
    let mut operator_stack = Vec::<Operator<I, Operand, E>>::new();

    let mut current_is_neither = None;
    'parse: loop {
        // Prefix unary operators
        while let Some((lpower, op)) = opt(prefix.by_ref()).parse_next(i)? {
            // prefix operators never trigger the evaluation of pending operators
            operator_stack.push(Operator::Unary(lpower, op));
        }

        // Operand
        if let Some(operand) = opt(operand.by_ref()).parse_next(i)? {
            value_stack.push(operand);
        } else {
            // error missing operand
            return Err(ErrMode::from_error_kind(i, ErrorKind::Token));
        }

        if i.eof_offset() <= 0 {
            break 'parse;
        }

        // Postfix unary operators
        while let Some((lpower, op)) = opt(postfix.by_ref()).parse_next(i)? {
            while operator_stack.last().is_some_and(|op| {
                let rpower = op.right_power();
                lpower < rpower
            }) {
                evaluate(
                    i,
                    &mut value_stack,
                    operator_stack.pop().expect("already checked"),
                )?;
            }
            // postfix operators are never put in pending state in `operator_stack`
            let lhs = value_stack.pop().expect("value");
            value_stack.push(op(i, lhs)?);
        }
        let start = i.checkpoint();
        // Infix binary operators
        if let Some((assoc, op)) = opt(infix.by_ref()).parse_next(i)? {
            let mut next_is_neither = None;
            let lpower = match assoc {
                Assoc::Left(p) => p,
                Assoc::Right(p) => p,
                Assoc::Neither(p) => {
                    next_is_neither = Some(p);
                    p
                }
            };
            if current_is_neither.is_some_and(|n| n == lpower) {
                i.reset(&start);
                break 'parse;
            }

            while operator_stack.last().is_some_and(|op| {
                let rpower = op.right_power();
                lpower < rpower
            }) {
                evaluate(
                    i,
                    &mut value_stack,
                    operator_stack.pop().expect("already checked"),
                )?;
            }
            current_is_neither = next_is_neither;
            // some hackery around `a ? b : c, end` -> `(, (? a b c) end)`
            // needs refactoring
            if start_precedence <= lpower {
                operator_stack.push(Operator::Binary(assoc, op));
            } else {
                i.reset(&start);
                break 'parse;
            }
        } else {
            // no more operators
            break 'parse;
        }
    }

    while let Some(op) = operator_stack.pop() {
        evaluate(i, &mut value_stack, op)?;
    }
    // TODO: when it could happen?
    // if eval_stack.len() > 1 {
    //     // Error: value left on stack
    // }

    Ok(value_stack.pop().expect("well-formed expression")) // TODO: error handling
}

enum Operator<I, Operand, E> {
    // left binding power for the postfix or the right one for the prefix
    Unary(i64, fn(&mut I, Operand) -> PResult<Operand, E>),
    // left binding power and right binding power for the infix operator
    Binary(Assoc, fn(&mut I, Operand, Operand) -> PResult<Operand, E>),
}

impl<I, O, E> Operator<I, O, E> {
    fn right_power(&self) -> i64 {
        match self {
            Operator::Unary(p, _) => *p,
            Operator::Binary(Assoc::Left(p), _) => *p + 1,
            Operator::Binary(Assoc::Right(p), _) => *p - 1,
            Operator::Binary(Assoc::Neither(p), _) => *p + 1,
        }
    }
}

fn evaluate<I, Operand, E>(
    i: &mut I,
    stack: &mut Vec<Operand>,
    op: Operator<I, Operand, E>,
) -> PResult<(), E> {
    match op {
        Operator::Unary(_, op) => {
            let lhs = stack.pop().expect("value");
            stack.push(op(i, lhs)?);
        }
        Operator::Binary(_, op) => {
            // TODO: confirm invariants. It should be already checked by the parser algorithm itself
            let rhs = stack.pop().expect("value");
            let lhs = stack.pop().expect("value");
            let folded = op(i, lhs, rhs)?;
            stack.push(folded);
        }
    };
    Ok(())
}

fn unwind_operators_stack_to<I, Operand, E>(
    i: &mut I,
    start_precedence: i64,
    current_power: Assoc,
    value_stack: &mut Vec<Operand>,
    operator_stack: &mut Vec<Operator<I, Operand, E>>,
) -> PResult<(), E> {
    let mut current_is_neither = None;
    while operator_stack.last().is_some_and(|op| {
        let rpower = op.right_power();
        let mut next_is_neither = None;
        let lpower = match current_power {
            Assoc::Left(p) => p,
            Assoc::Right(p) => p,
            Assoc::Neither(p) => {
                next_is_neither = Some(p);
                p
            }
        };
        dbg!(
            lpower,
            rpower,
            start_precedence,
            current_is_neither,
            next_is_neither
        );
        let r = lpower < rpower
            && lpower < start_precedence
            && current_is_neither.is_none_or(|n| n != lpower);
        current_is_neither = next_is_neither;
        r
    }) {
        evaluate(
            i,
            value_stack,
            operator_stack.pop().expect("already checked"),
        )?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::{
        ascii::digit1,
        combinator::{cut_err, delimited, empty, fail, peek},
        dispatch,
        token::any,
    };

    use super::*;

    fn parser(i: &mut &str) -> PResult<i32> {
        precedence(
            0,
            trace(
                "operand",
                dispatch! {peek(any);
                    '(' => delimited('(', trace("recursion",  parser), cut_err(')')),
                    _ => digit1.parse_to::<i32>()
                },
            ),
            trace(
                "prefix",
                dispatch! {any;
                    '+' => trace("+", empty).value((20, (|_: &mut _, a| Ok(a)) as _)),
                    '-' => trace("-", empty).value((20, (|_: &mut _,a: i32| Ok(-a)) as _)),
                    _ => fail
                },
            ),
            trace("postfix", fail),
            trace(
                "infix",
                dispatch! {any;
                   '+' => trace("+", empty).value((Assoc::Left(5), (|_: &mut _, a, b| {
                        println!("({a} + {b})");
                        Ok(a + b)
                    }) as _)),
                   '-' => trace("-", empty).value((Assoc::Left(5), (|_: &mut _, a, b| {
                        println!("({a} - {b})");
                        Ok(a - b)
                    }) as _)),
                   '*' => trace("*", empty).value((Assoc::Left(7), (|_: &mut _, a, b|{
                        println!("({a} * {b})");
                        Ok(a * b)
                    }) as _)),
                   '/' => trace("/", empty).value((Assoc::Left(7), (|_: &mut _, a, b| {
                        println!("({a} / {b})");
                        Ok(a / b)
                    }) as _)),
                   _ => fail
                },
            ),
        )
        .parse_next(i)
    }

    #[test]
    fn test_parser() {
        // assert_eq!(parser.parse("1==2==3"), Ok(11));
        assert_eq!(parser.parse("1+4+6"), Ok(11));
        // assert_eq!(parser.parse("2*(4+6)"), Ok(20));
        // assert!(matches!(parser.parse("2*"), Err(_)));
    }
}
