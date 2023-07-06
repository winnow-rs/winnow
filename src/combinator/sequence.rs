use crate::error::ParseError;
use crate::stream::Stream;
use crate::trace::trace;
use crate::*;

/// Sequence two parsers, only returning the output from the second.
///
/// # Arguments
/// * `first` The opening parser.
/// * `second` The second parser to get object.
///
/// # Example
///
/// ```rust
/// # use winnow::{error::ErrMode, error::ErrorKind, error::Error, error::Needed};
/// # use winnow::prelude::*;
/// # use winnow::error::Needed::Size;
/// use winnow::combinator::preceded;
/// use winnow::token::tag;
///
/// let mut parser = preceded("abc", "efg");
///
/// assert_eq!(parser.parse_peek("abcefg"), Ok(("", "efg")));
/// assert_eq!(parser.parse_peek("abcefghij"), Ok(("hij", "efg")));
/// assert_eq!(parser.parse_peek(""), Err(ErrMode::Backtrack(Error::new("", ErrorKind::Tag))));
/// assert_eq!(parser.parse_peek("123"), Err(ErrMode::Backtrack(Error::new("123", ErrorKind::Tag))));
/// ```
#[doc(alias = "ignore_then")]
pub fn preceded<I, O1, O2, E: ParseError<I>, F, G>(
    mut first: F,
    mut second: G,
) -> impl Parser<I, O2, E>
where
    I: Stream,
    F: Parser<I, O1, E>,
    G: Parser<I, O2, E>,
{
    trace(
        "preceded",
        unpeek(move |input: I| {
            let (input, _) = first.parse_peek(input)?;
            second.parse_peek(input)
        }),
    )
}

/// Sequence two parsers, only returning the output of the first.
///
/// # Arguments
/// * `first` The first parser to apply.
/// * `second` The second parser to match an object.
///
/// # Example
///
/// ```rust
/// # use winnow::{error::ErrMode, error::ErrorKind, error::Error, error::Needed};
/// # use winnow::prelude::*;
/// # use winnow::error::Needed::Size;
/// use winnow::combinator::terminated;
/// use winnow::token::tag;
///
/// let mut parser = terminated("abc", "efg");
///
/// assert_eq!(parser.parse_peek("abcefg"), Ok(("", "abc")));
/// assert_eq!(parser.parse_peek("abcefghij"), Ok(("hij", "abc")));
/// assert_eq!(parser.parse_peek(""), Err(ErrMode::Backtrack(Error::new("", ErrorKind::Tag))));
/// assert_eq!(parser.parse_peek("123"), Err(ErrMode::Backtrack(Error::new("123", ErrorKind::Tag))));
/// ```
#[doc(alias = "then_ignore")]
pub fn terminated<I, O1, O2, E: ParseError<I>, F, G>(
    mut first: F,
    mut second: G,
) -> impl Parser<I, O1, E>
where
    I: Stream,
    F: Parser<I, O1, E>,
    G: Parser<I, O2, E>,
{
    trace(
        "terminated",
        unpeek(move |input: I| {
            let (input, o1) = first.parse_peek(input)?;
            second.parse_peek(input).map(|(i, _)| (i, o1))
        }),
    )
}

/// Sequence three parsers, only returning the values of the first and third.
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
/// # use winnow::prelude::*;
/// use winnow::combinator::separated_pair;
/// use winnow::token::tag;
///
/// let mut parser = separated_pair("abc", "|", "efg");
///
/// assert_eq!(parser.parse_peek("abc|efg"), Ok(("", ("abc", "efg"))));
/// assert_eq!(parser.parse_peek("abc|efghij"), Ok(("hij", ("abc", "efg"))));
/// assert_eq!(parser.parse_peek(""), Err(ErrMode::Backtrack(Error::new("", ErrorKind::Tag))));
/// assert_eq!(parser.parse_peek("123"), Err(ErrMode::Backtrack(Error::new("123", ErrorKind::Tag))));
/// ```
pub fn separated_pair<I, O1, O2, O3, E: ParseError<I>, F, G, H>(
    mut first: F,
    mut sep: G,
    mut second: H,
) -> impl Parser<I, (O1, O3), E>
where
    I: Stream,
    F: Parser<I, O1, E>,
    G: Parser<I, O2, E>,
    H: Parser<I, O3, E>,
{
    trace(
        "separated_pair",
        unpeek(move |input: I| {
            let (input, o1) = first.parse_peek(input)?;
            let (input, _) = sep.parse_peek(input)?;
            second.parse_peek(input).map(|(i, o2)| (i, (o1, o2)))
        }),
    )
}

/// Sequence three parsers, only returning the output of the second.
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
/// # use winnow::prelude::*;
/// use winnow::combinator::delimited;
/// use winnow::token::tag;
///
/// let mut parser = delimited("(", "abc", ")");
///
/// assert_eq!(parser.parse_peek("(abc)"), Ok(("", "abc")));
/// assert_eq!(parser.parse_peek("(abc)def"), Ok(("def", "abc")));
/// assert_eq!(parser.parse_peek(""), Err(ErrMode::Backtrack(Error::new("", ErrorKind::Tag))));
/// assert_eq!(parser.parse_peek("123"), Err(ErrMode::Backtrack(Error::new("123", ErrorKind::Tag))));
/// ```
#[doc(alias = "between")]
#[doc(alias = "padded")]
pub fn delimited<I, O1, O2, O3, E: ParseError<I>, F, G, H>(
    mut first: F,
    mut second: G,
    mut third: H,
) -> impl Parser<I, O2, E>
where
    I: Stream,
    F: Parser<I, O1, E>,
    G: Parser<I, O2, E>,
    H: Parser<I, O3, E>,
{
    trace(
        "delimited",
        unpeek(move |input: I| {
            let (input, _) = first.parse_peek(input)?;
            let (input, o2) = second.parse_peek(input)?;
            third.parse_peek(input).map(|(i, _)| (i, o2))
        }),
    )
}
