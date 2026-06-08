use crate::combinator::impls::{BacktrackErr, Cond, CutErr, Not, Opt, Peek, Void};
use crate::combinator::trace;
use crate::error::{ModalError, ParserError};
use crate::stream::Stream;
use crate::{Parser, Result};

/// Apply a [`Parser`], producing `None` on [`ErrMode::Backtrack`][crate::error::ErrMode::Backtrack].
///
/// To chain an error up, see [`cut_err`].
///
/// # Example
///
/// ```rust
/// # #[cfg(feature = "ascii")] {
/// # use winnow::prelude::*;
/// use winnow::combinator::opt;
/// use winnow::ascii::alpha1;
///
/// fn parser<'i>(i: &mut &'i str) -> ModalResult<Option<&'i str>> {
///   opt(alpha1).parse_next(i)
/// }
///
/// assert_eq!(parser.parse_peek("abcd;"), Ok((";", Some("abcd"))));
/// assert_eq!(parser.parse_peek("123;"), Ok(("123;", None)));
/// # }
/// ```
pub fn opt<Input, Output, Error, ParseNext>(
    parser: ParseNext,
) -> Opt<Input, Output, Error, ParseNext>
where
    Input: Stream,
    ParseNext: Parser<Input, Output, Error>,
    Error: ParserError<Input>,
{
    Opt::new(parser)
}

/// Calls the parser if the condition is met.
///
/// # Example
///
/// ```rust
/// # #[cfg(feature = "ascii")] {
/// # use winnow::prelude::*;
/// # use winnow::combinator::opt;
/// use winnow::combinator::cond;
/// use winnow::ascii::alpha1;
///
/// fn parser<'i>(i: &mut &'i str) -> ModalResult<Option<&'i str>> {
///   let prefix = opt("-").parse_next(i)?;
///   let condition = prefix.is_some();
///   cond(condition, alpha1).parse_next(i)
/// }
///
/// assert_eq!(parser.parse_peek("-abcd;"), Ok((";", Some("abcd"))));
/// assert_eq!(parser.parse_peek("abcd;"), Ok(("abcd;", None)));
/// assert!(parser.parse_peek("-123;").is_err());
/// assert_eq!(parser.parse_peek("123;"), Ok(("123;", None)));
/// # }
/// ```
pub fn cond<Input, Output, Error, ParseNext>(
    cond: bool,
    parser: ParseNext,
) -> Cond<Input, Output, Error, ParseNext>
where
    Input: Stream,
    ParseNext: Parser<Input, Output, Error>,
    Error: ParserError<Input>,
{
    Cond::new(cond, parser)
}

/// Apply the parser without advancing the input.
///
/// To lookahead and only advance on success, see [`opt`].
///
/// # Example
///
/// ```rust
/// # #[cfg(feature = "ascii")] {
/// # use winnow::prelude::*;
/// use winnow::combinator::peek;
/// use winnow::ascii::alpha1;
///
/// fn parser<'i>(input: &mut &'i str) -> ModalResult<&'i str> {
///     peek(alpha1).parse_next(input)
/// }
///
/// assert_eq!(parser.parse_peek("abcd;"), Ok(("abcd;", "abcd")));
/// assert!(parser.parse_peek("123;").is_err());
/// # }
/// ```
#[doc(alias = "look_ahead")]
#[doc(alias = "rewind")]
pub fn peek<Input, Output, Error, ParseNext>(
    parser: ParseNext,
) -> Peek<Input, Output, Error, ParseNext>
where
    Input: Stream,
    Error: ParserError<Input>,
    ParseNext: Parser<Input, Output, Error>,
{
    Peek::new(parser)
}

/// Match the end of the [`Stream`]
///
/// Otherwise, it will error.
///
/// # Effective Signature
///
/// Assuming you are parsing a `&str` [Stream]:
/// ```rust
/// # use winnow::prelude::*;;
/// pub fn eof<'i>(input: &mut &'i str) -> ModalResult<&'i str>
/// # {
/// #     winnow::combinator::eof.parse_next(input)
/// # }
/// ```
///
/// # Example
///
/// ```rust
/// # use std::str;
/// # use winnow::combinator::eof;
/// # use winnow::prelude::*;
///
/// fn parser<'i>(input: &mut &'i str) -> ModalResult<&'i str> {
///     eof.parse_next(input)
/// }
/// assert!(parser.parse_peek("abc").is_err());
/// assert_eq!(parser.parse_peek(""), Ok(("", "")));
/// ```
#[doc(alias = "end")]
#[doc(alias = "eoi")]
pub fn eof<Input, Error>(input: &mut Input) -> Result<<Input as Stream>::Slice, Error>
where
    Input: Stream,
    Error: ParserError<Input>,
{
    trace("eof", move |input: &mut Input| {
        if input.eof_offset() == 0 {
            Ok(input.next_slice(0))
        } else {
            Err(ParserError::from_input(input))
        }
    })
    .parse_next(input)
}

/// Succeeds if the child parser returns an error.
///
/// <div class="warning">
///
/// **Note:** This does not advance the [`Stream`]
///
/// </div>
///
/// # Example
///
/// ```rust
/// # #[cfg(feature = "ascii")] {
/// # use winnow::prelude::*;
/// use winnow::combinator::not;
/// use winnow::ascii::alpha1;
///
/// fn parser<'i>(input: &mut &'i str) -> ModalResult<()> {
///     not(alpha1).parse_next(input)
/// }
///
/// assert_eq!(parser.parse_peek("123"), Ok(("123", ())));
/// assert!(parser.parse_peek("abcd").is_err());
/// # }
/// ```
pub fn not<Input, Output, Error, ParseNext>(
    parser: ParseNext,
) -> Not<Input, Error, Void<ParseNext, Input, Output, Error>>
where
    Input: Stream,
    Error: ParserError<Input>,
    ParseNext: Parser<Input, Output, Error>,
{
    Not::<_, _, Void<ParseNext, _, _, _>>::new_voided(parser)
}

/// Transforms an [`ErrMode::Backtrack`][crate::error::ErrMode::Backtrack] (recoverable) to [`ErrMode::Cut`][crate::error::ErrMode::Cut] (unrecoverable)
///
/// This commits the parse result, preventing alternative branch paths like with
/// [`winnow::combinator::alt`][crate::combinator::alt].
///
/// See the [tutorial][crate::_tutorial::chapter_7] for more details.
///
/// # Example
///
/// Without `cut_err`:
/// ```rust
/// # #[cfg(feature = "ascii")] {
/// # use winnow::token::one_of;
/// # use winnow::token::rest;
/// # use winnow::ascii::digit1;
/// # use winnow::combinator::alt;
/// # use winnow::combinator::preceded;
/// # use winnow::prelude::*;
///
/// fn parser<'i>(input: &mut &'i str) -> ModalResult<&'i str> {
///   alt((
///     preceded(one_of(['+', '-']), digit1),
///     rest
///   )).parse_next(input)
/// }
///
/// assert_eq!(parser.parse_peek("+10 ab"), Ok((" ab", "10")));
/// assert_eq!(parser.parse_peek("ab"), Ok(("", "ab")));
/// assert_eq!(parser.parse_peek("+"), Ok(("", "+")));
/// # }
/// ```
///
/// With `cut_err`:
/// ```rust
/// # #[cfg(feature = "ascii")] {
/// # use winnow::{error::ErrMode, error::ContextError};
/// # use winnow::prelude::*;
/// # use winnow::token::one_of;
/// # use winnow::token::rest;
/// # use winnow::ascii::digit1;
/// # use winnow::combinator::alt;
/// # use winnow::combinator::preceded;
/// use winnow::combinator::cut_err;
///
/// fn parser<'i>(input: &mut &'i str) -> ModalResult<&'i str> {
///   alt((
///     preceded(one_of(['+', '-']), cut_err(digit1)),
///     rest
///   )).parse_next(input)
/// }
///
/// assert_eq!(parser.parse_peek("+10 ab"), Ok((" ab", "10")));
/// assert_eq!(parser.parse_peek("ab"), Ok(("", "ab")));
/// assert_eq!(parser.parse_peek("+"), Err(ErrMode::Cut(ContextError::new())));
/// # }
/// ```
pub fn cut_err<Input, Output, Error, ParseNext>(
    parser: ParseNext,
) -> CutErr<Input, Output, Error, ParseNext>
where
    Input: Stream,
    Error: ParserError<Input> + ModalError,
    ParseNext: Parser<Input, Output, Error>,
{
    CutErr::new(parser)
}

/// Transforms an [`ErrMode::Cut`][crate::error::ErrMode::Cut] (unrecoverable) to [`ErrMode::Backtrack`][crate::error::ErrMode::Backtrack] (recoverable)
///
/// This attempts the parse, allowing other parsers to be tried on failure, like with
/// [`winnow::combinator::alt`][crate::combinator::alt].
pub fn backtrack_err<Input, Output, Error, ParseNext>(
    parser: ParseNext,
) -> BacktrackErr<Input, Output, Error, ParseNext>
where
    Input: Stream,
    Error: ParserError<Input> + ModalError,
    ParseNext: Parser<Input, Output, Error>,
{
    BacktrackErr::new(parser)
}

/// A placeholder for a not-yet-implemented [`Parser`]
///
/// This is analogous to the [`todo!`] macro and helps with prototyping.
///
/// # Panic
///
/// This will panic when parsing
///
/// # Example
///
/// ```rust
/// # use winnow::prelude::*;
/// # use winnow::combinator::todo;
///
/// fn parser(input: &mut &str) -> ModalResult<u64> {
///     todo(input)
/// }
/// ```
#[track_caller]
pub fn todo<Input, Output, Error>(input: &mut Input) -> Result<Output, Error>
where
    Input: Stream,
    Error: ParserError<Input>,
{
    #![allow(clippy::todo)]
    trace("todo", move |_input: &mut Input| {
        todo!("unimplemented parse")
    })
    .parse_next(input)
}

/// Succeed, consuming no input
///
/// For example, it can be used as the last alternative in `alt` to
/// specify the default case.
///
/// Useful with:
/// - [`Parser::value`]
/// - [`Parser::default_value`]
/// - [`Parser::map`]
///
/// <div class="warning">
///
/// **Note:** This never advances the [`Stream`]
///
/// </div>
///
/// # Example
///
/// ```rust
/// # use winnow::prelude::*;
/// use winnow::combinator::alt;
/// use winnow::combinator::empty;
///
/// fn sign(input: &mut &str) -> ModalResult<isize> {
///     alt((
///         '-'.value(-1),
///         '+'.value(1),
///         empty.value(1)
///     )).parse_next(input)
/// }
/// assert_eq!(sign.parse_peek("+10"), Ok(("10", 1)));
/// assert_eq!(sign.parse_peek("-10"), Ok(("10", -1)));
/// assert_eq!(sign.parse_peek("10"), Ok(("10", 1)));
/// ```
#[doc(alias = "value")]
#[doc(alias = "success")]
#[inline]
pub fn empty<Input, Error>(_input: &mut Input) -> Result<(), Error>
where
    Input: Stream,
    Error: ParserError<Input>,
{
    Ok(())
}

/// A parser which always fails.
///
/// For example, it can be used as the last alternative in `alt` to
/// control the error message given.
///
/// # Example
///
/// ```rust
/// # use winnow::{error::ErrMode, error::InputError};
/// # use winnow::prelude::*;
/// use winnow::combinator::fail;
///
/// fn parser<'i>(input: &mut &'i str) -> ModalResult<(), InputError<&'i str>> {
///     fail.parse_next(input)
/// }
///
/// assert_eq!(parser.parse_peek("string"), Err(ErrMode::Backtrack(InputError::at("string"))));
/// ```
#[doc(alias = "unexpected")]
#[inline]
pub fn fail<Input, Output, Error>(i: &mut Input) -> Result<Output, Error>
where
    Input: Stream,
    Error: ParserError<Input>,
{
    trace("fail", |i: &mut Input| Err(ParserError::from_input(i))).parse_next(i)
}
