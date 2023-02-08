//! Input capability for nom combinators to parse
//!
//! Input types include:
//! - `&str` and `&[u8]` are the standard input types
//! - [`Located`] can track the location within the original buffer to report
//!   [spans][crate::Parser::with_span]
//! - [`Stateful`] to thread global state through your parsers
//! - [`Streaming`] can mark an input as partial buffer that is being streamed into
//!
//! # How do a parse a custom input type?
//!
//! While historically, nom has worked mainly on `&[u8]` and `&str`, it can actually
//! use any type as input, as long as they follow a specific set of traits.
//! Those traits were developed first to abstract away the differences between
//! `&[u8]` and `&str`, but were then employed for more interesting types,
//! like [`Located`], a wrapper type
//! that can carry line and column information, or to parse
//! [a list of tokens](https://github.com/Rydgel/monkey-rust/blob/master/lib/parser/mod.rs).
//!
//! ## Implementing a custom type
//!
//! Let's assume we have an input type we'll call `MyInput`. `MyInput` is a sequence of `MyItem` type.
//! The goal is to define nom parsers with this signature: `MyInput -> IResult<MyInput, Output>`.
//!
//! ```rust,ignore
//! fn parser(i: MyInput) -> IResult<MyInput, Output> {
//!     tag("test")(i)
//! }
//! ```
//!
//! Here are the traits we have to implement for `MyInput`:
//!
//! | trait | usage |
//! |---|---|
//! | [`Input`] |Core trait for driving parsing|
//! | [`InputIsStreaming`] | Marks the input as being the complete buffer or a partial buffer for streaming input |
//! | [`AsBytes`] |Casts the input type to a byte slice|
//! | [`AsBStr`] |Casts the input type to a slice of ASCII / UTF-8-like bytes|
//! | [`Compare`] |Character comparison operations|
//! | [`FindSlice`] |Look for a substring in self|
//! | [`Location`] |Calculate location within initial input|
//! | [`Offset`] |Calculate the offset between slices|
//! | [`HexDisplay`] |Debug dump of input|
//!
//! Here are the traits we have to implement for `MyItem`:
//!
//! | trait | usage |
//! |---|---|
//! | [`AsChar`] |Transforms common types to a char for basic token parsing|
//! | [`ContainsToken`] |Look for the token in the given set|
//!
//! And traits for slices of `MyItem`:
//!
//! | [`SliceLen`] |Calculate the input length|
//! | [`ParseSlice`] |Used to integrate `&str`'s `parse()` method|

use core::num::NonZeroUsize;

use crate::error::{ErrMode, ErrorKind, Needed, ParseError};
use crate::lib::std::iter::{Cloned, Enumerate};
use crate::lib::std::ops::{
    Range, RangeFrom, RangeFull, RangeInclusive, RangeTo, RangeToInclusive,
};
use crate::lib::std::slice::Iter;
use crate::lib::std::str::from_utf8;
use crate::lib::std::str::CharIndices;
use crate::lib::std::str::FromStr;
use crate::IResult;

#[cfg(feature = "alloc")]
use crate::lib::std::collections::BTreeMap;
#[cfg(feature = "std")]
use crate::lib::std::collections::HashMap;
#[cfg(feature = "alloc")]
use crate::lib::std::string::String;
#[cfg(feature = "alloc")]
use crate::lib::std::vec::Vec;

/// Allow collecting the span of a parsed token
///
/// See [`Parser::span`][crate::Parser::span] and [`Parser::with_span`][crate::Parser::with_span] for more details
#[derive(Copy, Clone, Default, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Located<I> {
    initial: I,
    input: I,
}

impl<I> Located<I>
where
    I: Clone + Offset,
{
    /// Wrap another Input with span tracking
    pub fn new(input: I) -> Self {
        let initial = input.clone();
        Self { initial, input }
    }

    fn location(&self) -> usize {
        self.initial.offset_to(&self.input)
    }
}

impl<I> AsRef<I> for Located<I> {
    fn as_ref(&self) -> &I {
        &self.input
    }
}

impl<I> crate::lib::std::ops::Deref for Located<I> {
    type Target = I;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.input
    }
}

/// Thread global state through your parsers
///
/// Use cases
/// - Recursion checks
/// - Error recovery
/// - Debugging
///
/// # Example
///
/// ```
/// # use std::cell::Cell;
/// # use winnow::prelude::*;
/// # use winnow::input::Stateful;
/// # use winnow::character::alpha1;
/// # type Error = ();
///
/// #[derive(Clone, Debug)]
/// struct State<'s>(&'s Cell<u32>);
///
/// impl<'s> State<'s> {
///     fn count(&self) {
///         self.0.set(self.0.get() + 1);
///     }
/// }
///
/// type Input<'is> = Stateful<&'is str, State<'is>>;
///
/// fn word(i: Input<'_>) -> IResult<Input<'_>, &str> {
///   i.state.count();
///   alpha1(i)
/// }
///
/// let data = "Hello";
/// let state = Cell::new(0);
/// let input = Input { input: data, state: State(&state) };
/// let output = word.parse_next(input).finish().unwrap();
/// assert_eq!(state.get(), 1);
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Stateful<I, S> {
    /// Inner input being wrapped in state
    pub input: I,
    /// User-provided state
    pub state: S,
}

impl<I, S> AsRef<I> for Stateful<I, S> {
    fn as_ref(&self) -> &I {
        &self.input
    }
}

impl<I, S> crate::lib::std::ops::Deref for Stateful<I, S> {
    type Target = I;

    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

/// Mark the input as a partial buffer for streaming input.
///
/// Complete input means that we already have all of the data.  This will be the common case with
/// small files that can be read entirely to memory.
///
/// In contrast, streaming input assumes that we might not have all of the data.
/// This can happen with some network protocol or large file parsers, where the
/// input buffer can be full and need to be resized or refilled.
/// - [`ErrMode::Incomplete`] will report how much more data is needed.
/// - [`Parser::complete`][crate::Parser::complete] transform [`ErrMode::Incomplete`] to
///   [`ErrMode::Backtrack`]
///
/// See also [`InputIsStreaming`] to tell whether the input supports complete or streaming parsing.
///
/// # Example
///
/// Here is how it works in practice:
///
/// ```rust
/// use winnow::{IResult, error::ErrMode, error::Needed, error::{Error, ErrorKind}, bytes, character, input::Streaming};
///
/// fn take_streaming(i: Streaming<&[u8]>) -> IResult<Streaming<&[u8]>, &[u8]> {
///   bytes::take(4u8)(i)
/// }
///
/// fn take_complete(i: &[u8]) -> IResult<&[u8], &[u8]> {
///   bytes::take(4u8)(i)
/// }
///
/// // both parsers will take 4 bytes as expected
/// assert_eq!(take_streaming(Streaming(&b"abcde"[..])), Ok((Streaming(&b"e"[..]), &b"abcd"[..])));
/// assert_eq!(take_complete(&b"abcde"[..]), Ok((&b"e"[..], &b"abcd"[..])));
///
/// // if the input is smaller than 4 bytes, the streaming parser
/// // will return `Incomplete` to indicate that we need more data
/// assert_eq!(take_streaming(Streaming(&b"abc"[..])), Err(ErrMode::Incomplete(Needed::new(1))));
///
/// // but the complete parser will return an error
/// assert_eq!(take_complete(&b"abc"[..]), Err(ErrMode::Backtrack(Error::new(&b"abc"[..], ErrorKind::Eof))));
///
/// // the alpha0 function recognizes 0 or more alphabetic characters
/// fn alpha0_streaming(i: Streaming<&str>) -> IResult<Streaming<&str>, &str> {
///   character::alpha0(i)
/// }
///
/// fn alpha0_complete(i: &str) -> IResult<&str, &str> {
///   character::alpha0(i)
/// }
///
/// // if there's a clear limit to the recognized characters, both parsers work the same way
/// assert_eq!(alpha0_streaming(Streaming("abcd;")), Ok((Streaming(";"), "abcd")));
/// assert_eq!(alpha0_complete("abcd;"), Ok((";", "abcd")));
///
/// // but when there's no limit, the streaming version returns `Incomplete`, because it cannot
/// // know if more input data should be recognized. The whole input could be "abcd;", or
/// // "abcde;"
/// assert_eq!(alpha0_streaming(Streaming("abcd")), Err(ErrMode::Incomplete(Needed::new(1))));
///
/// // while the complete version knows that all of the data is there
/// assert_eq!(alpha0_complete("abcd"), Ok(("", "abcd")));
/// ```
#[derive(Copy, Clone, Default, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Streaming<I>(pub I);

impl<I> Streaming<I> {
    /// Convert to complete counterpart
    #[inline(always)]
    pub fn into_complete(self) -> I {
        self.0
    }
}

impl<I> crate::lib::std::ops::Deref for Streaming<I> {
    type Target = I;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Abstract method to calculate the input length
pub trait SliceLen {
    /// Calculates the input length, as indicated by its name,
    /// and the name of the trait itself
    fn slice_len(&self) -> usize;
}

impl<I> SliceLen for Located<I>
where
    I: SliceLen,
{
    fn slice_len(&self) -> usize {
        self.input.slice_len()
    }
}

impl<I, S> SliceLen for Stateful<I, S>
where
    I: SliceLen,
{
    fn slice_len(&self) -> usize {
        self.input.slice_len()
    }
}

impl<I> SliceLen for Streaming<I>
where
    I: SliceLen,
{
    #[inline(always)]
    fn slice_len(&self) -> usize {
        self.0.slice_len()
    }
}

impl<'a, T> SliceLen for &'a [T] {
    #[inline]
    fn slice_len(&self) -> usize {
        self.len()
    }
}

impl<T, const LEN: usize> SliceLen for [T; LEN] {
    #[inline]
    fn slice_len(&self) -> usize {
        self.len()
    }
}

impl<'a, T, const LEN: usize> SliceLen for &'a [T; LEN] {
    #[inline]
    fn slice_len(&self) -> usize {
        self.len()
    }
}

impl<'a> SliceLen for &'a str {
    #[inline]
    fn slice_len(&self) -> usize {
        self.len()
    }
}

/// Core definition for parser input state
pub trait Input: Clone {
    /// The smallest unit being parsed
    ///
    /// Example: `u8` for `&[u8]` or `char` for `&str`
    type Token;
    /// Sequence of `Token`s
    ///
    /// Example: `&[u8]` for `Located<&[u8]>` or `&str` for `Located<&str>`
    type Slice;

    /// Iterate with the offset from the current location
    type IterOffsets: Iterator<Item = (usize, Self::Token)>;

    /// Iterate with the offset from the current location
    fn iter_offsets(&self) -> Self::IterOffsets;
    /// Returns the offaet to the end of the input
    fn input_len(&self) -> usize;

    /// Split off the next token from the input
    fn next_token(&self) -> Option<(Self, Self::Token)>;

    /// Finds the offset of the next matching token
    fn offset_for<P>(&self, predicate: P) -> Option<usize>
    where
        P: Fn(Self::Token) -> bool;
    /// Get the offset for the number of `tokens` into the stream
    ///
    /// This means "0 tokens" will return `0` offset
    fn offset_at(&self, tokens: usize) -> Result<usize, Needed>;
    /// Split off a slice of tokens from the input
    ///
    /// **NOTE:** For inputs with variable width tokens, like `&str`'s `char`, `offset` might not correspond
    /// with the number of tokens.  To get a valid offset, use:
    /// - [`Input::input_len`]
    /// - [`Input::iter_offsets`]
    /// - [`Input::offset_for`]
    /// - [`Input::offset_at`]
    ///
    /// # Panic
    ///
    /// This will panic if
    ///
    /// * Indexes must be within bounds of the original input;
    /// * Indexes must uphold invariants of the Input, like for `str` they must lie on UTF-8
    ///   sequence boundaries.
    ///
    fn next_slice(&self, offset: usize) -> (Self, Self::Slice);
}

impl<'i, T> Input for &'i [T]
where
    T: Clone,
{
    type Token = T;
    type Slice = &'i [T];

    type IterOffsets = Enumerate<Cloned<Iter<'i, T>>>;

    #[inline(always)]
    fn iter_offsets(&self) -> Self::IterOffsets {
        self.iter().cloned().enumerate()
    }
    #[inline(always)]
    fn input_len(&self) -> usize {
        self.len()
    }

    #[inline(always)]
    fn next_token(&self) -> Option<(Self, Self::Token)> {
        if self.is_empty() {
            None
        } else {
            Some((&self[1..], self[0].clone()))
        }
    }

    #[inline(always)]
    fn offset_for<P>(&self, predicate: P) -> Option<usize>
    where
        P: Fn(Self::Token) -> bool,
    {
        self.iter().position(|b| predicate(b.clone()))
    }
    #[inline(always)]
    fn offset_at(&self, tokens: usize) -> Result<usize, Needed> {
        if let Some(needed) = tokens.checked_sub(self.len()).and_then(NonZeroUsize::new) {
            Err(Needed::Size(needed))
        } else {
            Ok(tokens)
        }
    }
    #[inline(always)]
    fn next_slice(&self, offset: usize) -> (Self, Self::Slice) {
        (&self[offset..], &self[0..offset])
    }
}

impl<'i> Input for &'i str {
    type Token = char;
    type Slice = &'i str;

    type IterOffsets = CharIndices<'i>;

    #[inline(always)]
    fn iter_offsets(&self) -> Self::IterOffsets {
        self.char_indices()
    }
    #[inline(always)]
    fn input_len(&self) -> usize {
        self.len()
    }

    #[inline(always)]
    fn next_token(&self) -> Option<(Self, Self::Token)> {
        let c = self.chars().next()?;
        let offset = c.len();
        Some((&self[offset..], c))
    }

    #[inline(always)]
    fn offset_for<P>(&self, predicate: P) -> Option<usize>
    where
        P: Fn(Self::Token) -> bool,
    {
        for (o, c) in self.iter_offsets() {
            if predicate(c) {
                return Some(o);
            }
        }
        None
    }
    #[inline]
    fn offset_at(&self, tokens: usize) -> Result<usize, Needed> {
        let mut cnt = 0;
        for (offset, _) in self.iter_offsets() {
            if cnt == tokens {
                return Ok(offset);
            }
            cnt += 1;
        }

        if cnt == tokens {
            Ok(self.input_len())
        } else {
            Err(Needed::Unknown)
        }
    }
    #[inline(always)]
    fn next_slice(&self, offset: usize) -> (Self, Self::Slice) {
        (&self[offset..], &self[0..offset])
    }
}

impl<I: Input> Input for Located<I> {
    type Token = <I as Input>::Token;
    type Slice = <I as Input>::Slice;

    type IterOffsets = <I as Input>::IterOffsets;

    #[inline(always)]
    fn iter_offsets(&self) -> Self::IterOffsets {
        self.input.iter_offsets()
    }
    #[inline(always)]
    fn input_len(&self) -> usize {
        self.input.input_len()
    }

    #[inline(always)]
    fn next_token(&self) -> Option<(Self, Self::Token)> {
        let (next, token) = self.input.next_token()?;
        Some((
            Self {
                initial: self.initial.clone(),
                input: next,
            },
            token,
        ))
    }

    #[inline(always)]
    fn offset_for<P>(&self, predicate: P) -> Option<usize>
    where
        P: Fn(Self::Token) -> bool,
    {
        self.input.offset_for(predicate)
    }
    #[inline(always)]
    fn offset_at(&self, tokens: usize) -> Result<usize, Needed> {
        self.input.offset_at(tokens)
    }
    #[inline(always)]
    fn next_slice(&self, offset: usize) -> (Self, Self::Slice) {
        let (next, slice) = self.input.next_slice(offset);
        (
            Self {
                initial: self.initial.clone(),
                input: next,
            },
            slice,
        )
    }
}

impl<I: Input, S: Clone> Input for Stateful<I, S> {
    type Token = <I as Input>::Token;
    type Slice = <I as Input>::Slice;

    type IterOffsets = <I as Input>::IterOffsets;

    #[inline(always)]
    fn iter_offsets(&self) -> Self::IterOffsets {
        self.input.iter_offsets()
    }
    #[inline(always)]
    fn input_len(&self) -> usize {
        self.input.input_len()
    }

    #[inline(always)]
    fn next_token(&self) -> Option<(Self, Self::Token)> {
        let (next, token) = self.input.next_token()?;
        Some((
            Self {
                input: next,
                state: self.state.clone(),
            },
            token,
        ))
    }

    #[inline(always)]
    fn offset_for<P>(&self, predicate: P) -> Option<usize>
    where
        P: Fn(Self::Token) -> bool,
    {
        self.input.offset_for(predicate)
    }
    #[inline(always)]
    fn offset_at(&self, tokens: usize) -> Result<usize, Needed> {
        self.input.offset_at(tokens)
    }
    #[inline(always)]
    fn next_slice(&self, offset: usize) -> (Self, Self::Slice) {
        let (next, slice) = self.input.next_slice(offset);
        (
            Self {
                input: next,
                state: self.state.clone(),
            },
            slice,
        )
    }
}

impl<I: Input> Input for Streaming<I> {
    type Token = <I as Input>::Token;
    type Slice = <I as Input>::Slice;

    type IterOffsets = <I as Input>::IterOffsets;

    #[inline(always)]
    fn iter_offsets(&self) -> Self::IterOffsets {
        self.0.iter_offsets()
    }
    #[inline(always)]
    fn input_len(&self) -> usize {
        self.0.input_len()
    }

    #[inline(always)]
    fn next_token(&self) -> Option<(Self, Self::Token)> {
        let (next, token) = self.0.next_token()?;
        Some((Streaming(next), token))
    }

    #[inline(always)]
    fn offset_for<P>(&self, predicate: P) -> Option<usize>
    where
        P: Fn(Self::Token) -> bool,
    {
        self.0.offset_for(predicate)
    }
    #[inline(always)]
    fn offset_at(&self, tokens: usize) -> Result<usize, Needed> {
        self.0.offset_at(tokens)
    }
    #[inline(always)]
    fn next_slice(&self, offset: usize) -> (Self, Self::Slice) {
        let (next, slice) = self.0.next_slice(offset);
        (Streaming(next), slice)
    }
}

/// Number of indices input has advanced since start of parsing
pub trait Location {
    /// Number of indices input has advanced since start of parsing
    fn location(&self) -> usize;
}

impl<I> Location for Located<I>
where
    I: Clone + Offset,
{
    fn location(&self) -> usize {
        self.location()
    }
}

impl<I, S> Location for Stateful<I, S>
where
    I: Location,
{
    fn location(&self) -> usize {
        self.input.location()
    }
}

impl<I> Location for Streaming<I>
where
    I: Location,
{
    fn location(&self) -> usize {
        self.0.location()
    }
}

/// Marks the input as being the complete buffer or a partial buffer for streaming input
///
/// See [Streaming] for marking a presumed complete buffer type as a streaming buffer.
pub trait InputIsStreaming<const YES: bool>: Sized {
    /// Complete counterpart
    ///
    /// - Set to `Self` if this is a complete buffer.
    /// - Set to [`std::convert::Infallible`] if there isn't an associated complete buffer type
    type Complete: InputIsStreaming<false>;
    /// Streaming counterpart
    ///
    /// - Set to `Self` if this is a streaming buffer.
    /// - Set to [`std::convert::Infallible`] if there isn't an associated streaming buffer type
    type Streaming: InputIsStreaming<true>;

    /// Convert to complete counterpart
    fn into_complete(self) -> Self::Complete;
    /// Convert to streaming counterpart
    fn into_streaming(self) -> Self::Streaming;
}

impl<'a, T> InputIsStreaming<false> for &'a [T] {
    type Complete = Self;
    type Streaming = Streaming<Self>;

    #[inline(always)]
    fn into_complete(self) -> Self::Complete {
        self
    }
    #[inline(always)]
    fn into_streaming(self) -> Self::Streaming {
        Streaming(self)
    }
}

impl<'a> InputIsStreaming<false> for &'a str {
    type Complete = Self;
    type Streaming = Streaming<Self>;

    #[inline(always)]
    fn into_complete(self) -> Self::Complete {
        self
    }
    #[inline(always)]
    fn into_streaming(self) -> Self::Streaming {
        Streaming(self)
    }
}

impl<const YES: bool> InputIsStreaming<YES> for crate::lib::std::convert::Infallible {
    type Complete = Self;
    type Streaming = Self;

    #[inline(always)]
    fn into_complete(self) -> Self::Complete {
        self
    }
    #[inline(always)]
    fn into_streaming(self) -> Self::Streaming {
        self
    }
}

impl<I> InputIsStreaming<true> for Located<I>
where
    I: InputIsStreaming<true>,
{
    type Complete = Located<<I as InputIsStreaming<true>>::Complete>;
    type Streaming = Self;

    #[inline(always)]
    fn into_complete(self) -> Self::Complete {
        Located {
            initial: self.initial.into_complete(),
            input: self.input.into_complete(),
        }
    }
    #[inline(always)]
    fn into_streaming(self) -> Self::Streaming {
        self
    }
}

impl<I> InputIsStreaming<false> for Located<I>
where
    I: InputIsStreaming<false>,
{
    type Complete = Self;
    type Streaming = Located<<I as InputIsStreaming<false>>::Streaming>;

    #[inline(always)]
    fn into_complete(self) -> Self::Complete {
        self
    }
    #[inline(always)]
    fn into_streaming(self) -> Self::Streaming {
        Located {
            initial: self.initial.into_streaming(),
            input: self.input.into_streaming(),
        }
    }
}

impl<I, S> InputIsStreaming<true> for Stateful<I, S>
where
    I: InputIsStreaming<true>,
{
    type Complete = Stateful<<I as InputIsStreaming<true>>::Complete, S>;
    type Streaming = Self;

    #[inline(always)]
    fn into_complete(self) -> Self::Complete {
        Stateful {
            input: self.input.into_complete(),
            state: self.state,
        }
    }
    #[inline(always)]
    fn into_streaming(self) -> Self::Streaming {
        self
    }
}

impl<I, S> InputIsStreaming<false> for Stateful<I, S>
where
    I: InputIsStreaming<false>,
{
    type Complete = Self;
    type Streaming = Stateful<<I as InputIsStreaming<false>>::Streaming, S>;

    #[inline(always)]
    fn into_complete(self) -> Self::Complete {
        self
    }
    #[inline(always)]
    fn into_streaming(self) -> Self::Streaming {
        Stateful {
            input: self.input.into_streaming(),
            state: self.state,
        }
    }
}

impl<I> InputIsStreaming<true> for Streaming<I>
where
    I: InputIsStreaming<false>,
{
    type Complete = I;
    type Streaming = Self;

    #[inline(always)]
    fn into_complete(self) -> Self::Complete {
        self.0
    }
    #[inline(always)]
    fn into_streaming(self) -> Self::Streaming {
        self
    }
}

/// Useful functions to calculate the offset between slices and show a hexdump of a slice
pub trait Offset {
    /// Offset between the first byte of self and the first byte of the argument
    fn offset_to(&self, second: &Self) -> usize;
}

impl<'a, T> Offset for &'a [T] {
    fn offset_to(&self, second: &Self) -> usize {
        let fst = self.as_ptr();
        let snd = second.as_ptr();

        debug_assert!(
            fst <= snd,
            "`Offset::offset_to` only accepts slices of `self`"
        );
        snd as usize - fst as usize
    }
}

/// Convenience implementation to accept `&[T]` instead of `&&[T]` as above
impl<T> Offset for [T] {
    fn offset_to(&self, second: &Self) -> usize {
        let fst = self.as_ptr();
        let snd = second.as_ptr();

        debug_assert!(
            fst <= snd,
            "`Offset::offset_to` only accepts slices of `self`"
        );
        snd as usize - fst as usize
    }
}

impl<'a> Offset for &'a str {
    fn offset_to(&self, second: &Self) -> usize {
        let fst = self.as_ptr();
        let snd = second.as_ptr();

        debug_assert!(
            fst <= snd,
            "`Offset::offset_to` only accepts slices of `self`"
        );
        snd as usize - fst as usize
    }
}

/// Convenience implementation to accept `&str` instead of `&&str` as above
impl Offset for str {
    fn offset_to(&self, second: &Self) -> usize {
        let fst = self.as_ptr();
        let snd = second.as_ptr();

        debug_assert!(
            fst <= snd,
            "`Offset::offset_to` only accepts slices of `self`"
        );
        snd as usize - fst as usize
    }
}

impl<I> Offset for Located<I>
where
    I: Offset,
{
    fn offset_to(&self, other: &Self) -> usize {
        self.input.offset_to(&other.input)
    }
}

impl<I, S> Offset for Stateful<I, S>
where
    I: Offset,
{
    fn offset_to(&self, other: &Self) -> usize {
        self.input.offset_to(&other.input)
    }
}

impl<I> Offset for Streaming<I>
where
    I: Offset,
{
    #[inline(always)]
    fn offset_to(&self, second: &Self) -> usize {
        self.0.offset_to(&second.0)
    }
}

/// Helper trait for types that can be viewed as a byte slice
pub trait AsBytes {
    /// Casts the input type to a byte slice
    fn as_bytes(&self) -> &[u8];
}

impl<'a> AsBytes for &'a [u8] {
    #[inline(always)]
    fn as_bytes(&self) -> &[u8] {
        self
    }
}

impl<I> AsBytes for Located<I>
where
    I: AsBytes,
{
    fn as_bytes(&self) -> &[u8] {
        self.input.as_bytes()
    }
}

impl<I, S> AsBytes for Stateful<I, S>
where
    I: AsBytes,
{
    fn as_bytes(&self) -> &[u8] {
        self.input.as_bytes()
    }
}

impl<I> AsBytes for Streaming<I>
where
    I: AsBytes,
{
    #[inline(always)]
    fn as_bytes(&self) -> &[u8] {
        self.0.as_bytes()
    }
}

/// Helper trait for types that can be viewed as a byte slice
pub trait AsBStr {
    /// Casts the input type to a byte slice
    fn as_bstr(&self) -> &[u8];
}

impl<'a> AsBStr for &'a [u8] {
    #[inline(always)]
    fn as_bstr(&self) -> &[u8] {
        self
    }
}

impl<'a> AsBStr for &'a str {
    #[inline(always)]
    fn as_bstr(&self) -> &[u8] {
        (*self).as_bytes()
    }
}

impl<I> AsBStr for Located<I>
where
    I: AsBStr,
{
    fn as_bstr(&self) -> &[u8] {
        self.input.as_bstr()
    }
}

impl<I, S> AsBStr for Stateful<I, S>
where
    I: AsBStr,
{
    fn as_bstr(&self) -> &[u8] {
        self.input.as_bstr()
    }
}

impl<I> AsBStr for Streaming<I>
where
    I: AsBStr,
{
    #[inline(always)]
    fn as_bstr(&self) -> &[u8] {
        self.0.as_bstr()
    }
}

/// Indicates whether a comparison was successful, an error, or
/// if more data was needed
#[derive(Debug, Eq, PartialEq)]
pub enum CompareResult {
    /// Comparison was successful
    Ok,
    /// We need more data to be sure
    Incomplete,
    /// Comparison failed
    Error,
}

/// Abstracts comparison operations
pub trait Compare<T> {
    /// Compares self to another value for equality
    fn compare(&self, t: T) -> CompareResult;
    /// Compares self to another value for equality
    /// independently of the case.
    ///
    /// Warning: for `&str`, the comparison is done
    /// by lowercasing both strings and comparing
    /// the result. This is a temporary solution until
    /// a better one appears
    fn compare_no_case(&self, t: T) -> CompareResult;
}

fn lowercase_byte(c: u8) -> u8 {
    match c {
        b'A'..=b'Z' => c - b'A' + b'a',
        _ => c,
    }
}

impl<'a, 'b> Compare<&'b [u8]> for &'a [u8] {
    #[inline(always)]
    fn compare(&self, t: &'b [u8]) -> CompareResult {
        let pos = self.iter().zip(t.iter()).position(|(a, b)| a != b);

        match pos {
            Some(_) => CompareResult::Error,
            None => {
                if self.len() >= t.len() {
                    CompareResult::Ok
                } else {
                    CompareResult::Incomplete
                }
            }
        }
    }

    #[inline(always)]
    fn compare_no_case(&self, t: &'b [u8]) -> CompareResult {
        if self
            .iter()
            .zip(t)
            .any(|(a, b)| lowercase_byte(*a) != lowercase_byte(*b))
        {
            CompareResult::Error
        } else if self.len() < t.len() {
            CompareResult::Incomplete
        } else {
            CompareResult::Ok
        }
    }
}

impl<'a, 'b, const LEN: usize> Compare<&'b [u8; LEN]> for &'a [u8] {
    #[inline(always)]
    fn compare(&self, t: &'b [u8; LEN]) -> CompareResult {
        self.compare(&t[..])
    }

    #[inline(always)]
    fn compare_no_case(&self, t: &'b [u8; LEN]) -> CompareResult {
        self.compare_no_case(&t[..])
    }
}

impl<'a, 'b> Compare<&'b str> for &'a [u8] {
    #[inline(always)]
    fn compare(&self, t: &'b str) -> CompareResult {
        self.compare(t.as_bytes())
    }
    #[inline(always)]
    fn compare_no_case(&self, t: &'b str) -> CompareResult {
        self.compare_no_case(t.as_bytes())
    }
}

impl<'a, 'b> Compare<&'b str> for &'a str {
    #[inline(always)]
    fn compare(&self, t: &'b str) -> CompareResult {
        self.as_bytes().compare(t.as_bytes())
    }

    //FIXME: this version is too simple and does not use the current locale
    #[inline(always)]
    fn compare_no_case(&self, t: &'b str) -> CompareResult {
        let pos = self
            .chars()
            .zip(t.chars())
            .position(|(a, b)| a.to_lowercase().ne(b.to_lowercase()));

        match pos {
            Some(_) => CompareResult::Error,
            None => {
                if self.len() >= t.len() {
                    CompareResult::Ok
                } else {
                    CompareResult::Incomplete
                }
            }
        }
    }
}

impl<'a, 'b> Compare<&'b [u8]> for &'a str {
    #[inline(always)]
    fn compare(&self, t: &'b [u8]) -> CompareResult {
        AsBStr::as_bstr(self).compare(t)
    }
    #[inline(always)]
    fn compare_no_case(&self, t: &'b [u8]) -> CompareResult {
        AsBStr::as_bstr(self).compare_no_case(t)
    }
}

impl<'a, const LEN: usize> Compare<[u8; LEN]> for &'a [u8] {
    #[inline(always)]
    fn compare(&self, t: [u8; LEN]) -> CompareResult {
        self.compare(&t[..])
    }

    #[inline(always)]
    fn compare_no_case(&self, t: [u8; LEN]) -> CompareResult {
        self.compare_no_case(&t[..])
    }
}

impl<I, U> Compare<U> for Located<I>
where
    I: Compare<U>,
{
    fn compare(&self, other: U) -> CompareResult {
        self.input.compare(other)
    }

    fn compare_no_case(&self, other: U) -> CompareResult {
        self.input.compare_no_case(other)
    }
}

impl<I, S, U> Compare<U> for Stateful<I, S>
where
    I: Compare<U>,
{
    fn compare(&self, other: U) -> CompareResult {
        self.input.compare(other)
    }

    fn compare_no_case(&self, other: U) -> CompareResult {
        self.input.compare_no_case(other)
    }
}

impl<I, T> Compare<T> for Streaming<I>
where
    I: Compare<T>,
{
    #[inline(always)]
    fn compare(&self, t: T) -> CompareResult {
        self.0.compare(t)
    }

    #[inline(always)]
    fn compare_no_case(&self, t: T) -> CompareResult {
        self.0.compare_no_case(t)
    }
}

/// Look for a slice in self
pub trait FindSlice<T> {
    /// Returns the offset of the slice if it is found
    fn find_slice(&self, substr: T) -> Option<usize>;
}

impl<'i, 's> FindSlice<&'s [u8]> for &'i [u8] {
    fn find_slice(&self, substr: &'s [u8]) -> Option<usize> {
        memchr::memmem::find(self, substr)
    }
}

impl<'i> FindSlice<u8> for &'i [u8] {
    fn find_slice(&self, substr: u8) -> Option<usize> {
        memchr::memchr(substr, self)
    }
}

impl<'i, 's> FindSlice<&'s str> for &'i [u8] {
    fn find_slice(&self, substr: &'s str) -> Option<usize> {
        self.find_slice(substr.as_bytes())
    }
}

impl<'i, 's> FindSlice<&'s str> for &'i str {
    fn find_slice(&self, substr: &'s str) -> Option<usize> {
        self.find(substr)
    }
}

impl<'i> FindSlice<char> for &'i str {
    fn find_slice(&self, substr: char) -> Option<usize> {
        self.find(substr)
    }
}

impl<I, T> FindSlice<T> for Located<I>
where
    I: FindSlice<T>,
{
    #[inline(always)]
    fn find_slice(&self, substr: T) -> Option<usize> {
        self.input.find_slice(substr)
    }
}

impl<I, S, T> FindSlice<T> for Stateful<I, S>
where
    I: FindSlice<T>,
{
    #[inline(always)]
    fn find_slice(&self, substr: T) -> Option<usize> {
        self.input.find_slice(substr)
    }
}

impl<I, T> FindSlice<T> for Streaming<I>
where
    I: FindSlice<T>,
{
    #[inline(always)]
    fn find_slice(&self, substr: T) -> Option<usize> {
        self.0.find_slice(substr)
    }
}

/// Used to integrate `str`'s `parse()` method
pub trait ParseSlice<R> {
    /// Succeeds if `parse()` succeeded. The byte slice implementation
    /// will first convert it to a `&str`, then apply the `parse()` function
    fn parse_slice(&self) -> Option<R>;
}

impl<'a, R: FromStr> ParseSlice<R> for &'a [u8] {
    fn parse_slice(&self) -> Option<R> {
        from_utf8(self).ok().and_then(|s| s.parse().ok())
    }
}

impl<'a, R: FromStr> ParseSlice<R> for &'a str {
    fn parse_slice(&self) -> Option<R> {
        self.parse().ok()
    }
}

/// Convert an `Input` into an appropriate `Output` type
pub trait UpdateSlice: Input {
    /// Convert an `Output` type to be used as `Input`
    fn update_slice(self, inner: Self::Slice) -> Self;
}

impl<'a, T> UpdateSlice for &'a [T]
where
    T: Clone,
{
    #[inline]
    fn update_slice(self, inner: Self::Slice) -> Self {
        inner
    }
}

impl<'a> UpdateSlice for &'a str {
    #[inline]
    fn update_slice(self, inner: Self::Slice) -> Self {
        inner
    }
}

impl<I> UpdateSlice for Located<I>
where
    I: UpdateSlice,
{
    #[inline]
    fn update_slice(mut self, inner: Self::Slice) -> Self {
        self.input = I::update_slice(self.input, inner);
        self
    }
}

impl<I, S> UpdateSlice for Stateful<I, S>
where
    I: UpdateSlice,
    S: Clone,
{
    #[inline]
    fn update_slice(mut self, inner: Self::Slice) -> Self {
        self.input = I::update_slice(self.input, inner);
        self
    }
}

impl<I> UpdateSlice for Streaming<I>
where
    I: UpdateSlice,
{
    #[inline]
    fn update_slice(self, inner: Self::Slice) -> Self {
        Streaming(I::update_slice(self.0, inner))
    }
}

/// Abstracts something which can extend an `Extend`.
/// Used to build modified input slices in `escaped_transform`
pub trait Accumulate<T>: Sized {
    /// Create a new `Extend` of the correct type
    fn initial(capacity: Option<usize>) -> Self;
    /// Accumulate the input into an accumulator
    fn accumulate(&mut self, acc: T);
}

impl<T> Accumulate<T> for () {
    #[inline(always)]
    fn initial(_capacity: Option<usize>) -> Self {}
    #[inline(always)]
    fn accumulate(&mut self, _acc: T) {}
}

impl<T> Accumulate<T> for usize {
    #[inline(always)]
    fn initial(_capacity: Option<usize>) -> Self {
        0
    }
    #[inline(always)]
    fn accumulate(&mut self, _acc: T) {
        *self += 1;
    }
}

#[cfg(feature = "alloc")]
impl<T> Accumulate<T> for Vec<T> {
    #[inline(always)]
    fn initial(capacity: Option<usize>) -> Self {
        match capacity {
            Some(capacity) => Vec::with_capacity(clamp_capacity::<T>(capacity)),
            None => Vec::new(),
        }
    }
    #[inline(always)]
    fn accumulate(&mut self, acc: T) {
        self.push(acc);
    }
}

#[cfg(feature = "alloc")]
impl<'i, T: Clone> Accumulate<&'i [T]> for Vec<T> {
    #[inline(always)]
    fn initial(capacity: Option<usize>) -> Self {
        match capacity {
            Some(capacity) => Vec::with_capacity(clamp_capacity::<T>(capacity)),
            None => Vec::new(),
        }
    }
    #[inline(always)]
    fn accumulate(&mut self, acc: &'i [T]) {
        self.extend(acc.iter().cloned());
    }
}

#[cfg(feature = "alloc")]
impl Accumulate<char> for String {
    #[inline(always)]
    fn initial(capacity: Option<usize>) -> Self {
        match capacity {
            Some(capacity) => String::with_capacity(clamp_capacity::<char>(capacity)),
            None => String::new(),
        }
    }
    #[inline(always)]
    fn accumulate(&mut self, acc: char) {
        self.push(acc);
    }
}

#[cfg(feature = "alloc")]
impl<'i> Accumulate<&'i str> for String {
    #[inline(always)]
    fn initial(capacity: Option<usize>) -> Self {
        match capacity {
            Some(capacity) => String::with_capacity(clamp_capacity::<char>(capacity)),
            None => String::new(),
        }
    }
    #[inline(always)]
    fn accumulate(&mut self, acc: &'i str) {
        self.push_str(acc);
    }
}

#[cfg(feature = "alloc")]
impl<K, V> Accumulate<(K, V)> for BTreeMap<K, V>
where
    K: crate::lib::std::cmp::Ord,
{
    #[inline(always)]
    fn initial(_capacity: Option<usize>) -> Self {
        BTreeMap::new()
    }
    #[inline(always)]
    fn accumulate(&mut self, (key, value): (K, V)) {
        self.insert(key, value);
    }
}

#[cfg(feature = "std")]
impl<K, V> Accumulate<(K, V)> for HashMap<K, V>
where
    K: crate::lib::std::cmp::Eq + crate::lib::std::hash::Hash,
{
    #[inline(always)]
    fn initial(capacity: Option<usize>) -> Self {
        match capacity {
            Some(capacity) => HashMap::with_capacity(clamp_capacity::<(K, V)>(capacity)),
            None => HashMap::new(),
        }
    }
    #[inline(always)]
    fn accumulate(&mut self, (key, value): (K, V)) {
        self.insert(key, value);
    }
}

#[cfg(feature = "alloc")]
#[inline]
pub(crate) fn clamp_capacity<T>(capacity: usize) -> usize {
    /// Don't pre-allocate more than 64KiB when calling `Vec::with_capacity`.
    ///
    /// Pre-allocating memory is a nice optimization but count fields can't
    /// always be trusted. We should clamp initial capacities to some reasonable
    /// amount. This reduces the risk of a bogus count value triggering a panic
    /// due to an OOM error.
    ///
    /// This does not affect correctness. Nom will always read the full number
    /// of elements regardless of the capacity cap.
    const MAX_INITIAL_CAPACITY_BYTES: usize = 65536;

    let max_initial_capacity =
        MAX_INITIAL_CAPACITY_BYTES / crate::lib::std::mem::size_of::<T>().max(1);
    capacity.min(max_initial_capacity)
}

/// Helper trait to convert numbers to usize.
///
/// By default, usize implements `From<u8>` and `From<u16>` but not
/// `From<u32>` and `From<u64>` because that would be invalid on some
/// platforms. This trait implements the conversion for platforms
/// with 32 and 64 bits pointer platforms
pub trait ToUsize {
    /// converts self to usize
    fn to_usize(&self) -> usize;
}

impl ToUsize for u8 {
    #[inline]
    fn to_usize(&self) -> usize {
        *self as usize
    }
}

impl ToUsize for u16 {
    #[inline]
    fn to_usize(&self) -> usize {
        *self as usize
    }
}

impl ToUsize for usize {
    #[inline]
    fn to_usize(&self) -> usize {
        *self
    }
}

#[cfg(any(target_pointer_width = "32", target_pointer_width = "64"))]
impl ToUsize for u32 {
    #[inline]
    fn to_usize(&self) -> usize {
        *self as usize
    }
}

#[cfg(target_pointer_width = "64")]
impl ToUsize for u64 {
    #[inline]
    fn to_usize(&self) -> usize {
        *self as usize
    }
}

/// Helper trait to show a byte slice as a hex dump
#[cfg(feature = "std")]
pub trait HexDisplay {
    /// Converts the value of `self` to a hex dump, returning the owned
    /// `String`.
    fn to_hex(&self, chunk_size: usize) -> String;

    /// Converts the value of `self` to a hex dump beginning at `from` address, returning the owned
    /// `String`.
    fn to_hex_from(&self, chunk_size: usize, from: usize) -> String;
}

#[cfg(feature = "std")]
static CHARS: &[u8] = b"0123456789abcdef";

#[cfg(feature = "std")]
impl HexDisplay for &'_ [u8] {
    #[allow(unused_variables)]
    fn to_hex(&self, chunk_size: usize) -> String {
        self.to_hex_from(chunk_size, 0)
    }

    #[allow(unused_variables)]
    fn to_hex_from(&self, chunk_size: usize, from: usize) -> String {
        let mut v = Vec::with_capacity(self.len() * 3);
        let mut i = from;
        for chunk in self.chunks(chunk_size) {
            let s = format!("{:08x}", i);
            for &ch in s.as_bytes().iter() {
                v.push(ch);
            }
            v.push(b'\t');

            i += chunk_size;

            for &byte in chunk {
                v.push(CHARS[(byte >> 4) as usize]);
                v.push(CHARS[(byte & 0xf) as usize]);
                v.push(b' ');
            }
            if chunk_size > chunk.len() {
                for j in 0..(chunk_size - chunk.len()) {
                    v.push(b' ');
                    v.push(b' ');
                    v.push(b' ');
                }
            }
            v.push(b'\t');

            for &byte in chunk {
                if matches!(byte, 32..=126 | 128..=255) {
                    v.push(byte);
                } else {
                    v.push(b'.');
                }
            }
            v.push(b'\n');
        }

        String::from_utf8_lossy(&v[..]).into_owned()
    }
}

#[cfg(feature = "std")]
impl HexDisplay for &'_ str {
    #[allow(unused_variables)]
    fn to_hex(&self, chunk_size: usize) -> String {
        self.to_hex_from(chunk_size, 0)
    }

    #[allow(unused_variables)]
    fn to_hex_from(&self, chunk_size: usize, from: usize) -> String {
        self.as_bytes().to_hex_from(chunk_size, from)
    }
}

#[cfg(feature = "std")]
impl<I> HexDisplay for Located<I>
where
    I: HexDisplay,
{
    #[inline(always)]
    fn to_hex(&self, chunk_size: usize) -> String {
        self.input.to_hex(chunk_size)
    }

    #[inline(always)]
    fn to_hex_from(&self, chunk_size: usize, from: usize) -> String {
        self.input.to_hex_from(chunk_size, from)
    }
}

#[cfg(feature = "std")]
impl<I, S> HexDisplay for Stateful<I, S>
where
    I: HexDisplay,
{
    #[inline(always)]
    fn to_hex(&self, chunk_size: usize) -> String {
        self.input.to_hex(chunk_size)
    }

    #[inline(always)]
    fn to_hex_from(&self, chunk_size: usize, from: usize) -> String {
        self.input.to_hex_from(chunk_size, from)
    }
}

#[cfg(feature = "std")]
impl<I> HexDisplay for Streaming<I>
where
    I: HexDisplay,
{
    #[inline(always)]
    fn to_hex(&self, chunk_size: usize) -> String {
        self.0.to_hex(chunk_size)
    }

    #[inline(always)]
    fn to_hex_from(&self, chunk_size: usize, from: usize) -> String {
        self.0.to_hex_from(chunk_size, from)
    }
}

/// Transforms common types to a char for basic token parsing
#[allow(clippy::len_without_is_empty)]
#[allow(clippy::wrong_self_convention)]
pub trait AsChar {
    /// Makes a char from self
    ///
    /// ```
    /// use winnow::input::AsChar as _;
    ///
    /// assert_eq!('a'.as_char(), 'a');
    /// assert_eq!(u8::MAX.as_char(), std::char::from_u32(u8::MAX as u32).unwrap());
    /// ```
    fn as_char(self) -> char;

    /// Tests that self is an alphabetic character
    ///
    /// Warning: for `&str` it recognizes alphabetic
    /// characters outside of the 52 ASCII letters
    fn is_alpha(self) -> bool;

    /// Tests that self is an alphabetic character
    /// or a decimal digit
    fn is_alphanum(self) -> bool;
    /// Tests that self is a decimal digit
    fn is_dec_digit(self) -> bool;
    /// Tests that self is an hex digit
    fn is_hex_digit(self) -> bool;
    /// Tests that self is an octal digit
    fn is_oct_digit(self) -> bool;
    /// Gets the len in bytes for self
    fn len(self) -> usize;
    /// Tests that self is ASCII space or tab
    fn is_space(self) -> bool;
    /// Tests if byte is ASCII newline: \n
    fn is_newline(self) -> bool;
}

impl AsChar for u8 {
    #[inline]
    fn as_char(self) -> char {
        self as char
    }
    #[inline]
    fn is_alpha(self) -> bool {
        matches!(self, 0x41..=0x5A | 0x61..=0x7A)
    }
    #[inline]
    fn is_alphanum(self) -> bool {
        self.is_alpha() || self.is_dec_digit()
    }
    #[inline]
    fn is_dec_digit(self) -> bool {
        matches!(self, 0x30..=0x39)
    }
    #[inline]
    fn is_hex_digit(self) -> bool {
        matches!(self, 0x30..=0x39 | 0x41..=0x46 | 0x61..=0x66)
    }
    #[inline]
    fn is_oct_digit(self) -> bool {
        matches!(self, 0x30..=0x37)
    }
    #[inline]
    fn len(self) -> usize {
        1
    }
    #[inline]
    fn is_space(self) -> bool {
        self == b' ' || self == b'\t'
    }
    fn is_newline(self) -> bool {
        self == b'\n'
    }
}
impl<'a> AsChar for &'a u8 {
    #[inline]
    fn as_char(self) -> char {
        *self as char
    }
    #[inline]
    fn is_alpha(self) -> bool {
        matches!(*self, 0x41..=0x5A | 0x61..=0x7A)
    }
    #[inline]
    fn is_alphanum(self) -> bool {
        self.is_alpha() || self.is_dec_digit()
    }
    #[inline]
    fn is_dec_digit(self) -> bool {
        matches!(*self, 0x30..=0x39)
    }
    #[inline]
    fn is_hex_digit(self) -> bool {
        matches!(*self, 0x30..=0x39 | 0x41..=0x46 | 0x61..=0x66)
    }
    #[inline]
    fn is_oct_digit(self) -> bool {
        matches!(*self, 0x30..=0x37)
    }
    #[inline]
    fn len(self) -> usize {
        1
    }
    #[inline]
    fn is_space(self) -> bool {
        *self == b' ' || *self == b'\t'
    }
    fn is_newline(self) -> bool {
        *self == b'\n'
    }
}

impl AsChar for char {
    #[inline]
    fn as_char(self) -> char {
        self
    }
    #[inline]
    fn is_alpha(self) -> bool {
        self.is_ascii_alphabetic()
    }
    #[inline]
    fn is_alphanum(self) -> bool {
        self.is_alpha() || self.is_dec_digit()
    }
    #[inline]
    fn is_dec_digit(self) -> bool {
        self.is_ascii_digit()
    }
    #[inline]
    fn is_hex_digit(self) -> bool {
        self.is_ascii_hexdigit()
    }
    #[inline]
    fn is_oct_digit(self) -> bool {
        self.is_digit(8)
    }
    #[inline]
    fn len(self) -> usize {
        self.len_utf8()
    }
    #[inline]
    fn is_space(self) -> bool {
        self == ' ' || self == '\t'
    }
    fn is_newline(self) -> bool {
        self == '\n'
    }
}

impl<'a> AsChar for &'a char {
    #[inline]
    fn as_char(self) -> char {
        *self
    }
    #[inline]
    fn is_alpha(self) -> bool {
        self.is_ascii_alphabetic()
    }
    #[inline]
    fn is_alphanum(self) -> bool {
        self.is_alpha() || self.is_dec_digit()
    }
    #[inline]
    fn is_dec_digit(self) -> bool {
        self.is_ascii_digit()
    }
    #[inline]
    fn is_hex_digit(self) -> bool {
        self.is_ascii_hexdigit()
    }
    #[inline]
    fn is_oct_digit(self) -> bool {
        self.is_digit(8)
    }
    #[inline]
    fn len(self) -> usize {
        self.len_utf8()
    }
    #[inline]
    fn is_space(self) -> bool {
        *self == ' ' || *self == '\t'
    }
    fn is_newline(self) -> bool {
        *self == '\n'
    }
}

/// Check if a token in in a set of possible tokens
///
/// This is generally implemented on patterns that a token may match and supports `u8` and `char`
/// tokens along with the following patterns
/// - `b'c'` and `'c'`
/// - `b""` and `""`
/// - `|c| true`
/// - `b'a'..=b'z'`, `'a'..='z'` (etc for each [range type][std::ops])
/// - `(pattern1, pattern2, ...)`
///
/// For example, you could implement `hex_digit0` as:
/// ```
/// # use winnow::prelude::*;
/// # use winnow::{error::ErrMode, error::ErrorKind, error::Error};
/// # use winnow::bytes::take_while1;
/// fn hex_digit1(input: &str) -> IResult<&str, &str> {
///     take_while1(('a'..='f', 'A'..='F', '0'..='9')).parse_next(input)
/// }
///
/// assert_eq!(hex_digit1("21cZ"), Ok(("Z", "21c")));
/// assert_eq!(hex_digit1("H2"), Err(ErrMode::Backtrack(Error::new("H2", ErrorKind::TakeWhile1))));
/// assert_eq!(hex_digit1(""), Err(ErrMode::Backtrack(Error::new("", ErrorKind::TakeWhile1))));
/// ```
pub trait ContainsToken<T> {
    /// Returns true if self contains the token
    fn contains_token(&self, token: T) -> bool;
}

impl ContainsToken<u8> for u8 {
    fn contains_token(&self, token: u8) -> bool {
        *self == token
    }
}

impl<'a> ContainsToken<&'a u8> for u8 {
    fn contains_token(&self, token: &u8) -> bool {
        self.contains_token(*token)
    }
}

impl ContainsToken<char> for u8 {
    fn contains_token(&self, token: char) -> bool {
        self.as_char() == token
    }
}

impl<'a> ContainsToken<&'a char> for u8 {
    fn contains_token(&self, token: &char) -> bool {
        self.contains_token(*token)
    }
}

impl<C: AsChar> ContainsToken<C> for char {
    fn contains_token(&self, token: C) -> bool {
        *self == token.as_char()
    }
}

impl<C: AsChar, F: Fn(C) -> bool> ContainsToken<C> for F {
    fn contains_token(&self, token: C) -> bool {
        self(token)
    }
}

impl<C1: AsChar, C2: AsChar + Clone> ContainsToken<C1> for Range<C2> {
    fn contains_token(&self, token: C1) -> bool {
        let start = self.start.clone().as_char();
        let end = self.end.clone().as_char();
        (start..end).contains(&token.as_char())
    }
}

impl<C1: AsChar, C2: AsChar + Clone> ContainsToken<C1> for RangeInclusive<C2> {
    fn contains_token(&self, token: C1) -> bool {
        let start = self.start().clone().as_char();
        let end = self.end().clone().as_char();
        (start..=end).contains(&token.as_char())
    }
}

impl<C1: AsChar, C2: AsChar + Clone> ContainsToken<C1> for RangeFrom<C2> {
    fn contains_token(&self, token: C1) -> bool {
        let start = self.start.clone().as_char();
        (start..).contains(&token.as_char())
    }
}

impl<C1: AsChar, C2: AsChar + Clone> ContainsToken<C1> for RangeTo<C2> {
    fn contains_token(&self, token: C1) -> bool {
        let end = self.end.clone().as_char();
        (..end).contains(&token.as_char())
    }
}

impl<C1: AsChar, C2: AsChar + Clone> ContainsToken<C1> for RangeToInclusive<C2> {
    fn contains_token(&self, token: C1) -> bool {
        let end = self.end.clone().as_char();
        (..=end).contains(&token.as_char())
    }
}

impl<C1: AsChar> ContainsToken<C1> for RangeFull {
    fn contains_token(&self, _token: C1) -> bool {
        true
    }
}

impl<'a> ContainsToken<u8> for &'a [u8] {
    fn contains_token(&self, token: u8) -> bool {
        memchr::memchr(token, self).is_some()
    }
}

impl<'a, 'b> ContainsToken<&'a u8> for &'b [u8] {
    fn contains_token(&self, token: &u8) -> bool {
        self.contains_token(*token)
    }
}

impl<'a> ContainsToken<char> for &'a [u8] {
    fn contains_token(&self, token: char) -> bool {
        self.iter().any(|i| i.as_char() == token)
    }
}

impl<'a, 'b> ContainsToken<&'a char> for &'b [u8] {
    fn contains_token(&self, token: &char) -> bool {
        self.contains_token(*token)
    }
}

impl<const LEN: usize> ContainsToken<u8> for [u8; LEN] {
    fn contains_token(&self, token: u8) -> bool {
        memchr::memchr(token, &self[..]).is_some()
    }
}

impl<'a, const LEN: usize> ContainsToken<&'a u8> for [u8; LEN] {
    fn contains_token(&self, token: &u8) -> bool {
        self.contains_token(*token)
    }
}

impl<const LEN: usize> ContainsToken<char> for [u8; LEN] {
    fn contains_token(&self, token: char) -> bool {
        self.iter().any(|i| i.as_char() == token)
    }
}

impl<'a, const LEN: usize> ContainsToken<&'a char> for [u8; LEN] {
    fn contains_token(&self, token: &char) -> bool {
        self.contains_token(*token)
    }
}

impl<'a> ContainsToken<u8> for &'a str {
    fn contains_token(&self, token: u8) -> bool {
        self.as_bytes().contains_token(token)
    }
}

impl<'a, 'b> ContainsToken<&'a u8> for &'b str {
    fn contains_token(&self, token: &u8) -> bool {
        self.as_bytes().contains_token(token)
    }
}

impl<'a> ContainsToken<char> for &'a str {
    fn contains_token(&self, token: char) -> bool {
        self.chars().any(|i| i == token)
    }
}

impl<'a, 'b> ContainsToken<&'a char> for &'b str {
    fn contains_token(&self, token: &char) -> bool {
        self.contains_token(*token)
    }
}

impl<'a> ContainsToken<u8> for &'a [char] {
    fn contains_token(&self, token: u8) -> bool {
        self.iter().any(|i| *i == token.as_char())
    }
}

impl<'a, 'b> ContainsToken<&'a u8> for &'b [char] {
    fn contains_token(&self, token: &u8) -> bool {
        self.contains_token(*token)
    }
}

impl<'a> ContainsToken<char> for &'a [char] {
    fn contains_token(&self, token: char) -> bool {
        self.iter().any(|i| *i == token)
    }
}

impl<'a, 'b> ContainsToken<&'a char> for &'b [char] {
    fn contains_token(&self, token: &char) -> bool {
        self.contains_token(*token)
    }
}

impl<T> ContainsToken<T> for () {
    fn contains_token(&self, _token: T) -> bool {
        false
    }
}

macro_rules! impl_contains_token_for_tuple {
  ($($haystack:ident),+) => (
    #[allow(non_snake_case)]
    impl<T, $($haystack),+> ContainsToken<T> for ($($haystack),+,)
    where
    T: Clone,
      $($haystack: ContainsToken<T>),+
    {
      fn contains_token(&self, token: T) -> bool {
        let ($(ref $haystack),+,) = *self;
        $($haystack.contains_token(token.clone()) || )+ false
      }
    }
  )
}

macro_rules! impl_contains_token_for_tuples {
    ($haystack1:ident, $($haystack:ident),+) => {
        impl_contains_token_for_tuples!(__impl $haystack1; $($haystack),+);
    };
    (__impl $($haystack:ident),+; $haystack1:ident $(,$haystack2:ident)*) => {
        impl_contains_token_for_tuple!($($haystack),+);
        impl_contains_token_for_tuples!(__impl $($haystack),+, $haystack1; $($haystack2),*);
    };
    (__impl $($haystack:ident),+;) => {
        impl_contains_token_for_tuple!($($haystack),+);
    }
}

impl_contains_token_for_tuples!(
    F1, F2, F3, F4, F5, F6, F7, F8, F9, F10, F11, F12, F13, F14, F15, F16, F17, F18, F19, F20, F21
);

/// Looks for the first element of the input type for which the condition returns true,
/// and returns the input up to this position.
///
/// *streaming version*: If no element is found matching the condition, this will return `Incomplete`
pub(crate) fn split_at_offset_streaming<P, I: Input, E: ParseError<I>>(
    input: &I,
    predicate: P,
) -> IResult<I, <I as Input>::Slice, E>
where
    P: Fn(I::Token) -> bool,
{
    let offset = input
        .offset_for(predicate)
        .ok_or_else(|| ErrMode::Incomplete(Needed::new(1)))?;
    Ok(input.next_slice(offset))
}

/// Looks for the first element of the input type for which the condition returns true
/// and returns the input up to this position.
///
/// Fails if the produced slice is empty.
///
/// *streaming version*: If no element is found matching the condition, this will return `Incomplete`
pub(crate) fn split_at_offset1_streaming<P, I: Input, E: ParseError<I>>(
    input: &I,
    predicate: P,
    e: ErrorKind,
) -> IResult<I, <I as Input>::Slice, E>
where
    P: Fn(I::Token) -> bool,
{
    let offset = input
        .offset_for(predicate)
        .ok_or_else(|| ErrMode::Incomplete(Needed::new(1)))?;
    if offset == 0 {
        Err(ErrMode::from_error_kind(input.clone(), e))
    } else {
        Ok(input.next_slice(offset))
    }
}

/// Looks for the first element of the input type for which the condition returns true,
/// and returns the input up to this position.
///
/// *complete version*: If no element is found matching the condition, this will return the whole input
pub(crate) fn split_at_offset_complete<P, I: Input, E: ParseError<I>>(
    input: &I,
    predicate: P,
) -> IResult<I, <I as Input>::Slice, E>
where
    P: Fn(I::Token) -> bool,
{
    let offset = input
        .offset_for(predicate)
        .unwrap_or_else(|| input.input_len());
    Ok(input.next_slice(offset))
}

/// Looks for the first element of the input type for which the condition returns true
/// and returns the input up to this position.
///
/// Fails if the produced slice is empty.
///
/// *complete version*: If no element is found matching the condition, this will return the whole input
pub(crate) fn split_at_offset1_complete<P, I: Input, E: ParseError<I>>(
    input: &I,
    predicate: P,
    e: ErrorKind,
) -> IResult<I, <I as Input>::Slice, E>
where
    P: Fn(I::Token) -> bool,
{
    let offset = input
        .offset_for(predicate)
        .unwrap_or_else(|| input.input_len());
    if offset == 0 {
        Err(ErrMode::from_error_kind(input.clone(), e))
    } else {
        Ok(input.next_slice(offset))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_offset_u8() {
        let s = b"abcd123";
        let a = &s[..];
        let b = &a[2..];
        let c = &a[..4];
        let d = &a[3..5];
        assert_eq!(a.offset_to(b), 2);
        assert_eq!(a.offset_to(c), 0);
        assert_eq!(a.offset_to(d), 3);
    }

    #[test]
    fn test_offset_str() {
        let a = "abcřèÂßÇd123";
        let b = &a[7..];
        let c = &a[..5];
        let d = &a[5..9];
        assert_eq!(a.offset_to(b), 7);
        assert_eq!(a.offset_to(c), 0);
        assert_eq!(a.offset_to(d), 5);
    }
}
