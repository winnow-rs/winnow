//! Deprecated, see [`combinator`]
#![deprecated(since = "0.4.2", note = "Replaced with `combinator`")]

use crate::binary;
use crate::combinator;
use crate::error::ParseError;
use crate::stream::Accumulate;
use crate::stream::Stream;
use crate::Parser;

/// Deprecated, replaced by [`combinator::repeat`]
#[deprecated(since = "0.4.2", note = "Replaced with `combinator::repeat`")]
#[inline(always)]
pub fn many0<I, O, C, E, F>(f: F) -> impl Parser<I, C, E>
where
    I: Stream,
    C: Accumulate<O>,
    F: Parser<I, O, E>,
    E: ParseError<I>,
{
    combinator::repeat(0.., f)
}

/// Deprecated, replaced by [`combinator::repeat`]
#[deprecated(since = "0.4.2", note = "Replaced with `combinator::repeat`")]
#[inline(always)]
pub fn many1<I, O, C, E, F>(f: F) -> impl Parser<I, C, E>
where
    I: Stream,
    C: Accumulate<O>,
    F: Parser<I, O, E>,
    E: ParseError<I>,
{
    combinator::repeat(1.., f)
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

pub use combinator::separated0;
pub use combinator::separated1;
pub use combinator::separated_foldl1;
#[cfg(feature = "alloc")]
pub use combinator::separated_foldr1;

/// Deprecated, replaced by [`combinator::repeat`]
#[deprecated(since = "0.4.2", note = "Replaced with `combinator::repeat`")]
#[inline(always)]
pub fn many_m_n<I, O, C, E, F>(min: usize, max: usize, parse: F) -> impl Parser<I, C, E>
where
    I: Stream,
    C: Accumulate<O>,
    F: Parser<I, O, E>,
    E: ParseError<I>,
{
    combinator::repeat(min..=max, parse)
}

#[allow(deprecated)]
pub use combinator::count;
pub use combinator::fill;

/// Deprecated, replaced by [`combinator::fold_repeat`]
#[deprecated(since = "0.4.2", note = "Replaced with `combinator::fold_repeat`")]
#[inline(always)]
pub fn fold_many0<I, O, E, F, G, H, R>(f: F, init: H, g: G) -> impl Parser<I, R, E>
where
    I: Stream,
    F: Parser<I, O, E>,
    G: FnMut(R, O) -> R,
    H: FnMut() -> R,
    E: ParseError<I>,
{
    combinator::fold_repeat(0.., f, init, g)
}

/// Deprecated, replaced by [`combinator::fold_repeat`]
#[deprecated(since = "0.4.2", note = "Replaced with `combinator::fold_repeat`")]
#[inline(always)]
pub fn fold_many1<I, O, E, F, G, H, R>(f: F, init: H, g: G) -> impl Parser<I, R, E>
where
    I: Stream,
    F: Parser<I, O, E>,
    G: FnMut(R, O) -> R,
    H: FnMut() -> R,
    E: ParseError<I>,
{
    combinator::fold_repeat(1.., f, init, g)
}

/// Deprecated, replaced by [`combinator::fold_repeat`]
#[deprecated(since = "0.4.2", note = "Replaced with `combinator::fold_repeat`")]
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
    combinator::fold_repeat(min..=max, parse, init, fold)
}

pub use binary::length_count;
pub use binary::length_data;
pub use binary::length_value;
