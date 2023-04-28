//! Deprecated, see [`combinator`]
#![deprecated(since = "0.4.2", note = "Replaced with `combinator`")]

use crate::binary;
use crate::combinator;
use crate::error::ParseError;
use crate::stream::Accumulate;
use crate::stream::{Stream, StreamIsPartial, ToUsize, UpdateSlice};
use crate::Parser;

/// Deprecated, replaced by [`combinator::repeat0`]
#[deprecated(since = "0.4.2", note = "Replaced with `combinator::repeat0`")]
#[inline(always)]
pub fn many0<I, O, C, E, F>(f: F) -> impl Parser<I, C, E>
where
    I: Stream,
    C: Accumulate<O>,
    F: Parser<I, O, E>,
    E: ParseError<I>,
{
    combinator::repeat0(f)
}

/// Deprecated, replaced by [`combinator::repeat1`]
#[deprecated(since = "0.4.2", note = "Replaced with `combinator::repeat1`")]
#[inline(always)]
pub fn many1<I, O, C, E, F>(f: F) -> impl Parser<I, C, E>
where
    I: Stream,
    C: Accumulate<O>,
    F: Parser<I, O, E>,
    E: ParseError<I>,
{
    combinator::repeat1(f)
}

/// Deprecated, replaced by [`combinator::repeat_till0`]
#[deprecated(since = "0.4.2", note = "Replaced with `combinator::repeat_till0`")]
#[inline(always)]
pub fn many_till0<I, O, C, P, E, F, G>(f: F, g: G) -> impl Parser<I, (C, P), E>
where
    I: Stream,
    C: Accumulate<O>,
    F: Parser<I, O, E>,
    G: Parser<I, P, E>,
    E: ParseError<I>,
{
    combinator::repeat_till0(f, g)
}

/// Deprecated, replaced by [`combinator::separated0`]
#[deprecated(since = "0.4.2", note = "Replaced with `combinator::separated0`")]
#[inline(always)]
pub fn separated0<I, O, C, O2, E, P, S>(parser: P, sep: S) -> impl Parser<I, C, E>
where
    I: Stream,
    C: Accumulate<O>,
    P: Parser<I, O, E>,
    S: Parser<I, O2, E>,
    E: ParseError<I>,
{
    combinator::separated0(parser, sep)
}

/// Deprecated, replaced by [`combinator::separated1`]
#[deprecated(since = "0.4.2", note = "Replaced with `combinator::separated1`")]
#[inline(always)]
pub fn separated1<I, O, C, O2, E, P, S>(parser: P, sep: S) -> impl Parser<I, C, E>
where
    I: Stream,
    C: Accumulate<O>,
    P: Parser<I, O, E>,
    S: Parser<I, O2, E>,
    E: ParseError<I>,
{
    combinator::separated1(parser, sep)
}

/// Deprecated, replaced by [`combinator::separated_foldl1`]
#[deprecated(since = "0.4.2", note = "Replaced with `combinator::separated_foldl1`")]
#[inline(always)]
pub fn separated_foldl1<I, O, O2, E, P, S, Op>(parser: P, sep: S, op: Op) -> impl Parser<I, O, E>
where
    I: Stream,
    P: Parser<I, O, E>,
    S: Parser<I, O2, E>,
    E: ParseError<I>,
    Op: Fn(O, O2, O) -> O,
{
    combinator::separated_foldl1(parser, sep, op)
}

/// Deprecated, replaced by [`combinator::separated_foldr1`]
#[deprecated(since = "0.4.2", note = "Replaced with `combinator::separated_foldr1`")]
#[inline(always)]
#[cfg(feature = "alloc")]
pub fn separated_foldr1<I, O, O2, E, P, S, Op>(parser: P, sep: S, op: Op) -> impl Parser<I, O, E>
where
    I: Stream,
    P: Parser<I, O, E>,
    S: Parser<I, O2, E>,
    E: ParseError<I>,
    Op: Fn(O, O2, O) -> O,
{
    combinator::separated_foldr1(parser, sep, op)
}

/// Deprecated, replaced by [`combinator::repeat_m_n`]
#[deprecated(since = "0.4.2", note = "Replaced with `combinator::repeat_m_n`")]
#[inline(always)]
pub fn many_m_n<I, O, C, E, F>(min: usize, max: usize, parse: F) -> impl Parser<I, C, E>
where
    I: Stream,
    C: Accumulate<O>,
    F: Parser<I, O, E>,
    E: ParseError<I>,
{
    combinator::repeat_m_n(min, max, parse)
}

/// Deprecated, replaced by [`combinator::count`]
#[deprecated(since = "0.4.2", note = "Replaced with `combinator::count`")]
#[inline(always)]
pub fn count<I, O, C, E, F>(f: F, count: usize) -> impl Parser<I, C, E>
where
    I: Stream,
    C: Accumulate<O>,
    F: Parser<I, O, E>,
    E: ParseError<I>,
{
    combinator::count(f, count)
}

/// Deprecated, replaced by [`combinator::fill`]
#[deprecated(since = "0.4.2", note = "Replaced with `combinator::fill`")]
#[inline(always)]
pub fn fill<'a, I, O, E, F>(f: F, buf: &'a mut [O]) -> impl Parser<I, (), E> + 'a
where
    I: Stream + 'a,
    F: Parser<I, O, E> + 'a,
    E: ParseError<I> + 'a,
{
    combinator::fill(f, buf)
}

/// Deprecated, replaced by [`combinator::fold_repeat0`]
#[deprecated(since = "0.4.2", note = "Replaced with `combinator::fold_repeat0`")]
#[inline(always)]
pub fn fold_many0<I, O, E, F, G, H, R>(f: F, init: H, g: G) -> impl Parser<I, R, E>
where
    I: Stream,
    F: Parser<I, O, E>,
    G: FnMut(R, O) -> R,
    H: FnMut() -> R,
    E: ParseError<I>,
{
    combinator::fold_repeat0(f, init, g)
}

/// Deprecated, replaced by [`combinator::fold_repeat1`]
#[deprecated(since = "0.4.2", note = "Replaced with `combinator::fold_repeat1`")]
#[inline(always)]
pub fn fold_many1<I, O, E, F, G, H, R>(f: F, init: H, g: G) -> impl Parser<I, R, E>
where
    I: Stream,
    F: Parser<I, O, E>,
    G: FnMut(R, O) -> R,
    H: FnMut() -> R,
    E: ParseError<I>,
{
    combinator::fold_repeat1(f, init, g)
}

/// Deprecated, replaced by [`combinator::fold_repeat_m_n`]
#[deprecated(since = "0.4.2", note = "Replaced with `combinator::fold_repeat_m_n`")]
#[inline(always)]
pub fn fold_many_m_n<I, O, E, F, G, H, R>(
    min: usize,
    max: usize,
    parse: F,
    init: H,
    fold: G,
) -> impl Parser<I, R, E>
where
    I: Stream,
    F: Parser<I, O, E>,
    G: FnMut(R, O) -> R,
    H: FnMut() -> R,
    E: ParseError<I>,
{
    combinator::fold_repeat_m_n(min, max, parse, init, fold)
}

/// Deprecated, replaced by [`binary::length_data`]
#[deprecated(since = "0.4.2", note = "Replaced with `binary::length_data`")]
#[inline(always)]
pub fn length_data<I, N, E, F>(f: F) -> impl Parser<I, <I as Stream>::Slice, E>
where
    I: StreamIsPartial,
    I: Stream,
    N: ToUsize,
    F: Parser<I, N, E>,
    E: ParseError<I>,
{
    binary::length_data(f)
}

/// Deprecated, replaced by [`binary::length_value`]
#[deprecated(since = "0.4.2", note = "Replaced with `binary::length_value`")]
#[inline(always)]
pub fn length_value<I, O, N, E, F, G>(f: F, g: G) -> impl Parser<I, O, E>
where
    I: StreamIsPartial,
    I: Stream + UpdateSlice,
    N: ToUsize,
    F: Parser<I, N, E>,
    G: Parser<I, O, E>,
    E: ParseError<I>,
{
    binary::length_value(f, g)
}

/// Deprecated, replaced by [`binary::length_count`]
#[deprecated(since = "0.4.2", note = "Replaced with `binary::length_count`")]
#[inline(always)]
pub fn length_count<I, O, C, N, E, F, G>(f: F, g: G) -> impl Parser<I, C, E>
where
    I: Stream,
    N: ToUsize,
    C: Accumulate<O>,
    F: Parser<I, N, E>,
    G: Parser<I, O, E>,
    E: ParseError<I>,
{
    binary::length_count(f, g)
}
