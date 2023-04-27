//! Deprecated, see [`combinator`]
#![deprecated(since = "0.4.2", note = "Replaced with `combinator`")]

use crate::combinator;
use crate::error::ParseError;
use crate::stream::Stream;
use crate::*;

pub use combinator::dispatch;

pub use combinator::Alt;

/// Deprecated, replaced with [`combinator::alt`]
#[deprecated(since = "0.4.2", note = "Replaced with `combinator::alt")]
#[inline(always)]
pub fn alt<I: Stream, O, E: ParseError<I>, List: Alt<I, O, E>>(l: List) -> impl Parser<I, O, E> {
    combinator::alt(l)
}

pub use combinator::Permutation;

/// Deprecated, replaced with [`combinator::permutation`]
#[deprecated(since = "0.4.2", note = "Replaced with `combinator::permutation")]
#[inline(always)]
pub fn permutation<I: Stream, O, E: ParseError<I>, List: Permutation<I, O, E>>(
    l: List,
) -> impl Parser<I, O, E> {
    combinator::permutation(l)
}
