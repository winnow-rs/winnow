use crate::{
    combinator::{opt, trace},
    error::{ErrMode, ParserError},
    stream::{Stream, StreamIsPartial},
    PResult, Parser,
};

/// Parses an expression based on operator precedence.
#[doc(alias = "pratt")]
#[doc(alias = "separated")]
#[doc(alias = "shunting_yard")]
#[doc(alias = "precedence_climbing")]
#[inline(always)]
pub fn precedence<I, ParseOperand, ParseInfix, ParsePrefix, ParsePostfix, Operand, E>(
    start_power: i64,
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
        let result = precedence_impl(
            i,
            &mut operand,
            &mut prefix,
            &mut postfix,
            &mut infix,
            start_power,
        )?;
        Ok(result)
    })
}

#[derive(Debug, Clone, Copy)]
pub enum Assoc {
    Left(i64),
    Right(i64),
    Neither(i64),
}

// recursive function
fn precedence_impl<I, ParseOperand, ParseInfix, ParsePrefix, ParsePostfix, Operand, E>(
    i: &mut I,
    parse_operand: &mut ParseOperand,
    prefix: &mut ParsePrefix,
    postfix: &mut ParsePostfix,
    infix: &mut ParseInfix,
    min_power: i64,
) -> PResult<Operand, E>
where
    I: Stream + StreamIsPartial,
    ParseOperand: Parser<I, Operand, E>,
    ParseInfix: Parser<I, (Assoc, fn(&mut I, Operand, Operand) -> PResult<Operand, E>), E>,
    ParsePrefix: Parser<I, (i64, fn(&mut I, Operand) -> PResult<Operand, E>), E>,
    ParsePostfix: Parser<I, (i64, fn(&mut I, Operand) -> PResult<Operand, E>), E>,
    E: ParserError<I>,
{
    let operand = opt(parse_operand.by_ref()).parse_next(i)?;
    let mut operand = if let Some(operand) = operand {
        operand
    } else {
        // Prefix unary operators
        let len = i.eof_offset();
        let (power, fold_prefix) = prefix.parse_next(i)?;
        // infinite loop check: the parser must always consume
        if i.eof_offset() == len {
            return Err(ErrMode::assert(i, "`prefix` parsers must always consume"));
        }
        let operand = precedence_impl(i, parse_operand, prefix, postfix, infix, power)?;
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
        if let Some((power, fold_postfix)) = opt(postfix.by_ref()).parse_next(i)? {
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
        let parse_result = opt(infix.by_ref()).parse_next(i)?;
        if let Some((assoc, fold_infix)) = parse_result {
            let mut is_neither = None;
            let (lpower, rpower) = match assoc {
                Assoc::Right(p) => (p, p - 1),
                Assoc::Left(p) => (p, p + 1),
                Assoc::Neither(p) => {
                    is_neither = Some(p);
                    (p, p + 1)
                }
            };
            if lpower < min_power || prev_op_is_neither.is_some_and(|p| lpower == p) {
                i.reset(&start);
                break 'parse;
            }
            prev_op_is_neither = is_neither;
            let rhs = precedence_impl(i, parse_operand, prefix, postfix, infix, rpower)?;
            operand = fold_infix(i, operand, rhs)?;

            continue 'parse;
        }

        break 'parse;
    }

    Ok(operand)
}

#[cfg(test)]
mod tests {
    use crate::ascii::{digit1, space0};
    use crate::combinator::{delimited, empty, fail, peek};
    use crate::dispatch;
    use crate::error::ContextError;
    use crate::token::any;

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
                0,
                trace(
                    "operand",
                    delimited(
                        space0,
                        dispatch! {peek(any);
                            '(' => delimited('(',  parser(), ')'),
                            _ => digit1.parse_to::<i32>()
                        },
                        space0,
                    ),
                ),
                trace(
                    "prefix",
                    dispatch! {any;
                        '+' => empty.value((9, (|_: &mut _, a| Ok(a)) as _)),
                        '-' => empty.value((9, (|_: &mut _, a: i32| Ok(-a)) as _)),
                        _ => fail
                    },
                ),
                trace(
                    "postfix",
                    dispatch! {any;
                        '!' => empty.value((9, (|_: &mut _, a| {Ok(factorial(a))}) as _)),
                        _ => fail
                    },
                ),
                trace(
                    "infix",
                    dispatch! {any;
                       '+' => empty.value((Assoc::Left(5), (|_: &mut _, a, b| Ok(a + b)) as _  )),
                       '-' => empty.value((Assoc::Left(5), (|_: &mut _, a, b| Ok(a - b)) as _)),
                       '*' => empty.value((Assoc::Left(7), (|_: &mut _, a, b| Ok(a * b)) as _)),
                       '/' => empty.value((Assoc::Left(7), (|_: &mut _, a, b| Ok(a / b)) as _)),
                       '%' => empty.value((Assoc::Left(7), (|_: &mut _, a, b| Ok(a % b)) as _)),
                       '^' => empty.value((Assoc::Right(9), (|_: &mut _, a, b| Ok(a ^ b)) as _)),
                       _ => fail
                    },
                ),
            )
            .parse_next(i)
        }
    }

    #[test]
    fn test_precedence() {
        // assert_eq!(parser().parse("-3!+-3 *  4"), Ok(-18));
        // assert_eq!(parser().parse("+2 + 3 *  4"), Ok(14));
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
