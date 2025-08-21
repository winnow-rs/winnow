//! Deprecated, see [`combinator`]
#![deprecated(since = "0.4.2", note = "Replaced with `combinator`")]

use crate::combinator;

pub use combinator::dispatch;
pub use combinator::Alt;
pub use combinator::Permutation;

use crate::error::ParseError;
use crate::stream::Stream;
use crate::Parser;

/// Deprecated, see [`combinator::alt`]
#[deprecated(since = "0.4.2", note = "Replaced with `combinator::alt`")]
pub fn alt<I: Stream, O, E: ParseError<I>, List: Alt<I, O, E>>(l: List) -> impl Parser<I, O, E> {
    combinator::alt(l)
}

/// Deprecated, see [`combinator::permutation`]
#[deprecated(since = "0.4.2", note = "Replaced with `combinator::permutation`")]
pub fn permutation<I: Stream, O, E: ParseError<I>, List: Permutation<I, O, E>>(
    l: List,
) -> impl Parser<I, O, E> {
    combinator::permutation(l)
}
