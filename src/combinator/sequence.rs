use crate::combinator::impls::{Delimited, Preceded, SeparatedPair, Terminated, Void};
use crate::error::ParserError;
use crate::stream::Stream;
use crate::Parser;

/// Sequence two parsers, only returning the output from the second.
///
/// See also [`seq`][crate::combinator::seq] to generalize this across any number of fields.
///
/// # Example
///
/// ```rust
/// # use winnow::{error::ErrMode, error::Needed};
/// # use winnow::prelude::*;
/// # use winnow::error::Needed::Size;
/// use winnow::combinator::preceded;
///
/// fn parser<'i>(input: &mut &'i str) -> ModalResult<&'i str> {
///     preceded("abc", "efg").parse_next(input)
/// }
///
/// assert_eq!(parser.parse_peek("abcefg"), Ok(("", "efg")));
/// assert_eq!(parser.parse_peek("abcefghij"), Ok(("hij", "efg")));
/// assert!(parser.parse_peek("").is_err());
/// assert!(parser.parse_peek("123").is_err());
/// ```
#[doc(alias = "ignore_then")]
pub fn preceded<Input, Output, Error, Ignore, IgnoredParser, ParseNext>(
    ignored: IgnoredParser,
    parser: ParseNext,
) -> Preceded<Input, Output, Error, Void<IgnoredParser, Input, Ignore, Error>, ParseNext>
where
    Input: Stream,
    Error: ParserError<Input>,
    IgnoredParser: Parser<Input, Ignore, Error>,
    ParseNext: Parser<Input, Output, Error>,
{
    Preceded::<_, _, _, Void<IgnoredParser, _, _, _>, _>::new_voided(ignored, parser)
}

/// Sequence two parsers, only returning the output of the first.
///
/// See also [`seq`][crate::combinator::seq] to generalize this across any number of fields.
///
/// # Example
///
/// ```rust
/// # use winnow::{error::ErrMode, error::Needed};
/// # use winnow::prelude::*;
/// # use winnow::error::Needed::Size;
/// use winnow::combinator::terminated;
///
/// fn parser<'i>(input: &mut &'i str) -> ModalResult<&'i str> {
///     terminated("abc", "efg").parse_next(input)
/// }
///
/// assert_eq!(parser.parse_peek("abcefg"), Ok(("", "abc")));
/// assert_eq!(parser.parse_peek("abcefghij"), Ok(("hij", "abc")));
/// assert!(parser.parse_peek("").is_err());
/// assert!(parser.parse_peek("123").is_err());
/// ```
#[doc(alias = "then_ignore")]
pub fn terminated<Input, Output, Error, ParseNext, Ignore, IgnoredParser>(
    parser: ParseNext,
    ignored: IgnoredParser,
) -> Terminated<Input, Output, Error, Void<IgnoredParser, Input, Ignore, Error>, ParseNext>
where
    Input: Stream,
    Error: ParserError<Input>,
    ParseNext: Parser<Input, Output, Error>,
    IgnoredParser: Parser<Input, Ignore, Error>,
{
    Terminated::<_, _, _, Void<IgnoredParser, _, _, _>, _>::new_voided(ignored, parser)
}

/// Sequence three parsers, only returning the values of the first and third.
///
/// See also [`seq`][crate::combinator::seq] to generalize this across any number of fields.
///
/// # Example
///
/// ```rust
/// # use winnow::{error::ErrMode, error::Needed};
/// # use winnow::error::Needed::Size;
/// # use winnow::prelude::*;
/// use winnow::combinator::separated_pair;
///
/// fn parser<'i>(input: &mut &'i str) -> ModalResult<(&'i str, &'i str)> {
///     separated_pair("abc", "|", "efg").parse_next(input)
/// }
///
/// assert_eq!(parser.parse_peek("abc|efg"), Ok(("", ("abc", "efg"))));
/// assert_eq!(parser.parse_peek("abc|efghij"), Ok(("hij", ("abc", "efg"))));
/// assert!(parser.parse_peek("").is_err());
/// assert!(parser.parse_peek("123").is_err());
/// ```
pub fn separated_pair<Input, O1, O2, Error, P1, Ignore, SepParser, P2>(
    first: P1,
    sep: SepParser,
    second: P2,
) -> SeparatedPair<Input, O1, O2, Error, P1, Void<SepParser, Input, Ignore, Error>, P2>
where
    Input: Stream,
    Error: ParserError<Input>,
    P1: Parser<Input, O1, Error>,
    SepParser: Parser<Input, Ignore, Error>,
    P2: Parser<Input, O2, Error>,
{
    SeparatedPair::<_, _, _, _, _, Void<SepParser, _, _, _>, _>::new_voided(first, sep, second)
}

/// Sequence three parsers, only returning the output of the second.
///
/// See also [`seq`][crate::combinator::seq] to generalize this across any number of fields.
///
/// # Example
///
/// ```rust
/// # use winnow::{error::ErrMode, error::Needed};
/// # use winnow::error::Needed::Size;
/// # use winnow::prelude::*;
/// use winnow::combinator::delimited;
///
/// fn parser<'i>(input: &mut &'i str) -> ModalResult<&'i str> {
///     delimited("(", "abc", ")").parse_next(input)
/// }
///
/// assert_eq!(parser.parse_peek("(abc)"), Ok(("", "abc")));
/// assert_eq!(parser.parse_peek("(abc)def"), Ok(("def", "abc")));
/// assert!(parser.parse_peek("").is_err());
/// assert!(parser.parse_peek("123").is_err());
/// ```
#[doc(alias = "between")]
#[doc(alias = "padded")]
pub fn delimited<
    Input,
    Output,
    Error,
    Ignore1,
    IgnoredParser1,
    ParseNext,
    Ignore2,
    IgnoredParser2,
>(
    ignored1: IgnoredParser1,
    parser: ParseNext,
    ignored2: IgnoredParser2,
) -> Delimited<
    Input,
    Output,
    Error,
    Void<IgnoredParser1, Input, Ignore1, Error>,
    ParseNext,
    Void<IgnoredParser2, Input, Ignore2, Error>,
>
where
    Input: Stream,
    Error: ParserError<Input>,
    IgnoredParser1: Parser<Input, Ignore1, Error>,
    ParseNext: Parser<Input, Output, Error>,
    IgnoredParser2: Parser<Input, Ignore2, Error>,
{
    Delimited::<_, _, _, Void<IgnoredParser1, _, _, _>, _, Void<IgnoredParser2, _, _, _>>::new_voided(ignored1, parser, ignored2)
}
