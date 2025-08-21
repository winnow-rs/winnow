//! Deprecated, see [`token`]
#![deprecated(since = "0.4.2", note = "Replaced with `token`")]

use crate::error::ParseError;
use crate::stream::Range;
use crate::stream::StreamIsPartial;
use crate::stream::ToUsize;
use crate::stream::{Compare, ContainsToken, FindSlice, SliceLen, Stream};
use crate::token;
use crate::IResult;
use crate::Parser;

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

/// Deprecated, see [`token::any`]
#[deprecated(since = "0.4.2", note = "Replaced with `token::any`")]
pub fn any<I, E: ParseError<I>>(input: I) -> IResult<I, <I as Stream>::Token, E>
where
    I: StreamIsPartial,
    I: Stream,
{
    crate::token::any.parse_next(input)
}

/// Deprecated, see [`token::tag`]
#[deprecated(since = "0.4.2", note = "Replaced with `token::tag`")]
pub fn tag<T, I, Error: ParseError<I>>(tag: T) -> impl Parser<I, <I as Stream>::Slice, Error>
where
    I: StreamIsPartial,
    I: Stream + Compare<T>,
    T: SliceLen + Clone,
{
    crate::token::tag(tag)
}

/// Deprecated, see [`token::tag_no_case`]
#[deprecated(since = "0.4.2", note = "Replaced with `token::tag_no_case`")]
pub fn tag_no_case<T, I, Error: ParseError<I>>(
    tag: T,
) -> impl Parser<I, <I as Stream>::Slice, Error>
where
    I: StreamIsPartial,
    I: Stream + Compare<T>,
    T: SliceLen + Clone,
{
    crate::token::tag_no_case(tag)
}

/// Deprecated, see [`token::one_of`]
#[deprecated(since = "0.4.2", note = "Replaced with `token::one_of`")]
pub fn one_of<I, T, Error: ParseError<I>>(list: T) -> impl Parser<I, <I as Stream>::Token, Error>
where
    I: StreamIsPartial,
    I: Stream,
    <I as Stream>::Token: Copy,
    T: ContainsToken<<I as Stream>::Token>,
{
    crate::token::one_of(list)
}

/// Deprecated, see [`token::none_of`]
#[deprecated(since = "0.4.2", note = "Replaced with `token::none_of`")]
pub fn none_of<I, T, Error: ParseError<I>>(list: T) -> impl Parser<I, <I as Stream>::Token, Error>
where
    I: StreamIsPartial,
    I: Stream,
    <I as Stream>::Token: Copy,
    T: ContainsToken<<I as Stream>::Token>,
{
    crate::token::none_of(list)
}

/// Deprecated, see [`token::take_while`]
#[deprecated(since = "0.4.2", note = "Replaced with `token::take_while`")]
pub fn take_while<T, I, Error: ParseError<I>>(
    range: impl Into<Range>,
    list: T,
) -> impl Parser<I, <I as Stream>::Slice, Error>
where
    I: StreamIsPartial,
    I: Stream,
    T: ContainsToken<<I as Stream>::Token>,
{
    crate::token::take_while(range, list)
}

/// Deprecated, see [`token::take_while`]
#[deprecated(since = "0.4.6", note = "Replaced with `token::take_while`")]
#[inline(always)]
pub fn take_while0<T, I, Error: ParseError<I>>(
    list: T,
) -> impl Parser<I, <I as Stream>::Slice, Error>
where
    I: StreamIsPartial,
    I: Stream,
    T: ContainsToken<<I as Stream>::Token>,
{
    crate::token::take_while(0.., list)
}

/// Deprecated, see [`token::take_while`]
#[deprecated(since = "0.4.6", note = "Replaced with `token::take_while`")]
#[inline(always)]
pub fn take_while1<T, I, Error: ParseError<I>>(
    list: T,
) -> impl Parser<I, <I as Stream>::Slice, Error>
where
    I: StreamIsPartial,
    I: Stream,
    T: ContainsToken<<I as Stream>::Token>,
{
    crate::token::take_while(1.., list)
}

/// Deprecated, see [`token::take_till0`]
#[deprecated(since = "0.4.6", note = "Replaced with `token::take_till0`")]
pub fn take_till0<T, I, Error: ParseError<I>>(
    list: T,
) -> impl Parser<I, <I as Stream>::Slice, Error>
where
    I: StreamIsPartial,
    I: Stream,
    T: ContainsToken<<I as Stream>::Token>,
{
    crate::token::take_till0(list)
}

/// Deprecated, see [`token::take_till1`]
#[deprecated(since = "0.4.6", note = "Replaced with `token::take_till1`")]
pub fn take_till1<T, I, Error: ParseError<I>>(
    list: T,
) -> impl Parser<I, <I as Stream>::Slice, Error>
where
    I: StreamIsPartial,
    I: Stream,
    T: ContainsToken<<I as Stream>::Token>,
{
    crate::token::take_till1(list)
}

/// Deprecated, see [`token::take`]
#[deprecated(since = "0.4.6", note = "Replaced with `token::take`")]
pub fn take<C, I, Error: ParseError<I>>(count: C) -> impl Parser<I, <I as Stream>::Slice, Error>
where
    I: StreamIsPartial,
    I: Stream,
    C: ToUsize,
{
    crate::token::take(count)
}

/// Deprecated, see [`token::take_until0`]
#[deprecated(since = "0.4.6", note = "Replaced with `token::take_until0`")]
pub fn take_until0<T, I, Error: ParseError<I>>(
    tag: T,
) -> impl Parser<I, <I as Stream>::Slice, Error>
where
    I: StreamIsPartial,
    I: Stream + FindSlice<T>,
    T: SliceLen + Clone,
{
    crate::token::take_until0(tag)
}

/// Deprecated, see [`token::take_until1`]
#[deprecated(since = "0.4.6", note = "Replaced with `token::take_until1`")]
pub fn take_until1<T, I, Error: ParseError<I>>(
    tag: T,
) -> impl Parser<I, <I as Stream>::Slice, Error>
where
    I: StreamIsPartial,
    I: Stream + FindSlice<T>,
    T: SliceLen + Clone,
{
    crate::token::take_until1(tag)
}
