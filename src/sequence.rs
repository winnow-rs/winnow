//! Deprecated, see [`combinator`]
#![deprecated(since = "0.4.2", note = "Replaced with `combinator`")]

use crate::combinator;
use crate::error::ParseError;
use crate::stream::Stream;
use crate::Parser;

/// Deprecated, replaced with [`combinator::preceded`]
#[deprecated(since = "0.4.2", note = "Replaced with `combinator::preceded")]
#[inline(always)]
pub fn preceded<I, O1, O2, E: ParseError<I>, F, G>(first: F, second: G) -> impl Parser<I, O2, E>
where
    I: Stream,
    F: Parser<I, O1, E>,
    G: Parser<I, O2, E>,
{
    combinator::preceded(first, second)
}

/// Deprecated, replaced with [`combinator::terminated`]
#[deprecated(since = "0.4.2", note = "Replaced with `combinator::terminated")]
#[inline(always)]
pub fn terminated<I, O1, O2, E: ParseError<I>, F, G>(first: F, second: G) -> impl Parser<I, O1, E>
where
    I: Stream,
    F: Parser<I, O1, E>,
    G: Parser<I, O2, E>,
{
    combinator::terminated(first, second)
}

/// Deprecated, replaced with [`combinator::separated_pair`]
#[deprecated(since = "0.4.2", note = "Replaced with `combinator::separated_pair")]
#[inline(always)]
pub fn separated_pair<I, O1, O2, O3, E: ParseError<I>, F, G, H>(
    first: F,
    sep: G,
    second: H,
) -> impl Parser<I, (O1, O3), E>
where
    I: Stream,
    F: Parser<I, O1, E>,
    G: Parser<I, O2, E>,
    H: Parser<I, O3, E>,
{
    combinator::separated_pair(first, sep, second)
}

/// Deprecated, replaced with [`combinator::delimited`]
#[deprecated(since = "0.4.2", note = "Replaced with `combinator::delimited")]
#[inline(always)]
pub fn delimited<I, O1, O2, O3, E: ParseError<I>, F, G, H>(
    first: F,
    second: G,
    third: H,
) -> impl Parser<I, O2, E>
where
    I: Stream,
    F: Parser<I, O1, E>,
    G: Parser<I, O2, E>,
    H: Parser<I, O3, E>,
{
    combinator::delimited(first, second, third)
}
