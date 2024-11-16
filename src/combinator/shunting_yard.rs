use crate::combinator::opt;
use crate::error::{ErrMode, ErrorKind, ParserError};
use crate::stream::{Stream, StreamIsPartial};
use crate::{PResult, Parser};

use super::trace;

pub fn precedence<'i, I, ParseOperand, ParseInfix, ParsePrefix, ParsePostfix, Operand: 'static, E>(
    mut operand: ParseOperand,
    mut prefix: ParsePrefix,
    mut postfix: ParsePostfix,
    mut infix: ParseInfix,
) -> impl Parser<I, Operand, E>
where
    I: Stream + StreamIsPartial,
    ParseOperand: Parser<I, Operand, E>,
    ParseInfix: Parser<I, (usize, usize, &'i dyn Fn(Operand, Operand) -> Operand), E>,
    ParsePrefix: Parser<I, (usize, &'i dyn Fn(Operand) -> Operand), E>,
    ParsePostfix: Parser<I, (usize, &'i dyn Fn(Operand) -> Operand), E>,
    E: ParserError<I>,
{
    trace("precedence", move |i: &mut I| {
        let result = shunting_yard(
            i,
            operand.by_ref(),
            prefix.by_ref(),
            postfix.by_ref(),
            infix.by_ref(),
        )?;
        Ok(result)
    })
}

fn shunting_yard<'i, I, ParseOperand, ParseInfix, ParsePrefix, ParsePostfix, Operand: 'static, E>(
    i: &mut I,
    mut operand: ParseOperand,
    mut prefix: ParsePrefix,
    mut postfix: ParsePostfix,
    mut infix: ParseInfix,
) -> PResult<Operand, E>
where
    I: Stream + StreamIsPartial,
    ParseOperand: Parser<I, Operand, E>,
    ParseInfix: Parser<I, (usize, usize, &'i dyn Fn(Operand, Operand) -> Operand), E>,
    ParsePrefix: Parser<I, (usize, &'i dyn Fn(Operand) -> Operand), E>,
    ParsePostfix: Parser<I, (usize, &'i dyn Fn(Operand) -> Operand), E>,
    E: ParserError<I>,
{
    // what we expecting to parse next
    let mut waiting_operand = true;
    // a stack for computing the result
    let mut value_stack = Vec::<Operand>::new();
    let mut operator_stack = Vec::<Operator<'_, Operand>>::new();

    'parse: loop {
        // operands and prefixes
        if waiting_operand {
            if let Some(operand) = opt(operand.by_ref()).parse_next(i)? {
                value_stack.push(operand);
                waiting_operand = false;
                continue 'parse;
            }

            // prefix operators never trigger the evaluation of pending operators
            if let Some((lpower, op)) = opt(prefix.by_ref()).parse_next(i)? {
                operator_stack.push(Operator::Unary(lpower, op));
                continue 'parse;
            }

            // error missing operand
            return Err(ErrMode::from_error_kind(i, ErrorKind::Token));
        } else {
            if i.eof_offset() == 0 {
                break 'parse;
            }

            // Postfix unary operators
            if let Some((lpower, op)) = opt(postfix.by_ref()).parse_next(i)? {
                unwind_operators_stack_to(lpower, &mut value_stack, &mut operator_stack);

                // postfix operators are never put in pending state in `operator_stack`
                // TODO: confirm that `expect` is valid for all invariants
                let lhs = value_stack.pop().expect("value");
                value_stack.push(op(lhs));
                continue 'parse;
            }

            // Infix binary operators
            if let Some((lpower, rpower, op)) = opt(infix.by_ref()).parse_next(i)? {
                unwind_operators_stack_to(lpower, &mut value_stack, &mut operator_stack);
                operator_stack.push(Operator::Binary(lpower, rpower, op));
                waiting_operand = true;
                continue 'parse;
            }

            // no more operators
            break 'parse;
        }
    }

    while let Some(op) = operator_stack.pop() {
        evaluate(&mut value_stack, op);
    }
    // TODO: when it can happen?
    // if eval_stack.len() > 1 {
    //     // Error: value left on stack
    // }

    Ok(value_stack.pop().expect("well-formed expression")) // TODO: error handling
}

enum Operator<'i, Operand> {
    // left binding power for the postfix or the right one for the prefix
    Unary(usize, &'i dyn Fn(Operand) -> Operand),
    // left binding power and right binding power for the infix operator
    Binary(usize, usize, &'i dyn Fn(Operand, Operand) -> Operand),
}

impl<O> Operator<'_, O> {
    fn right_power(&self) -> usize {
        match self {
            Operator::Unary(p, _) => *p,
            Operator::Binary(_, p, _) => *p,
        }
    }
}

fn evaluate<Operand>(stack: &mut Vec<Operand>, op: Operator<'_, Operand>) {
    match op {
        Operator::Unary(_, op) => {
            let lhs = stack.pop().expect("value");
            stack.push(op(lhs));
        }
        Operator::Binary(_, _, op) => {
            // TODO: confirm invariants. It should be already checked by the parser algorithm itself
            let rhs = stack.pop().expect("value");
            let lhs = stack.pop().expect("value");
            let folded = op(lhs, rhs);
            stack.push(folded);
        }
    };
}

fn unwind_operators_stack_to<Operand>(
    current_left_power: usize,
    value_stack: &mut Vec<Operand>,
    operator_stack: &mut Vec<Operator<'_, Operand>>,
) {
    while operator_stack
        .last()
        .is_some_and(|op| op.right_power() > current_left_power)
    {
        evaluate(value_stack, operator_stack.pop().expect("already checked"));
    }
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
                    '+' => trace("+", empty).value((9, (&|a| a) as _)),
                    '-' => trace("-", empty).value((9, (&|a: i32| -a) as _)),
                    _ => fail
                },
            ),
            trace("postfix", fail),
            trace(
                "infix",
                dispatch! {any;
                   '+' => trace("+", empty).value((5, 6, (&|a, b| {
                        println!("({a} + {b})");
                        a + b
                    }) as _)),
                   '-' => trace("-", empty).value((5, 6, (&|a, b| {
                        println!("({a} - {b})");
                        a - b
                    }) as _)),
                   '*' => trace("*", empty).value((7, 8, (&|a, b|{
                        println!("({a} * {b})");
                        a * b
                    }) as _)),
                   '/' => trace("/", empty).value((7, 8, (&|a, b| {
                        println!("({a} / {b})");
                        a / b
                    }) as _)),
                   _ => fail
                },
            ),
        )
        .parse_next(i)
    }

    #[test]
    fn test_parser() {
        assert_eq!(parser.parse("1+4+6"), Ok(11));
        assert_eq!(parser.parse("2*(4+6)"), Ok(20));
        assert!(matches!(parser.parse("2*"), Err(_)));
    }
}
