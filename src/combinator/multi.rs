//! Combinators applying their child parser multiple times

use crate::error::ErrMode;
use crate::error::ErrorKind;
use crate::error::ParseError;
use crate::stream::Accumulate;
use crate::stream::Range;
use crate::stream::Stream;
use crate::trace::trace;
use crate::IResult;
use crate::Parser;

/// [`Accumulate`] the output of a parser into a container, like `Vec`
///
/// This stops before `n` when the parser returns [`ErrMode::Backtrack`].  To instead chain an error up, see
/// [`cut_err`][crate::combinator::cut_err].
///
/// # Arguments
/// * `m` The minimum number of iterations.
/// * `n` The maximum number of iterations.
/// * `f` The parser to apply.
///
/// To recognize a series of tokens, [`Accumulate`] into a `()` and then [`Parser::recognize`].
///
/// **Warning:** If the parser passed to `repeat` accepts empty inputs
/// (like `alpha0` or `digit0`), `repeat` will return an error,
/// to prevent going into an infinite loop.
///
/// # Example
///
/// Zero or more reptitions:
/// ```rust
/// # #[cfg(feature = "std")] {
/// # use winnow::{error::ErrMode, error::ErrorKind, error::Needed};
/// # use winnow::prelude::*;
/// use winnow::combinator::repeat;
/// use winnow::token::tag;
///
/// fn parser(s: &str) -> IResult<&str, Vec<&str>> {
///   repeat(0.., "abc").parse_next(s)
/// }
///
/// assert_eq!(parser("abcabc"), Ok(("", vec!["abc", "abc"])));
/// assert_eq!(parser("abc123"), Ok(("123", vec!["abc"])));
/// assert_eq!(parser("123123"), Ok(("123123", vec![])));
/// assert_eq!(parser(""), Ok(("", vec![])));
/// # }
/// ```
///
/// One or more reptitions:
/// ```rust
/// # #[cfg(feature = "std")] {
/// # use winnow::{error::ErrMode, error::{Error, ErrorKind}, error::Needed};
/// # use winnow::prelude::*;
/// use winnow::combinator::repeat;
/// use winnow::token::tag;
///
/// fn parser(s: &str) -> IResult<&str, Vec<&str>> {
///   repeat(1.., "abc").parse_next(s)
/// }
///
/// assert_eq!(parser("abcabc"), Ok(("", vec!["abc", "abc"])));
/// assert_eq!(parser("abc123"), Ok(("123", vec!["abc"])));
/// assert_eq!(parser("123123"), Err(ErrMode::Backtrack(Error::new("123123", ErrorKind::Tag))));
/// assert_eq!(parser(""), Err(ErrMode::Backtrack(Error::new("", ErrorKind::Tag))));
/// # }
/// ```
///
/// Fixed number of repeitions:
/// ```rust
/// # #[cfg(feature = "std")] {
/// # use winnow::{error::ErrMode, error::{Error, ErrorKind}, error::Needed};
/// # use winnow::prelude::*;
/// use winnow::combinator::repeat;
/// use winnow::token::tag;
///
/// fn parser(s: &str) -> IResult<&str, Vec<&str>> {
///   repeat(2, "abc").parse_next(s)
/// }
///
/// assert_eq!(parser("abcabc"), Ok(("", vec!["abc", "abc"])));
/// assert_eq!(parser("abc123"), Err(ErrMode::Backtrack(Error::new("123", ErrorKind::Tag))));
/// assert_eq!(parser("123123"), Err(ErrMode::Backtrack(Error::new("123123", ErrorKind::Tag))));
/// assert_eq!(parser(""), Err(ErrMode::Backtrack(Error::new("", ErrorKind::Tag))));
/// assert_eq!(parser("abcabcabc"), Ok(("abc", vec!["abc", "abc"])));
/// # }
/// ```
///
/// Arbitrary reptitions:
/// ```rust
/// # #[cfg(feature = "std")] {
/// # use winnow::{error::ErrMode, error::ErrorKind, error::Needed};
/// # use winnow::prelude::*;
/// use winnow::combinator::repeat;
/// use winnow::token::tag;
///
/// fn parser(s: &str) -> IResult<&str, Vec<&str>> {
///   repeat(0..=2, "abc").parse_next(s)
/// }
///
/// assert_eq!(parser("abcabc"), Ok(("", vec!["abc", "abc"])));
/// assert_eq!(parser("abc123"), Ok(("123", vec!["abc"])));
/// assert_eq!(parser("123123"), Ok(("123123", vec![])));
/// assert_eq!(parser(""), Ok(("", vec![])));
/// assert_eq!(parser("abcabcabc"), Ok(("abc", vec!["abc", "abc"])));
/// # }
/// ```
#[doc(alias = "many0")]
#[doc(alias = "count")]
#[doc(alias = "many0_count")]
#[doc(alias = "many1")]
#[doc(alias = "many1_count")]
#[doc(alias = "many_m_n")]
#[doc(alias = "repeated")]
#[doc(alias = "skip_many")]
#[doc(alias = "skip_many1")]
#[inline(always)]
pub fn repeat<I, O, C, E, F>(range: impl Into<Range>, mut f: F) -> impl Parser<I, C, E>
where
    I: Stream,
    C: Accumulate<O>,
    F: Parser<I, O, E>,
    E: ParseError<I>,
{
    let Range {
        start_inclusive,
        end_inclusive,
    } = range.into();
    trace("repeat", move |i: I| {
        match (start_inclusive, end_inclusive) {
            (0, None) => repeat0_(&mut f, i),
            (1, None) => repeat1_(&mut f, i),
            (start, end) if Some(start) == end => repeat_n_(start, &mut f, i),
            (start, end) => repeat_m_n_(start, end.unwrap_or(usize::MAX), &mut f, i),
        }
    })
}

/// Deprecated, replaced by [`repeat`]
#[deprecated(since = "0.4.6", note = "Replaced with `repeat`")]
#[inline(always)]
pub fn repeat0<I, O, C, E, F>(f: F) -> impl Parser<I, C, E>
where
    I: Stream,
    C: Accumulate<O>,
    F: Parser<I, O, E>,
    E: ParseError<I>,
{
    repeat(0.., f)
}

fn repeat0_<I, O, C, E, F>(f: &mut F, mut i: I) -> IResult<I, C, E>
where
    I: Stream,
    C: Accumulate<O>,
    F: Parser<I, O, E>,
    E: ParseError<I>,
{
    let mut acc = C::initial(None);
    loop {
        let len = i.eof_offset();
        match f.parse_next(i.clone()) {
            Err(ErrMode::Backtrack(_)) => return Ok((i, acc)),
            Err(e) => return Err(e),
            Ok((i1, o)) => {
                // infinite loop check: the parser must always consume
                if i1.eof_offset() == len {
                    return Err(ErrMode::assert(i, "`repeat` parsers must always consume"));
                }

                i = i1;
                acc.accumulate(o);
            }
        }
    }
}

/// Deprecated, replaced by [`repeat`]
#[deprecated(since = "0.4.6", note = "Replaced with `repeat`")]
#[inline(always)]
pub fn repeat1<I, O, C, E, F>(f: F) -> impl Parser<I, C, E>
where
    I: Stream,
    C: Accumulate<O>,
    F: Parser<I, O, E>,
    E: ParseError<I>,
{
    repeat(1.., f)
}

fn repeat1_<I, O, C, E, F>(f: &mut F, mut i: I) -> IResult<I, C, E>
where
    I: Stream,
    C: Accumulate<O>,
    F: Parser<I, O, E>,
    E: ParseError<I>,
{
    match f.parse_next(i.clone()) {
        Err(e) => Err(e.append(i, ErrorKind::Many)),
        Ok((i1, o)) => {
            let mut acc = C::initial(None);
            acc.accumulate(o);
            i = i1;

            loop {
                let len = i.eof_offset();
                match f.parse_next(i.clone()) {
                    Err(ErrMode::Backtrack(_)) => return Ok((i, acc)),
                    Err(e) => return Err(e),
                    Ok((i1, o)) => {
                        // infinite loop check: the parser must always consume
                        if i1.eof_offset() == len {
                            return Err(ErrMode::assert(i, "`repeat` parsers must always consume"));
                        }

                        i = i1;
                        acc.accumulate(o);
                    }
                }
            }
        }
    }
}

/// [`Accumulate`] the output of parser `f` into a container, like `Vec`, until the parser `g`
/// produces a result.
///
/// Returns a tuple of the results of `f` in a `Vec` and the result of `g`.
///
/// `f` keeps going so long as `g` produces [`ErrMode::Backtrack`]. To instead chain an error up, see [`cut_err`][crate::combinator::cut_err].
///
/// To recognize a series of tokens, [`Accumulate`] into a `()` and then [`Parser::recognize`].
///
/// # Example
///
/// ```rust
/// # #[cfg(feature = "std")] {
/// # use winnow::{error::ErrMode, error::{Error, ErrorKind}, error::Needed};
/// # use winnow::prelude::*;
/// use winnow::combinator::repeat_till0;
/// use winnow::token::tag;
///
/// fn parser(s: &str) -> IResult<&str, (Vec<&str>, &str)> {
///   repeat_till0("abc", "end").parse_next(s)
/// };
///
/// assert_eq!(parser("abcabcend"), Ok(("", (vec!["abc", "abc"], "end"))));
/// assert_eq!(parser("abc123end"), Err(ErrMode::Backtrack(Error::new("123end", ErrorKind::Tag))));
/// assert_eq!(parser("123123end"), Err(ErrMode::Backtrack(Error::new("123123end", ErrorKind::Tag))));
/// assert_eq!(parser(""), Err(ErrMode::Backtrack(Error::new("", ErrorKind::Tag))));
/// assert_eq!(parser("abcendefg"), Ok(("efg", (vec!["abc"], "end"))));
/// # }
/// ```
#[doc(alias = "many_till0")]
pub fn repeat_till0<I, O, C, P, E, F, G>(mut f: F, mut g: G) -> impl Parser<I, (C, P), E>
where
    I: Stream,
    C: Accumulate<O>,
    F: Parser<I, O, E>,
    G: Parser<I, P, E>,
    E: ParseError<I>,
{
    trace("repeat_till0", move |mut i: I| {
        let mut res = C::initial(None);
        loop {
            let len = i.eof_offset();
            match g.parse_next(i.clone()) {
                Ok((i1, o)) => return Ok((i1, (res, o))),
                Err(ErrMode::Backtrack(_)) => {
                    match f.parse_next(i.clone()) {
                        Err(e) => return Err(e.append(i, ErrorKind::Many)),
                        Ok((i1, o)) => {
                            // infinite loop check: the parser must always consume
                            if i1.eof_offset() == len {
                                return Err(ErrMode::assert(
                                    i,
                                    "`repeat` parsers must always consume",
                                ));
                            }

                            res.accumulate(o);
                            i = i1;
                        }
                    }
                }
                Err(e) => return Err(e),
            }
        }
    })
}

/// [`Accumulate`] the output of a parser, interleaed with `sep`
///
/// This stops when either parser returns [`ErrMode::Backtrack`].  To instead chain an error up, see
/// [`cut_err`][crate::combinator::cut_err].
///
/// # Arguments
/// * `parser` Parses the elements of the list.
/// * `sep` Parses the separator between list elements.
///
/// # Example
///
/// ```rust
/// # #[cfg(feature = "std")] {
/// # use winnow::{error::ErrMode, error::ErrorKind, error::Needed};
/// # use winnow::prelude::*;
/// use winnow::combinator::separated0;
/// use winnow::token::tag;
///
/// fn parser(s: &str) -> IResult<&str, Vec<&str>> {
///   separated0("abc", "|").parse_next(s)
/// }
///
/// assert_eq!(parser("abc|abc|abc"), Ok(("", vec!["abc", "abc", "abc"])));
/// assert_eq!(parser("abc123abc"), Ok(("123abc", vec!["abc"])));
/// assert_eq!(parser("abc|def"), Ok(("|def", vec!["abc"])));
/// assert_eq!(parser(""), Ok(("", vec![])));
/// assert_eq!(parser("def|abc"), Ok(("def|abc", vec![])));
/// # }
/// ```
#[doc(alias = "sep_by")]
#[doc(alias = "separated_list0")]
pub fn separated0<I, O, C, O2, E, P, S>(mut parser: P, mut sep: S) -> impl Parser<I, C, E>
where
    I: Stream,
    C: Accumulate<O>,
    P: Parser<I, O, E>,
    S: Parser<I, O2, E>,
    E: ParseError<I>,
{
    trace("separated0", move |mut i: I| {
        let mut res = C::initial(None);

        match parser.parse_next(i.clone()) {
            Err(ErrMode::Backtrack(_)) => return Ok((i, res)),
            Err(e) => return Err(e),
            Ok((i1, o)) => {
                res.accumulate(o);
                i = i1;
            }
        }

        loop {
            let len = i.eof_offset();
            match sep.parse_next(i.clone()) {
                Err(ErrMode::Backtrack(_)) => return Ok((i, res)),
                Err(e) => return Err(e),
                Ok((i1, _)) => {
                    // infinite loop check: the parser must always consume
                    if i1.eof_offset() == len {
                        return Err(ErrMode::assert(i, "sep parsers must always consume"));
                    }

                    match parser.parse_next(i1.clone()) {
                        Err(ErrMode::Backtrack(_)) => return Ok((i, res)),
                        Err(e) => return Err(e),
                        Ok((i2, o)) => {
                            res.accumulate(o);
                            i = i2;
                        }
                    }
                }
            }
        }
    })
}

/// [`Accumulate`] the output of a parser, interleaed with `sep`
///
/// Fails if the element parser does not produce at least one element.$
///
/// This stops when either parser returns [`ErrMode::Backtrack`].  To instead chain an error up, see
/// [`cut_err`][crate::combinator::cut_err].
///
/// # Arguments
/// * `sep` Parses the separator between list elements.
/// * `f` Parses the elements of the list.
///
/// # Example
///
/// ```rust
/// # #[cfg(feature = "std")] {
/// # use winnow::{error::ErrMode, error::{Error, ErrorKind}, error::Needed};
/// # use winnow::prelude::*;
/// use winnow::combinator::separated1;
/// use winnow::token::tag;
///
/// fn parser(s: &str) -> IResult<&str, Vec<&str>> {
///   separated1("abc", "|").parse_next(s)
/// }
///
/// assert_eq!(parser("abc|abc|abc"), Ok(("", vec!["abc", "abc", "abc"])));
/// assert_eq!(parser("abc123abc"), Ok(("123abc", vec!["abc"])));
/// assert_eq!(parser("abc|def"), Ok(("|def", vec!["abc"])));
/// assert_eq!(parser(""), Err(ErrMode::Backtrack(Error::new("", ErrorKind::Tag))));
/// assert_eq!(parser("def|abc"), Err(ErrMode::Backtrack(Error::new("def|abc", ErrorKind::Tag))));
/// # }
/// ```
#[doc(alias = "sep_by1")]
#[doc(alias = "separated_list1")]
pub fn separated1<I, O, C, O2, E, P, S>(mut parser: P, mut sep: S) -> impl Parser<I, C, E>
where
    I: Stream,
    C: Accumulate<O>,
    P: Parser<I, O, E>,
    S: Parser<I, O2, E>,
    E: ParseError<I>,
{
    trace("separated1", move |mut i: I| {
        let mut res = C::initial(None);

        // Parse the first element
        match parser.parse_next(i.clone()) {
            Err(e) => return Err(e),
            Ok((i1, o)) => {
                res.accumulate(o);
                i = i1;
            }
        }

        loop {
            let len = i.eof_offset();
            match sep.parse_next(i.clone()) {
                Err(ErrMode::Backtrack(_)) => return Ok((i, res)),
                Err(e) => return Err(e),
                Ok((i1, _)) => {
                    // infinite loop check: the parser must always consume
                    if i1.eof_offset() == len {
                        return Err(ErrMode::assert(i, "sep parsers must always consume"));
                    }

                    match parser.parse_next(i1.clone()) {
                        Err(ErrMode::Backtrack(_)) => return Ok((i, res)),
                        Err(e) => return Err(e),
                        Ok((i2, o)) => {
                            res.accumulate(o);
                            i = i2;
                        }
                    }
                }
            }
        }
    })
}

/// Alternates between two parsers, merging the results (left associative)
///
/// This stops when either parser returns [`ErrMode::Backtrack`].  To instead chain an error up, see
/// [`cut_err`][crate::combinator::cut_err].
///
/// # Example
///
/// ```rust
/// # use winnow::{error::ErrMode, error::{Error, ErrorKind}, error::Needed};
/// # use winnow::prelude::*;
/// use winnow::combinator::separated_foldl1;
/// use winnow::ascii::dec_int;
///
/// fn parser(s: &str) -> IResult<&str, i32> {
///   separated_foldl1(dec_int, "-", |l, _, r| l - r).parse_next(s)
/// }
///
/// assert_eq!(parser("9-3-5"), Ok(("", 1)));
/// assert_eq!(parser(""), Err(ErrMode::Backtrack(Error::new("", ErrorKind::Slice))));
/// assert_eq!(parser("def|abc"), Err(ErrMode::Backtrack(Error::new("def|abc", ErrorKind::Slice))));
/// ```
pub fn separated_foldl1<I, O, O2, E, P, S, Op>(
    mut parser: P,
    mut sep: S,
    op: Op,
) -> impl Parser<I, O, E>
where
    I: Stream,
    P: Parser<I, O, E>,
    S: Parser<I, O2, E>,
    E: ParseError<I>,
    Op: Fn(O, O2, O) -> O,
{
    trace("separated_foldl1", move |i: I| {
        let (mut i, mut ol) = parser.parse_next(i)?;

        loop {
            let len = i.eof_offset();
            match sep.parse_next(i.clone()) {
                Err(ErrMode::Backtrack(_)) => return Ok((i, ol)),
                Err(e) => return Err(e),
                Ok((i1, s)) => {
                    // infinite loop check: the parser must always consume
                    if i1.eof_offset() == len {
                        return Err(ErrMode::assert(i, "`repeat` parsers must always consume"));
                    }

                    match parser.parse_next(i1.clone()) {
                        Err(ErrMode::Backtrack(_)) => return Ok((i, ol)),
                        Err(e) => return Err(e),
                        Ok((i2, or)) => {
                            ol = op(ol, s, or);
                            i = i2;
                        }
                    }
                }
            }
        }
    })
}

/// Alternates between two parsers, merging the results (right associative)
///
/// This stops when either parser returns [`ErrMode::Backtrack`].  To instead chain an error up, see
/// [`cut_err`][crate::combinator::cut_err].
///
/// # Example
///
/// ```
/// # use winnow::{error::ErrMode, error::{Error, ErrorKind}, error::Needed};
/// # use winnow::prelude::*;
/// use winnow::combinator::separated_foldr1;
/// use winnow::ascii::dec_uint;
///
/// fn parser(s: &str) -> IResult<&str, u32> {
///   separated_foldr1(dec_uint, "^", |l: u32, _, r: u32| l.pow(r)).parse_next(s)
/// }
///
/// assert_eq!(parser("2^3^2"), Ok(("", 512)));
/// assert_eq!(parser("2"), Ok(("", 2)));
/// assert_eq!(parser(""), Err(ErrMode::Backtrack(Error::new("", ErrorKind::Slice))));
/// assert_eq!(parser("def|abc"), Err(ErrMode::Backtrack(Error::new("def|abc", ErrorKind::Slice))));
/// ```
#[cfg(feature = "alloc")]
pub fn separated_foldr1<I, O, O2, E, P, S, Op>(
    mut parser: P,
    mut sep: S,
    op: Op,
) -> impl Parser<I, O, E>
where
    I: Stream,
    P: Parser<I, O, E>,
    S: Parser<I, O2, E>,
    E: ParseError<I>,
    Op: Fn(O, O2, O) -> O,
{
    trace("separated_foldr1", move |i: I| {
        let (i, ol) = parser.parse_next(i)?;
        let (i, all): (_, crate::lib::std::vec::Vec<(O2, O)>) =
            repeat(0.., (sep.by_ref(), parser.by_ref())).parse_next(i)?;
        if let Some((s, or)) = all
            .into_iter()
            .rev()
            .reduce(|(sr, or), (sl, ol)| (sl, op(ol, sr, or)))
        {
            let merged = op(ol, s, or);
            Ok((i, merged))
        } else {
            Ok((i, ol))
        }
    })
}

fn repeat_m_n_<I, O, C, E, F>(
    min: usize,
    max: usize,
    parse: &mut F,
    mut input: I,
) -> IResult<I, C, E>
where
    I: Stream,
    C: Accumulate<O>,
    F: Parser<I, O, E>,
    E: ParseError<I>,
{
    if min > max {
        return Err(ErrMode::Cut(E::from_error_kind(input, ErrorKind::Many)));
    }

    let mut res = C::initial(Some(min));
    for count in 0..max {
        let len = input.eof_offset();
        match parse.parse_next(input.clone()) {
            Ok((tail, value)) => {
                // infinite loop check: the parser must always consume
                if tail.eof_offset() == len {
                    return Err(ErrMode::assert(
                        input,
                        "`repeat` parsers must always consume",
                    ));
                }

                res.accumulate(value);
                input = tail;
            }
            Err(ErrMode::Backtrack(e)) => {
                if count < min {
                    return Err(ErrMode::Backtrack(e.append(input, ErrorKind::Many)));
                } else {
                    return Ok((input, res));
                }
            }
            Err(e) => {
                return Err(e);
            }
        }
    }

    Ok((input, res))
}

/// Deprecated, replaced by [`repeat`]
#[deprecated(since = "0.4.6", note = "Replaced with `repeat`")]
#[inline(always)]
pub fn count<I, O, C, E, F>(f: F, count: usize) -> impl Parser<I, C, E>
where
    I: Stream,
    C: Accumulate<O>,
    F: Parser<I, O, E>,
    E: ParseError<I>,
{
    repeat(count, f)
}

fn repeat_n_<I, O, C, E, F>(count: usize, f: &mut F, i: I) -> IResult<I, C, E>
where
    I: Stream,
    C: Accumulate<O>,
    F: Parser<I, O, E>,
    E: ParseError<I>,
{
    let mut input = i.clone();
    let mut res = C::initial(Some(count));

    for _ in 0..count {
        let input_ = input.clone();
        match f.parse_next(input_) {
            Ok((i, o)) => {
                res.accumulate(o);
                input = i;
            }
            Err(e) => {
                return Err(e.append(i, ErrorKind::Many));
            }
        }
    }

    Ok((input, res))
}

/// Repeats the embedded parser, filling the given slice with results.
///
/// This parser fails if the input runs out before the given slice is full.
///
/// # Arguments
/// * `f` The parser to apply.
/// * `buf` The slice to fill
///
/// # Example
///
/// ```rust
/// # use winnow::{error::ErrMode, error::{Error, ErrorKind}, error::Needed};
/// # use winnow::prelude::*;
/// use winnow::combinator::fill;
/// use winnow::token::tag;
///
/// fn parser(s: &str) -> IResult<&str, [&str; 2]> {
///   let mut buf = ["", ""];
///   let (rest, ()) = fill("abc", &mut buf).parse_next(s)?;
///   Ok((rest, buf))
/// }
///
/// assert_eq!(parser("abcabc"), Ok(("", ["abc", "abc"])));
/// assert_eq!(parser("abc123"), Err(ErrMode::Backtrack(Error::new("123", ErrorKind::Tag))));
/// assert_eq!(parser("123123"), Err(ErrMode::Backtrack(Error::new("123123", ErrorKind::Tag))));
/// assert_eq!(parser(""), Err(ErrMode::Backtrack(Error::new("", ErrorKind::Tag))));
/// assert_eq!(parser("abcabcabc"), Ok(("abc", ["abc", "abc"])));
/// ```
pub fn fill<'a, I, O, E, F>(mut f: F, buf: &'a mut [O]) -> impl Parser<I, (), E> + 'a
where
    I: Stream + 'a,
    F: Parser<I, O, E> + 'a,
    E: ParseError<I> + 'a,
{
    trace("fill", move |i: I| {
        let mut input = i.clone();

        for elem in buf.iter_mut() {
            let input_ = input.clone();
            match f.parse_next(input_) {
                Ok((i, o)) => {
                    *elem = o;
                    input = i;
                }
                Err(e) => {
                    return Err(e.append(i, ErrorKind::Many));
                }
            }
        }

        Ok((input, ()))
    })
}

/// Repeats the embedded parser `m..=n` times, calling `g` to gather the results
///
/// This stops before `n` when the parser returns [`ErrMode::Backtrack`].  To instead chain an error up, see
/// [`cut_err`][crate::combinator::cut_err].
///
/// # Arguments
/// * `m` The minimum number of iterations.
/// * `n` The maximum number of iterations.
/// * `f` The parser to apply.
/// * `init` A function returning the initial value.
/// * `g` The function that combines a result of `f` with
///       the current accumulator.
///
/// **Warning:** If the parser passed to `fold_repeat` accepts empty inputs
/// (like `alpha0` or `digit0`), `fold_repeat` will return an error,
/// to prevent going into an infinite loop.
///
/// # Example
///
/// Zero or more repetitions:
/// ```rust
/// # use winnow::{error::ErrMode, error::ErrorKind, error::Needed};
/// # use winnow::prelude::*;
/// use winnow::combinator::fold_repeat;
/// use winnow::token::tag;
///
/// fn parser(s: &str) -> IResult<&str, Vec<&str>> {
///   fold_repeat(
///     0..,
///     "abc",
///     Vec::new,
///     |mut acc: Vec<_>, item| {
///       acc.push(item);
///       acc
///     }
///   ).parse_next(s)
/// }
///
/// assert_eq!(parser("abcabc"), Ok(("", vec!["abc", "abc"])));
/// assert_eq!(parser("abc123"), Ok(("123", vec!["abc"])));
/// assert_eq!(parser("123123"), Ok(("123123", vec![])));
/// assert_eq!(parser(""), Ok(("", vec![])));
/// ```
///
/// One or more repetitions:
/// ```rust
/// # use winnow::{error::ErrMode, error::{Error, ErrorKind}, error::Needed};
/// # use winnow::prelude::*;
/// use winnow::combinator::fold_repeat;
/// use winnow::token::tag;
///
/// fn parser(s: &str) -> IResult<&str, Vec<&str>> {
///   fold_repeat(
///     1..,
///     "abc",
///     Vec::new,
///     |mut acc: Vec<_>, item| {
///       acc.push(item);
///       acc
///     }
///   ).parse_next(s)
/// }
///
/// assert_eq!(parser("abcabc"), Ok(("", vec!["abc", "abc"])));
/// assert_eq!(parser("abc123"), Ok(("123", vec!["abc"])));
/// assert_eq!(parser("123123"), Err(ErrMode::Backtrack(Error::new("123123", ErrorKind::Many))));
/// assert_eq!(parser(""), Err(ErrMode::Backtrack(Error::new("", ErrorKind::Many))));
/// ```
///
/// Arbitrary number of repetitions:
/// ```rust
/// # use winnow::{error::ErrMode, error::ErrorKind, error::Needed};
/// # use winnow::prelude::*;
/// use winnow::combinator::fold_repeat;
/// use winnow::token::tag;
///
/// fn parser(s: &str) -> IResult<&str, Vec<&str>> {
///   fold_repeat(
///     0..=2,
///     "abc",
///     Vec::new,
///     |mut acc: Vec<_>, item| {
///       acc.push(item);
///       acc
///     }
///   ).parse_next(s)
/// }
///
/// assert_eq!(parser("abcabc"), Ok(("", vec!["abc", "abc"])));
/// assert_eq!(parser("abc123"), Ok(("123", vec!["abc"])));
/// assert_eq!(parser("123123"), Ok(("123123", vec![])));
/// assert_eq!(parser(""), Ok(("", vec![])));
/// assert_eq!(parser("abcabcabc"), Ok(("abc", vec!["abc", "abc"])));
/// ```
#[doc(alias = "fold_many0")]
#[doc(alias = "fold_many1")]
#[doc(alias = "fold_many_m_n")]
#[inline(always)]
pub fn fold_repeat<I, O, E, F, G, H, R>(
    range: impl Into<Range>,
    mut f: F,
    mut init: H,
    mut g: G,
) -> impl Parser<I, R, E>
where
    I: Stream,
    F: Parser<I, O, E>,
    G: FnMut(R, O) -> R,
    H: FnMut() -> R,
    E: ParseError<I>,
{
    let Range {
        start_inclusive,
        end_inclusive,
    } = range.into();
    trace("fold_repeat", move |i: I| {
        match (start_inclusive, end_inclusive) {
            (0, None) => fold_repeat0_(&mut f, &mut init, &mut g, i),
            (1, None) => fold_repeat1_(&mut f, &mut init, &mut g, i),
            (start, end) => fold_repeat_m_n_(
                start,
                end.unwrap_or(usize::MAX),
                &mut f,
                &mut init,
                &mut g,
                i,
            ),
        }
    })
}

/// Deprecated, replaced by [`fold_repeat`]
#[deprecated(since = "0.4.6", note = "Replaced with `fold_repeat`")]
#[inline(always)]
pub fn fold_repeat0<I, O, E, F, G, H, R>(mut f: F, mut init: H, mut g: G) -> impl Parser<I, R, E>
where
    I: Stream,
    F: Parser<I, O, E>,
    G: FnMut(R, O) -> R,
    H: FnMut() -> R,
    E: ParseError<I>,
{
    trace("fold_repeat0", move |i: I| {
        fold_repeat0_(&mut f, &mut init, &mut g, i)
    })
}

fn fold_repeat0_<I, O, E, F, G, H, R>(f: &mut F, init: &mut H, g: &mut G, i: I) -> IResult<I, R, E>
where
    I: Stream,
    F: Parser<I, O, E>,
    G: FnMut(R, O) -> R,
    H: FnMut() -> R,
    E: ParseError<I>,
{
    let mut res = init();
    let mut input = i;

    loop {
        let i_ = input.clone();
        let len = input.eof_offset();
        match f.parse_next(i_) {
            Ok((i, o)) => {
                // infinite loop check: the parser must always consume
                if i.eof_offset() == len {
                    return Err(ErrMode::assert(i, "`repeat` parsers must always consume"));
                }

                res = g(res, o);
                input = i;
            }
            Err(ErrMode::Backtrack(_)) => {
                return Ok((input, res));
            }
            Err(e) => {
                return Err(e);
            }
        }
    }
}

/// Deprecated, replaced by [`fold_repeat`]
#[deprecated(since = "0.4.6", note = "Replaced with `fold_repeat`")]
#[inline(always)]
pub fn fold_repeat1<I, O, E, F, G, H, R>(mut f: F, mut init: H, mut g: G) -> impl Parser<I, R, E>
where
    I: Stream,
    F: Parser<I, O, E>,
    G: FnMut(R, O) -> R,
    H: FnMut() -> R,
    E: ParseError<I>,
{
    trace("fold_repeat1", move |i: I| {
        fold_repeat1_(&mut f, &mut init, &mut g, i)
    })
}

fn fold_repeat1_<I, O, E, F, G, H, R>(f: &mut F, init: &mut H, g: &mut G, i: I) -> IResult<I, R, E>
where
    I: Stream,
    F: Parser<I, O, E>,
    G: FnMut(R, O) -> R,
    H: FnMut() -> R,
    E: ParseError<I>,
{
    let _i = i.clone();
    let init = init();
    match f.parse_next(_i) {
        Err(ErrMode::Backtrack(_)) => Err(ErrMode::from_error_kind(i, ErrorKind::Many)),
        Err(e) => Err(e),
        Ok((i1, o1)) => {
            let mut acc = g(init, o1);
            let mut input = i1;

            loop {
                let _input = input.clone();
                let len = input.eof_offset();
                match f.parse_next(_input) {
                    Err(ErrMode::Backtrack(_)) => {
                        break;
                    }
                    Err(e) => return Err(e),
                    Ok((i, o)) => {
                        // infinite loop check: the parser must always consume
                        if i.eof_offset() == len {
                            return Err(ErrMode::assert(i, "`repeat` parsers must always consume"));
                        }

                        acc = g(acc, o);
                        input = i;
                    }
                }
            }

            Ok((input, acc))
        }
    }
}

fn fold_repeat_m_n_<I, O, E, F, G, H, R>(
    min: usize,
    max: usize,
    parse: &mut F,
    init: &mut H,
    fold: &mut G,
    mut input: I,
) -> IResult<I, R, E>
where
    I: Stream,
    F: Parser<I, O, E>,
    G: FnMut(R, O) -> R,
    H: FnMut() -> R,
    E: ParseError<I>,
{
    if min > max {
        return Err(ErrMode::Cut(E::from_error_kind(input, ErrorKind::Many)));
    }

    let mut acc = init();
    for count in 0..max {
        let len = input.eof_offset();
        match parse.parse_next(input.clone()) {
            Ok((tail, value)) => {
                // infinite loop check: the parser must always consume
                if tail.eof_offset() == len {
                    return Err(ErrMode::assert(
                        input,
                        "`repeat` parsers must always consume",
                    ));
                }

                acc = fold(acc, value);
                input = tail;
            }
            //FInputXMError: handle failure properly
            Err(ErrMode::Backtrack(err)) => {
                if count < min {
                    return Err(ErrMode::Backtrack(err.append(input, ErrorKind::Many)));
                } else {
                    break;
                }
            }
            Err(e) => return Err(e),
        }
    }

    Ok((input, acc))
}
