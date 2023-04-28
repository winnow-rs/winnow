//! Deprecated, see [`token`]
#![deprecated(since = "0.4.2", note = "Replaced with `token`")]

use crate::error::ParseError;
use crate::stream::StreamIsPartial;
use crate::stream::{ContainsToken, Stream};
use crate::token;
use crate::Parser;

pub use crate::token::*;

/// Deprecated, see [`token::take_while`]
#[deprecated(since = "0.4.2", note = "Replaced with `token::take_while`")]
#[inline(always)]
pub fn take_while_m_n<T, I, Error: ParseError<I>>(
    m: usize,
    n: usize,
    list: T,
) -> impl Parser<I, <I as Stream>::Slice, Error>
where
    I: StreamIsPartial,
    I: Stream,
    T: ContainsToken<<I as Stream>::Token>,
{
    token::take_while(m..=n, list)
}
