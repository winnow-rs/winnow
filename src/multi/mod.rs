//! Combinators applying their child parser multiple times

#[cfg(test)]
mod tests;

use crate::error::ErrMode;
use crate::error::ErrorKind;
use crate::error::ParseError;
use crate::stream::Accumulate;
use crate::stream::{Stream, StreamIsPartial, ToUsize, UpdateSlice};
use crate::trace::trace;
use crate::{IResult, Parser};

/// Repeats the embedded parser, gathering the results in a `Vec`.
///
/// This stops on [`ErrMode::Backtrack`].  To instead chain an error up, see
/// [`cut_err`][crate::combinator::cut_err].
///
/// # Arguments
/// * `f` The parser to apply.
///
/// *Note*: if the parser passed in accepts empty inputs (like `alpha0` or `digit0`), `many0` will
/// return an error, to prevent going into an infinite loop
///
#[cfg_attr(not(feature = "std"), doc = "```ignore")]
#[cfg_attr(feature = "std", doc = "```")]
/// # use winnow::{error::ErrMode, error::ErrorKind, error::Needed, IResult};
/// use winnow::multi::many0;
/// use winnow::bytes::tag;
///
/// fn parser(s: &str) -> IResult<&str, Vec<&str>> {
///   many0(tag("abc"))(s)
/// }
///
/// assert_eq!(parser("abcabc"), Ok(("", vec!["abc", "abc"])));
/// assert_eq!(parser("abc123"), Ok(("123", vec!["abc"])));
/// assert_eq!(parser("123123"), Ok(("123123", vec![])));
/// assert_eq!(parser(""), Ok(("", vec![])));
/// ```
pub fn many0<I, O, C, E, F>(mut f: F) -> impl FnMut(I) -> IResult<I, C, E>
where
    I: Stream,
    C: Accumulate<O>,
    F: Parser<I, O, E>,
    E: ParseError<I>,
{
    trace("many0", move |mut i: I| {
        let mut acc = C::initial(None);
        loop {
            let len = i.eof_offset();
            match f.parse_next(i.clone()) {
                Err(ErrMode::Backtrack(_)) => return Ok((i, acc)),
                Err(e) => return Err(e),
                Ok((i1, o)) => {
                    // infinite loop check: the parser must always consume
                    if i1.eof_offset() == len {
                        return Err(ErrMode::assert(i, "many parsers must always consume"));
                    }

                    i = i1;
                    acc.accumulate(o);
                }
            }
        }
    })
}

/// Runs the embedded parser, gathering the results in a `Vec`.
///
/// This stops on [`ErrMode::Backtrack`] if there is at least one result.  To instead chain an error up,
/// see [`cut_err`][crate::combinator::cut_err].
///
/// # Arguments
/// * `f` The parser to apply.
///
/// *Note*: If the parser passed to `many1` accepts empty inputs
/// (like `alpha0` or `digit0`), `many1` will return an error,
/// to prevent going into an infinite loop.
///
#[cfg_attr(not(feature = "std"), doc = "```ignore")]
#[cfg_attr(feature = "std", doc = "```")]
/// # use winnow::{error::ErrMode, error::{Error, ErrorKind}, error::Needed, IResult};
/// use winnow::multi::many1;
/// use winnow::bytes::tag;
///
/// fn parser(s: &str) -> IResult<&str, Vec<&str>> {
///   many1(tag("abc"))(s)
/// }
///
/// assert_eq!(parser("abcabc"), Ok(("", vec!["abc", "abc"])));
/// assert_eq!(parser("abc123"), Ok(("123", vec!["abc"])));
/// assert_eq!(parser("123123"), Err(ErrMode::Backtrack(Error::new("123123", ErrorKind::Tag))));
/// assert_eq!(parser(""), Err(ErrMode::Backtrack(Error::new("", ErrorKind::Tag))));
/// ```
pub fn many1<I, O, C, E, F>(mut f: F) -> impl FnMut(I) -> IResult<I, C, E>
where
    I: Stream,
    C: Accumulate<O>,
    F: Parser<I, O, E>,
    E: ParseError<I>,
{
    trace("many1", move |mut i: I| match f.parse_next(i.clone()) {
        Err(e) => Err(e.append(i, ErrorKind::Many1)),
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
                            return Err(ErrMode::assert(i, "many parsers must always consume"));
                        }

                        i = i1;
                        acc.accumulate(o);
                    }
                }
            }
        }
    })
}

/// **WARNING:** Deprecated, replaced with [`many_till0`]
#[deprecated(since = "0.3.0", note = "Replaced with `many_till0`")]
pub fn many_till<I, O, C, P, E, F, G>(f: F, g: G) -> impl FnMut(I) -> IResult<I, (C, P), E>
where
    I: Stream,
    C: Accumulate<O>,
    F: Parser<I, O, E>,
    G: Parser<I, P, E>,
    E: ParseError<I>,
{
    many_till0(f, g)
}

/// Applies the parser `f` until the parser `g` produces a result.
///
/// Returns a tuple of the results of `f` in a `Vec` and the result of `g`.
///
/// `f` keeps going so long as `g` produces [`ErrMode::Backtrack`]. To instead chain an error up, see [`cut_err`][crate::combinator::cut_err].
///
#[cfg_attr(not(feature = "std"), doc = "```ignore")]
#[cfg_attr(feature = "std", doc = "```")]
/// # use winnow::{error::ErrMode, error::{Error, ErrorKind}, error::Needed, IResult};
/// use winnow::multi::many_till0;
/// use winnow::bytes::tag;
///
/// fn parser(s: &str) -> IResult<&str, (Vec<&str>, &str)> {
///   many_till0(tag("abc"), tag("end"))(s)
/// };
///
/// assert_eq!(parser("abcabcend"), Ok(("", (vec!["abc", "abc"], "end"))));
/// assert_eq!(parser("abc123end"), Err(ErrMode::Backtrack(Error::new("123end", ErrorKind::Tag))));
/// assert_eq!(parser("123123end"), Err(ErrMode::Backtrack(Error::new("123123end", ErrorKind::Tag))));
/// assert_eq!(parser(""), Err(ErrMode::Backtrack(Error::new("", ErrorKind::Tag))));
/// assert_eq!(parser("abcendefg"), Ok(("efg", (vec!["abc"], "end"))));
/// ```
pub fn many_till0<I, O, C, P, E, F, G>(mut f: F, mut g: G) -> impl FnMut(I) -> IResult<I, (C, P), E>
where
    I: Stream,
    C: Accumulate<O>,
    F: Parser<I, O, E>,
    G: Parser<I, P, E>,
    E: ParseError<I>,
{
    trace("many_till0", move |mut i: I| {
        let mut res = C::initial(None);
        loop {
            let len = i.eof_offset();
            match g.parse_next(i.clone()) {
                Ok((i1, o)) => return Ok((i1, (res, o))),
                Err(ErrMode::Backtrack(_)) => {
                    match f.parse_next(i.clone()) {
                        Err(e) => return Err(e.append(i, ErrorKind::ManyTill)),
                        Ok((i1, o)) => {
                            // infinite loop check: the parser must always consume
                            if i1.eof_offset() == len {
                                return Err(ErrMode::assert(i, "many parsers must always consume"));
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

/// Alternates between two parsers to produce a list of elements.
///
/// This stops when either parser returns [`ErrMode::Backtrack`].  To instead chain an error up, see
/// [`cut_err`][crate::combinator::cut_err].
///
/// # Arguments
/// * `parser` Parses the elements of the list.
/// * `sep` Parses the separator between list elements.
///
#[cfg_attr(not(feature = "std"), doc = "```ignore")]
#[cfg_attr(feature = "std", doc = "```")]
/// # use winnow::{error::ErrMode, error::ErrorKind, error::Needed, IResult};
/// use winnow::multi::separated0;
/// use winnow::bytes::tag;
///
/// fn parser(s: &str) -> IResult<&str, Vec<&str>> {
///   separated0(tag("abc"), tag("|"))(s)
/// }
///
/// assert_eq!(parser("abc|abc|abc"), Ok(("", vec!["abc", "abc", "abc"])));
/// assert_eq!(parser("abc123abc"), Ok(("123abc", vec!["abc"])));
/// assert_eq!(parser("abc|def"), Ok(("|def", vec!["abc"])));
/// assert_eq!(parser(""), Ok(("", vec![])));
/// assert_eq!(parser("def|abc"), Ok(("def|abc", vec![])));
/// ```
pub fn separated0<I, O, C, O2, E, P, S>(
    mut parser: P,
    mut sep: S,
) -> impl FnMut(I) -> IResult<I, C, E>
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
                        return Err(ErrMode::assert(i, "many parsers must always consume"));
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

/// **WARNING:** Deprecated, replaced with [`separated0`] (note the parameter swap)
#[deprecated(
    since = "0.3.0",
    note = "Replaced with `separated0` (note the parameter swap)"
)]
pub fn separated_list0<I, O, C, O2, E, F, G>(sep: G, f: F) -> impl FnMut(I) -> IResult<I, C, E>
where
    I: Stream,
    C: Accumulate<O>,
    F: Parser<I, O, E>,
    G: Parser<I, O2, E>,
    E: ParseError<I>,
{
    separated0(f, sep)
}

/// Alternates between two parsers to produce a list of elements until [`ErrMode::Backtrack`].
///
/// Fails if the element parser does not produce at least one element.$
///
/// This stops when either parser returns [`ErrMode::Backtrack`].  To instead chain an error up, see
/// [`cut_err`][crate::combinator::cut_err].
///
/// # Arguments
/// * `sep` Parses the separator between list elements.
/// * `f` Parses the elements of the list.
#[cfg_attr(not(feature = "std"), doc = "```ignore")]
#[cfg_attr(feature = "std", doc = "```")]
/// # use winnow::{error::ErrMode, error::{Error, ErrorKind}, error::Needed, IResult};
/// use winnow::multi::separated1;
/// use winnow::bytes::tag;
///
/// fn parser(s: &str) -> IResult<&str, Vec<&str>> {
///   separated1(tag("abc"), tag("|"))(s)
/// }
///
/// assert_eq!(parser("abc|abc|abc"), Ok(("", vec!["abc", "abc", "abc"])));
/// assert_eq!(parser("abc123abc"), Ok(("123abc", vec!["abc"])));
/// assert_eq!(parser("abc|def"), Ok(("|def", vec!["abc"])));
/// assert_eq!(parser(""), Err(ErrMode::Backtrack(Error::new("", ErrorKind::Tag))));
/// assert_eq!(parser("def|abc"), Err(ErrMode::Backtrack(Error::new("def|abc", ErrorKind::Tag))));
/// ```
pub fn separated1<I, O, C, O2, E, P, S>(
    mut parser: P,
    mut sep: S,
) -> impl FnMut(I) -> IResult<I, C, E>
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
                        return Err(ErrMode::from_error_kind(i1, ErrorKind::SeparatedList));
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

/// **WARNING:** Deprecated, replaced with [`separated1`] (note the parameter swap)
#[deprecated(
    since = "0.3.0",
    note = "Replaced with `separated1` (note the parameter swap)"
)]
pub fn separated_list1<I, O, C, O2, E, F, G>(sep: G, f: F) -> impl FnMut(I) -> IResult<I, C, E>
where
    I: Stream,
    C: Accumulate<O>,
    F: Parser<I, O, E>,
    G: Parser<I, O2, E>,
    E: ParseError<I>,
{
    separated1(f, sep)
}

/// Alternates between two parsers, merging the results (left associative)
///
/// This stops when either parser returns [`ErrMode::Backtrack`].  To instead chain an error up, see
/// [`cut_err`][crate::combinator::cut_err].
///
/// ```
/// # use winnow::{error::ErrMode, error::{Error, ErrorKind}, error::Needed, IResult};
/// use winnow::multi::separated_foldl1;
/// use winnow::character::dec_int;
///
/// fn parser(s: &str) -> IResult<&str, i32> {
///   separated_foldl1(dec_int, "-", |l, _, r| l - r)(s)
/// }
///
/// assert_eq!(parser("9-3-5"), Ok(("", 1)));
/// assert_eq!(parser(""), Err(ErrMode::Backtrack(Error::new("", ErrorKind::Digit))));
/// assert_eq!(parser("def|abc"), Err(ErrMode::Backtrack(Error::new("def|abc", ErrorKind::Digit))));
/// ```
pub fn separated_foldl1<I, O, O2, E, P, S, Op>(
    mut parser: P,
    mut sep: S,
    op: Op,
) -> impl FnMut(I) -> IResult<I, O, E>
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
                        return Err(ErrMode::from_error_kind(i1, ErrorKind::SeparatedList));
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
/// ```
/// # use winnow::{error::ErrMode, error::{Error, ErrorKind}, error::Needed, IResult};
/// use winnow::multi::separated_foldr1;
/// use winnow::character::dec_uint;
///
/// fn parser(s: &str) -> IResult<&str, u32> {
///   separated_foldr1(dec_uint, "^", |l: u32, _, r: u32| l.pow(r))(s)
/// }
///
/// assert_eq!(parser("2^3^2"), Ok(("", 512)));
/// assert_eq!(parser("2"), Ok(("", 2)));
/// assert_eq!(parser(""), Err(ErrMode::Backtrack(Error::new("", ErrorKind::Digit))));
/// assert_eq!(parser("def|abc"), Err(ErrMode::Backtrack(Error::new("def|abc", ErrorKind::Digit))));
/// ```
#[cfg(feature = "alloc")]
pub fn separated_foldr1<I, O, O2, E, P, S, Op>(
    mut parser: P,
    mut sep: S,
    op: Op,
) -> impl FnMut(I) -> IResult<I, O, E>
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
            many0((sep.by_ref(), parser.by_ref())).parse_next(i)?;
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

/// Repeats the embedded parser `m..=n` times
///
/// This stops before `n` when the parser returns [`ErrMode::Backtrack`].  To instead chain an error up, see
/// [`cut_err`][crate::combinator::cut_err].
///
/// # Arguments
/// * `m` The minimum number of iterations.
/// * `n` The maximum number of iterations.
/// * `f` The parser to apply.
///
/// *Note*: If the parser passed to `many1` accepts empty inputs
/// (like `alpha0` or `digit0`), `many1` will return an error,
/// to prevent going into an infinite loop.
///
#[cfg_attr(not(feature = "std"), doc = "```ignore")]
#[cfg_attr(feature = "std", doc = "```")]
/// # use winnow::{error::ErrMode, error::ErrorKind, error::Needed, IResult};
/// use winnow::multi::many_m_n;
/// use winnow::bytes::tag;
///
/// fn parser(s: &str) -> IResult<&str, Vec<&str>> {
///   many_m_n(0, 2, tag("abc"))(s)
/// }
///
/// assert_eq!(parser("abcabc"), Ok(("", vec!["abc", "abc"])));
/// assert_eq!(parser("abc123"), Ok(("123", vec!["abc"])));
/// assert_eq!(parser("123123"), Ok(("123123", vec![])));
/// assert_eq!(parser(""), Ok(("", vec![])));
/// assert_eq!(parser("abcabcabc"), Ok(("abc", vec!["abc", "abc"])));
/// ```
pub fn many_m_n<I, O, C, E, F>(
    min: usize,
    max: usize,
    mut parse: F,
) -> impl FnMut(I) -> IResult<I, C, E>
where
    I: Stream,
    C: Accumulate<O>,
    F: Parser<I, O, E>,
    E: ParseError<I>,
{
    trace("many_m_n", move |mut input: I| {
        if min > max {
            return Err(ErrMode::Cut(E::from_error_kind(input, ErrorKind::ManyMN)));
        }

        let mut res = C::initial(Some(min));
        for count in 0..max {
            let len = input.eof_offset();
            match parse.parse_next(input.clone()) {
                Ok((tail, value)) => {
                    // infinite loop check: the parser must always consume
                    if tail.eof_offset() == len {
                        return Err(ErrMode::from_error_kind(input, ErrorKind::ManyMN));
                    }

                    res.accumulate(value);
                    input = tail;
                }
                Err(ErrMode::Backtrack(e)) => {
                    if count < min {
                        return Err(ErrMode::Backtrack(e.append(input, ErrorKind::ManyMN)));
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
    })
}

/// Repeats the embedded parser, counting the results
///
/// This stops on [`ErrMode::Backtrack`].  To instead chain an error up, see
/// [`cut_err`][crate::combinator::cut_err].
///
/// # Arguments
/// * `f` The parser to apply.
///
/// *Note*: if the parser passed in accepts empty inputs (like `alpha0` or `digit0`), `many0` will
/// return an error, to prevent going into an infinite loop
///
/// ```rust
/// # use winnow::{error::ErrMode, error::ErrorKind, error::Needed, IResult};
/// use winnow::multi::many0_count;
/// use winnow::bytes::tag;
///
/// fn parser(s: &str) -> IResult<&str, usize> {
///   many0_count(tag("abc"))(s)
/// }
///
/// assert_eq!(parser("abcabc"), Ok(("", 2)));
/// assert_eq!(parser("abc123"), Ok(("123", 1)));
/// assert_eq!(parser("123123"), Ok(("123123", 0)));
/// assert_eq!(parser(""), Ok(("", 0)));
/// ```
///
/// **WARNING:** Deprecated, replaced with [`many0`]
#[deprecated(since = "0.3.0", note = "Replaced with `many0`")]
pub fn many0_count<I, O, E, F>(mut f: F) -> impl FnMut(I) -> IResult<I, usize, E>
where
    I: Stream,
    F: Parser<I, O, E>,
    E: ParseError<I>,
{
    move |i: I| {
        let mut input = i;
        let mut count = 0;

        loop {
            let input_ = input.clone();
            let len = input.eof_offset();
            match f.parse_next(input_) {
                Ok((i, _)) => {
                    // infinite loop check: the parser must always consume
                    if i.eof_offset() == len {
                        return Err(ErrMode::from_error_kind(input, ErrorKind::Many0Count));
                    }

                    input = i;
                    count += 1;
                }

                Err(ErrMode::Backtrack(_)) => return Ok((input, count)),

                Err(e) => return Err(e),
            }
        }
    }
}

/// Runs the embedded parser, counting the results.
///
/// This stops on [`ErrMode::Backtrack`] if there is at least one result.  To instead chain an error up,
/// see [`cut_err`][crate::combinator::cut_err].
///
/// # Arguments
/// * `f` The parser to apply.
///
/// *Note*: If the parser passed to `many1` accepts empty inputs
/// (like `alpha0` or `digit0`), `many1` will return an error,
/// to prevent going into an infinite loop.
///
/// ```rust
/// # use winnow::{error::ErrMode, error::{Error, ErrorKind}, error::Needed, IResult};
/// use winnow::multi::many1_count;
/// use winnow::bytes::tag;
///
/// fn parser(s: &str) -> IResult<&str, usize> {
///   many1_count(tag("abc"))(s)
/// }
///
/// assert_eq!(parser("abcabc"), Ok(("", 2)));
/// assert_eq!(parser("abc123"), Ok(("123", 1)));
/// assert_eq!(parser("123123"), Err(ErrMode::Backtrack(Error::new("123123", ErrorKind::Many1Count))));
/// assert_eq!(parser(""), Err(ErrMode::Backtrack(Error::new("", ErrorKind::Many1Count))));
/// ```
///
/// **WARNING:** Deprecated, replaced with [`many0`]
#[deprecated(since = "0.3.0", note = "Replaced with `many0`")]
pub fn many1_count<I, O, E, F>(mut f: F) -> impl FnMut(I) -> IResult<I, usize, E>
where
    I: Stream,
    F: Parser<I, O, E>,
    E: ParseError<I>,
{
    move |i: I| {
        let i_ = i.clone();
        match f.parse_next(i_) {
            Err(ErrMode::Backtrack(_)) => Err(ErrMode::from_error_kind(i, ErrorKind::Many1Count)),
            Err(i) => Err(i),
            Ok((i1, _)) => {
                let mut count = 1;
                let mut input = i1;

                loop {
                    let len = input.eof_offset();
                    let input_ = input.clone();
                    match f.parse_next(input_) {
                        Err(ErrMode::Backtrack(_)) => return Ok((input, count)),
                        Err(e) => return Err(e),
                        Ok((i, _)) => {
                            // infinite loop check: the parser must always consume
                            if i.eof_offset() == len {
                                return Err(ErrMode::from_error_kind(i, ErrorKind::Many1Count));
                            }

                            count += 1;
                            input = i;
                        }
                    }
                }
            }
        }
    }
}

/// Runs the embedded parser `count` times, gathering the results in a `Vec`
///
/// # Arguments
/// * `f` The parser to apply.
/// * `count` How often to apply the parser.
#[cfg_attr(not(feature = "std"), doc = "```ignore")]
#[cfg_attr(feature = "std", doc = "```")]
/// # use winnow::{error::ErrMode, error::{Error, ErrorKind}, error::Needed, IResult};
/// use winnow::multi::count;
/// use winnow::bytes::tag;
///
/// fn parser(s: &str) -> IResult<&str, Vec<&str>> {
///   count(tag("abc"), 2)(s)
/// }
///
/// assert_eq!(parser("abcabc"), Ok(("", vec!["abc", "abc"])));
/// assert_eq!(parser("abc123"), Err(ErrMode::Backtrack(Error::new("123", ErrorKind::Tag))));
/// assert_eq!(parser("123123"), Err(ErrMode::Backtrack(Error::new("123123", ErrorKind::Tag))));
/// assert_eq!(parser(""), Err(ErrMode::Backtrack(Error::new("", ErrorKind::Tag))));
/// assert_eq!(parser("abcabcabc"), Ok(("abc", vec!["abc", "abc"])));
/// ```
pub fn count<I, O, C, E, F>(mut f: F, count: usize) -> impl FnMut(I) -> IResult<I, C, E>
where
    I: Stream,
    C: Accumulate<O>,
    F: Parser<I, O, E>,
    E: ParseError<I>,
{
    trace("count", move |i: I| {
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
                    return Err(e.append(i, ErrorKind::Count));
                }
            }
        }

        Ok((input, res))
    })
}

/// Runs the embedded parser repeatedly, filling the given slice with results.
///
/// This parser fails if the input runs out before the given slice is full.
///
/// # Arguments
/// * `f` The parser to apply.
/// * `buf` The slice to fill
/// ```rust
/// # use winnow::{error::ErrMode, error::{Error, ErrorKind}, error::Needed, IResult};
/// use winnow::multi::fill;
/// use winnow::bytes::tag;
///
/// fn parser(s: &str) -> IResult<&str, [&str; 2]> {
///   let mut buf = ["", ""];
///   let (rest, ()) = fill(tag("abc"), &mut buf)(s)?;
///   Ok((rest, buf))
/// }
///
/// assert_eq!(parser("abcabc"), Ok(("", ["abc", "abc"])));
/// assert_eq!(parser("abc123"), Err(ErrMode::Backtrack(Error::new("123", ErrorKind::Tag))));
/// assert_eq!(parser("123123"), Err(ErrMode::Backtrack(Error::new("123123", ErrorKind::Tag))));
/// assert_eq!(parser(""), Err(ErrMode::Backtrack(Error::new("", ErrorKind::Tag))));
/// assert_eq!(parser("abcabcabc"), Ok(("abc", ["abc", "abc"])));
/// ```
pub fn fill<'a, I, O, E, F>(mut f: F, buf: &'a mut [O]) -> impl FnMut(I) -> IResult<I, (), E> + 'a
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
                    return Err(e.append(i, ErrorKind::Count));
                }
            }
        }

        Ok((input, ()))
    })
}

/// Repeats the embedded parser, calling `g` to gather the results.
///
/// This stops on [`ErrMode::Backtrack`].  To instead chain an error up, see
/// [`cut_err`][crate::combinator::cut_err].
///
/// # Arguments
/// * `f` The parser to apply.
/// * `init` A function returning the initial value.
/// * `g` The function that combines a result of `f` with
///       the current accumulator.
///
/// *Note*: if the parser passed in accepts empty inputs (like `alpha0` or `digit0`), `many0` will
/// return an error, to prevent going into an infinite loop
///
/// ```rust
/// # use winnow::{error::ErrMode, error::ErrorKind, error::Needed, IResult};
/// use winnow::multi::fold_many0;
/// use winnow::bytes::tag;
///
/// fn parser(s: &str) -> IResult<&str, Vec<&str>> {
///   fold_many0(
///     tag("abc"),
///     Vec::new,
///     |mut acc: Vec<_>, item| {
///       acc.push(item);
///       acc
///     }
///   )(s)
/// }
///
/// assert_eq!(parser("abcabc"), Ok(("", vec!["abc", "abc"])));
/// assert_eq!(parser("abc123"), Ok(("123", vec!["abc"])));
/// assert_eq!(parser("123123"), Ok(("123123", vec![])));
/// assert_eq!(parser(""), Ok(("", vec![])));
/// ```
pub fn fold_many0<I, O, E, F, G, H, R>(
    mut f: F,
    mut init: H,
    mut g: G,
) -> impl FnMut(I) -> IResult<I, R, E>
where
    I: Stream,
    F: Parser<I, O, E>,
    G: FnMut(R, O) -> R,
    H: FnMut() -> R,
    E: ParseError<I>,
{
    trace("fold_many0", move |i: I| {
        let mut res = init();
        let mut input = i;

        loop {
            let i_ = input.clone();
            let len = input.eof_offset();
            match f.parse_next(i_) {
                Ok((i, o)) => {
                    // infinite loop check: the parser must always consume
                    if i.eof_offset() == len {
                        return Err(ErrMode::from_error_kind(input, ErrorKind::Many0));
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
    })
}

/// Repeats the embedded parser, calling `g` to gather the results.
///
/// This stops on [`ErrMode::Backtrack`] if there is at least one result.  To instead chain an error up,
/// see [`cut_err`][crate::combinator::cut_err].
///
/// # Arguments
/// * `f` The parser to apply.
/// * `init` A function returning the initial value.
/// * `g` The function that combines a result of `f` with
///       the current accumulator.
///
/// *Note*: If the parser passed to `many1` accepts empty inputs
/// (like `alpha0` or `digit0`), `many1` will return an error,
/// to prevent going into an infinite loop.
///
/// ```rust
/// # use winnow::{error::ErrMode, error::{Error, ErrorKind}, error::Needed, IResult};
/// use winnow::multi::fold_many1;
/// use winnow::bytes::tag;
///
/// fn parser(s: &str) -> IResult<&str, Vec<&str>> {
///   fold_many1(
///     tag("abc"),
///     Vec::new,
///     |mut acc: Vec<_>, item| {
///       acc.push(item);
///       acc
///     }
///   )(s)
/// }
///
/// assert_eq!(parser("abcabc"), Ok(("", vec!["abc", "abc"])));
/// assert_eq!(parser("abc123"), Ok(("123", vec!["abc"])));
/// assert_eq!(parser("123123"), Err(ErrMode::Backtrack(Error::new("123123", ErrorKind::Many1))));
/// assert_eq!(parser(""), Err(ErrMode::Backtrack(Error::new("", ErrorKind::Many1))));
/// ```
pub fn fold_many1<I, O, E, F, G, H, R>(
    mut f: F,
    mut init: H,
    mut g: G,
) -> impl FnMut(I) -> IResult<I, R, E>
where
    I: Stream,
    F: Parser<I, O, E>,
    G: FnMut(R, O) -> R,
    H: FnMut() -> R,
    E: ParseError<I>,
{
    trace("fold_many1", move |i: I| {
        let _i = i.clone();
        let init = init();
        match f.parse_next(_i) {
            Err(ErrMode::Backtrack(_)) => Err(ErrMode::from_error_kind(i, ErrorKind::Many1)),
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
                                return Err(ErrMode::Cut(E::from_error_kind(i, ErrorKind::Many1)));
                            }

                            acc = g(acc, o);
                            input = i;
                        }
                    }
                }

                Ok((input, acc))
            }
        }
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
/// *Note*: If the parser passed to `many1` accepts empty inputs
/// (like `alpha0` or `digit0`), `many1` will return an error,
/// to prevent going into an infinite loop.
///
/// ```rust
/// # use winnow::{error::ErrMode, error::ErrorKind, error::Needed, IResult};
/// use winnow::multi::fold_many_m_n;
/// use winnow::bytes::tag;
///
/// fn parser(s: &str) -> IResult<&str, Vec<&str>> {
///   fold_many_m_n(
///     0,
///     2,
///     tag("abc"),
///     Vec::new,
///     |mut acc: Vec<_>, item| {
///       acc.push(item);
///       acc
///     }
///   )(s)
/// }
///
/// assert_eq!(parser("abcabc"), Ok(("", vec!["abc", "abc"])));
/// assert_eq!(parser("abc123"), Ok(("123", vec!["abc"])));
/// assert_eq!(parser("123123"), Ok(("123123", vec![])));
/// assert_eq!(parser(""), Ok(("", vec![])));
/// assert_eq!(parser("abcabcabc"), Ok(("abc", vec!["abc", "abc"])));
/// ```
pub fn fold_many_m_n<I, O, E, F, G, H, R>(
    min: usize,
    max: usize,
    mut parse: F,
    mut init: H,
    mut fold: G,
) -> impl FnMut(I) -> IResult<I, R, E>
where
    I: Stream,
    F: Parser<I, O, E>,
    G: FnMut(R, O) -> R,
    H: FnMut() -> R,
    E: ParseError<I>,
{
    trace("fold_many_m_n", move |mut input: I| {
        if min > max {
            return Err(ErrMode::Cut(E::from_error_kind(input, ErrorKind::ManyMN)));
        }

        let mut acc = init();
        for count in 0..max {
            let len = input.eof_offset();
            match parse.parse_next(input.clone()) {
                Ok((tail, value)) => {
                    // infinite loop check: the parser must always consume
                    if tail.eof_offset() == len {
                        return Err(ErrMode::from_error_kind(tail, ErrorKind::ManyMN));
                    }

                    acc = fold(acc, value);
                    input = tail;
                }
                //FInputXMError: handle failure properly
                Err(ErrMode::Backtrack(err)) => {
                    if count < min {
                        return Err(ErrMode::Backtrack(err.append(input, ErrorKind::ManyMN)));
                    } else {
                        break;
                    }
                }
                Err(e) => return Err(e),
            }
        }

        Ok((input, acc))
    })
}

/// Gets a number from the parser and returns a
/// subslice of the input of that size.
///
/// *Complete version*: Returns an error if there is not enough input data.
///
/// *Partial version*: Will return `Err(winnow::error::ErrMode::Incomplete(_))` if there is not enough data.
///
/// # Arguments
/// * `f` The parser to apply.
/// ```rust
/// # use winnow::{error::ErrMode, error::ErrorKind, error::Needed, IResult, stream::Partial};
/// use winnow::Bytes;
/// use winnow::number::be_u16;
/// use winnow::multi::length_data;
/// use winnow::bytes::tag;
///
/// type Stream<'i> = Partial<&'i Bytes>;
///
/// fn stream(b: &[u8]) -> Stream<'_> {
///     Partial(Bytes::new(b))
/// }
///
/// fn parser(s: Stream<'_>) -> IResult<Stream<'_>, &Bytes> {
///   length_data(be_u16)(s)
/// }
///
/// assert_eq!(parser(stream(b"\x00\x03abcefg")), Ok((stream(&b"efg"[..]), Bytes::new(&b"abc"[..]))));
/// assert_eq!(parser(stream(b"\x00\x03a")), Err(ErrMode::Incomplete(Needed::new(2))));
/// ```
pub fn length_data<I, N, E, F, const PARTIAL: bool>(
    mut f: F,
) -> impl FnMut(I) -> IResult<I, <I as Stream>::Slice, E>
where
    I: StreamIsPartial<PARTIAL>,
    I: Stream,
    N: ToUsize,
    F: Parser<I, N, E>,
    E: ParseError<I>,
{
    trace("length_data", move |i: I| {
        let (i, length) = f.parse_next(i)?;

        crate::bytes::take(length).parse_next(i)
    })
}

/// Gets a number from the first parser,
/// takes a subslice of the input of that size,
/// then applies the second parser on that subslice.
/// If the second parser returns `Incomplete`,
/// `length_value` will return an error.
///
/// *Complete version*: Returns an error if there is not enough input data.
///
/// *Partial version*: Will return `Err(winnow::error::ErrMode::Incomplete(_))` if there is not enough data.
///
/// # Arguments
/// * `f` The parser to apply.
/// * `g` The parser to apply on the subslice.
/// ```rust
/// # use winnow::{error::ErrMode, error::{Error, ErrorKind}, error::Needed, IResult, stream::Partial};
/// use winnow::Bytes;
/// use winnow::number::be_u16;
/// use winnow::multi::length_value;
/// use winnow::bytes::tag;
///
/// type Stream<'i> = Partial<&'i Bytes>;
///
/// fn stream(b: &[u8]) -> Stream<'_> {
///     Partial(Bytes::new(b))
/// }
///
/// fn parser(s: Stream<'_>) -> IResult<Stream<'_>, &Bytes> {
///   length_value(be_u16, tag("abc"))(s)
/// }
///
/// assert_eq!(parser(stream(b"\x00\x03abcefg")), Ok((stream(&b"efg"[..]), Bytes::new(&b"abc"[..]))));
/// assert_eq!(parser(stream(b"\x00\x03123123")), Err(ErrMode::Backtrack(Error::new(stream(&b"123"[..]), ErrorKind::Tag))));
/// assert_eq!(parser(stream(b"\x00\x03a")), Err(ErrMode::Incomplete(Needed::new(2))));
/// ```
pub fn length_value<I, O, N, E, F, G, const PARTIAL: bool>(
    mut f: F,
    mut g: G,
) -> impl FnMut(I) -> IResult<I, O, E>
where
    I: StreamIsPartial<PARTIAL>,
    I: Stream + UpdateSlice,
    N: ToUsize,
    F: Parser<I, N, E>,
    G: Parser<I, O, E>,
    E: ParseError<I>,
{
    trace("length_value", move |i: I| {
        let (i, data) = length_data(f.by_ref()).parse_next(i)?;
        let data = I::update_slice(i.clone(), data);
        let (_, o) = g.by_ref().complete().parse_next(data)?;
        Ok((i, o))
    })
}

/// Gets a number from the first parser,
/// then applies the second parser that many times.
/// # Arguments
/// * `f` The parser to apply to obtain the count.
/// * `g` The parser to apply repeatedly.
#[cfg_attr(not(feature = "std"), doc = "```ignore")]
#[cfg_attr(feature = "std", doc = "```")]
/// # use winnow::prelude::*;
/// # use winnow::{error::ErrMode, error::{Error, ErrorKind}, error::Needed, IResult};
/// use winnow::Bytes;
/// use winnow::number::u8;
/// use winnow::multi::length_count;
/// use winnow::bytes::tag;
///
/// type Stream<'i> = &'i Bytes;
///
/// fn stream(b: &[u8]) -> Stream<'_> {
///     Bytes::new(b)
/// }
///
/// fn parser(s: Stream<'_>) -> IResult<Stream<'_>, Vec<&Bytes>> {
///   length_count(u8.map(|i| {
///      println!("got number: {}", i);
///      i
///   }), tag("abc"))(s)
/// }
///
/// assert_eq!(parser(stream(b"\x02abcabcabc")), Ok((stream(b"abc"), vec![Bytes::new(b"abc"), Bytes::new(b"abc")])));
/// assert_eq!(parser(stream(b"\x03123123123")), Err(ErrMode::Backtrack(Error::new(stream(b"123123123"), ErrorKind::Tag))));
/// ```
pub fn length_count<I, O, C, N, E, F, G>(mut f: F, mut g: G) -> impl FnMut(I) -> IResult<I, C, E>
where
    I: Stream,
    N: ToUsize,
    C: Accumulate<O>,
    F: Parser<I, N, E>,
    G: Parser<I, O, E>,
    E: ParseError<I>,
{
    trace("length_count", move |i: I| {
        let (i, count) = f.parse_next(i)?;
        let mut input = i.clone();
        let mut res = C::initial(Some(count.to_usize()));

        for _ in 0..count.to_usize() {
            let input_ = input.clone();
            match g.parse_next(input_) {
                Ok((i, o)) => {
                    res.accumulate(o);
                    input = i;
                }
                Err(e) => {
                    return Err(e.append(i, ErrorKind::Count));
                }
            }
        }

        Ok((input, res))
    })
}
