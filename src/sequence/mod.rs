//! Combinators applying parsers in sequence

#[cfg(test)]
mod tests;

use crate::error::ParseError;
use crate::stream::Stream;
use crate::trace::trace;
use crate::{IResult, Parser};

/// Apply two parsers, only returning the output from the second.
///
/// # Arguments
/// * `first` The opening parser.
/// * `second` The second parser to get object.
///
/// # Example
///
/// ```rust
/// # use winnow::{error::ErrMode, error::ErrorKind, error::Error, error::Needed};
/// # use winnow::error::Needed::Size;
/// use winnow::sequence::preceded;
/// use winnow::bytes::tag;
///
/// let mut parser = preceded(tag("abc"), tag("efg"));
///
/// assert_eq!(parser("abcefg"), Ok(("", "efg")));
/// assert_eq!(parser("abcefghij"), Ok(("hij", "efg")));
/// assert_eq!(parser(""), Err(ErrMode::Backtrack(Error::new("", ErrorKind::Tag))));
/// assert_eq!(parser("123"), Err(ErrMode::Backtrack(Error::new("123", ErrorKind::Tag))));
/// ```
#[doc(alias = "ignore_then")]
pub fn preceded<I, O1, O2, E: ParseError<I>, F, G>(
    mut first: F,
    mut second: G,
) -> impl FnMut(I) -> IResult<I, O2, E>
where
    I: Stream,
    F: Parser<I, O1, E>,
    G: Parser<I, O2, E>,
{
    trace("preceded", move |input: I| {
        let (input, _) = first.parse_next(input)?;
        second.parse_next(input)
    })
}

/// Apply two parsers, only returning the output of the first.
///
/// # Arguments
/// * `first` The first parser to apply.
/// * `second` The second parser to match an object.
///
/// # Example
///
/// ```rust
/// # use winnow::{error::ErrMode, error::ErrorKind, error::Error, error::Needed};
/// # use winnow::error::Needed::Size;
/// use winnow::sequence::terminated;
/// use winnow::bytes::tag;
///
/// let mut parser = terminated(tag("abc"), tag("efg"));
///
/// assert_eq!(parser("abcefg"), Ok(("", "abc")));
/// assert_eq!(parser("abcefghij"), Ok(("hij", "abc")));
/// assert_eq!(parser(""), Err(ErrMode::Backtrack(Error::new("", ErrorKind::Tag))));
/// assert_eq!(parser("123"), Err(ErrMode::Backtrack(Error::new("123", ErrorKind::Tag))));
/// ```
#[doc(alias = "then_ignore")]
pub fn terminated<I, O1, O2, E: ParseError<I>, F, G>(
    mut first: F,
    mut second: G,
) -> impl FnMut(I) -> IResult<I, O1, E>
where
    I: Stream,
    F: Parser<I, O1, E>,
    G: Parser<I, O2, E>,
{
    trace("terminated", move |input: I| {
        let (input, o1) = first.parse_next(input)?;
        second.parse_next(input).map(|(i, _)| (i, o1))
    })
}

/// Apply three parsers, only returning the values of the first and third.
///
/// # Arguments
/// * `first` The first parser to apply.
/// * `sep` The separator parser to apply.
/// * `second` The second parser to apply.
///
/// # Example
///
/// ```rust
/// # use winnow::{error::ErrMode, error::ErrorKind, error::Error, error::Needed};
/// # use winnow::error::Needed::Size;
/// use winnow::sequence::separated_pair;
/// use winnow::bytes::tag;
///
/// let mut parser = separated_pair(tag("abc"), tag("|"), tag("efg"));
///
/// assert_eq!(parser("abc|efg"), Ok(("", ("abc", "efg"))));
/// assert_eq!(parser("abc|efghij"), Ok(("hij", ("abc", "efg"))));
/// assert_eq!(parser(""), Err(ErrMode::Backtrack(Error::new("", ErrorKind::Tag))));
/// assert_eq!(parser("123"), Err(ErrMode::Backtrack(Error::new("123", ErrorKind::Tag))));
/// ```
pub fn separated_pair<I, O1, O2, O3, E: ParseError<I>, F, G, H>(
    mut first: F,
    mut sep: G,
    mut second: H,
) -> impl FnMut(I) -> IResult<I, (O1, O3), E>
where
    I: Stream,
    F: Parser<I, O1, E>,
    G: Parser<I, O2, E>,
    H: Parser<I, O3, E>,
{
    trace("separated_pair", move |input: I| {
        let (input, o1) = first.parse_next(input)?;
        let (input, _) = sep.parse_next(input)?;
        second.parse_next(input).map(|(i, o2)| (i, (o1, o2)))
    })
}

/// Apply three parsers, only returning the output of the second.
///
/// # Arguments
/// * `first` The first parser to apply and discard.
/// * `second` The second parser to apply.
/// * `third` The third parser to apply and discard.
///
/// # Example
///
/// ```rust
/// # use winnow::{error::ErrMode, error::ErrorKind, error::Error, error::Needed};
/// # use winnow::error::Needed::Size;
/// use winnow::sequence::delimited;
/// use winnow::bytes::tag;
///
/// let mut parser = delimited(tag("("), tag("abc"), tag(")"));
///
/// assert_eq!(parser("(abc)"), Ok(("", "abc")));
/// assert_eq!(parser("(abc)def"), Ok(("def", "abc")));
/// assert_eq!(parser(""), Err(ErrMode::Backtrack(Error::new("", ErrorKind::Tag))));
/// assert_eq!(parser("123"), Err(ErrMode::Backtrack(Error::new("123", ErrorKind::Tag))));
/// ```
#[doc(alias = "between")]
#[doc(alias = "padded")]
pub fn delimited<I, O1, O2, O3, E: ParseError<I>, F, G, H>(
    mut first: F,
    mut second: G,
    mut third: H,
) -> impl FnMut(I) -> IResult<I, O2, E>
where
    I: Stream,
    F: Parser<I, O1, E>,
    G: Parser<I, O2, E>,
    H: Parser<I, O3, E>,
{
    trace("delimited", move |input: I| {
        let (input, _) = first.parse_next(input)?;
        let (input, o2) = second.parse_next(input)?;
        third.parse_next(input).map(|(i, _)| (i, o2))
    })
}
