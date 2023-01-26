//! # nom, eating data byte by byte
//!
//! **NOTE:** This is an unofficial, short-lived fork.  I'm nominating winnow to be nom v8.
//!
//! nom is a parser combinator library, supporting:
//! - String (`&str`), byte (`&[u8]`), and [custom input types][crate::input]
//! - [Streaming parsing][crate::input::Streaming]
//! - Zero copy parsing
//!
//! ## Example
//!
//! ```rust
//! use winnow::prelude::*;
//! use winnow::bytes::{tag, take_while_m_n};
//!
//! #[derive(Debug,PartialEq)]
//! pub struct Color {
//!   pub red:     u8,
//!   pub green:   u8,
//!   pub blue:    u8,
//! }
//!
//! fn from_hex(input: &str) -> Result<u8, std::num::ParseIntError> {
//!   u8::from_str_radix(input, 16)
//! }
//!
//! fn is_hex_digit(c: char) -> bool {
//!   c.is_digit(16)
//! }
//!
//! fn hex_primary(input: &str) -> IResult<&str, u8> {
//!   take_while_m_n(2, 2, is_hex_digit).map_res(from_hex).parse(input)
//! }
//!
//! fn hex_color(input: &str) -> IResult<&str, Color> {
//!   let (input, _) = tag("#")(input)?;
//!   let (input, (red, green, blue)) = (hex_primary, hex_primary, hex_primary).parse(input)?;
//!
//!   Ok((input, Color { red, green, blue }))
//! }
//!
//! fn main() {
//!   let result = hex_color("#2F14DF").finish();
//!   assert_eq!(result, Ok(Color {
//!     red: 47,
//!     green: 20,
//!     blue: 223,
//!   }));
//! }
//! ```
//!
//! The code is available on [Github](https://github.com/Geal/nom)
//!
//! There are a few [guides](https://github.com/Geal/nom/tree/main/doc) with more details
//! about [how to write parsers][_tutorial],
//! or the [error management system][error].
//! You can also check out the [_cookbook] module that contains examples of common patterns.
//!
//! **Looking for a specific combinator? Read the
//! ["choose a combinator" guide][combinator]**
//!
//! If you are upgrading to nom 5.0, please read the
//! [migration document](https://github.com/Geal/nom/blob/main/doc/upgrading_to_nom_5.md).
//!
//! ## Parser combinators
//!
//! Parser combinators are an approach to parsers that is very different from
//! software like [lex](https://en.wikipedia.org/wiki/Lex_(software)) and
//! [yacc](https://en.wikipedia.org/wiki/Yacc). Instead of writing the grammar
//! in a separate syntax and generating the corresponding code, you use very small
//! functions with very specific purposes, like "take 5 bytes", or "recognize the
//! word 'HTTP'", and assemble them in meaningful patterns like "recognize
//! 'HTTP', then a space, then a version".
//! The resulting code is small, and looks like the grammar you would have
//! written with other parser approaches.
//!
//! This gives us a few advantages:
//!
//! - The parsers are small and easy to write
//! - The parsers components are easy to reuse (if they're general enough, please add them to nom!)
//! - The parsers components are easy to test separately (unit tests and property-based tests)
//! - The parser combination code looks close to the grammar you would have written
//! - You can build partial parsers, specific to the data you need at the moment, and ignore the rest
//!
//! Here is an example of one such parser, to recognize text between parentheses:
//!
//! ```rust
//! use winnow::{
//!   IResult,
//!   sequence::delimited,
//!   bytes::take_till1
//! };
//!
//! fn parens(input: &str) -> IResult<&str, &str> {
//!   delimited('(', take_till1(")"), ')')(input)
//! }
//! ```
//!
//! It defines a function named `parens` which will recognize a sequence of the
//! character `(`, the longest byte array not containing `)`, then the character
//! `)`, and will return the byte array in the middle.
//!
//! Here is another parser, written without using nom's combinators this time:
//!
//! ```rust
//! use winnow::{IResult, Err, Needed};
//!
//! # fn main() {
//! fn take4(i: &[u8]) -> IResult<&[u8], &[u8]>{
//!   if i.len() < 4 {
//!     Err(Err::Incomplete(Needed::new(4)))
//!   } else {
//!     Ok((&i[4..], &i[0..4]))
//!   }
//! }
//! # }
//! ```
//!
//! This function takes a byte array as input, and tries to consume 4 bytes.
//! Writing all the parsers manually, like this, is dangerous, despite Rust's
//! safety features. There are still a lot of mistakes one can make. That's why
//! nom provides a list of functions to help in developing parsers.
//!
//! With functions, you would write it like this:
//!
//! ```rust
//! use winnow::{IResult, bytes::take, input::Streaming};
//! fn take4(input: Streaming<&str>) -> IResult<Streaming<&str>, &str> {
//!   take(4u8)(input)
//! }
//! ```
//!
//! A parser in nom is a function which, for an input type `I`, an output type `O`
//! and an optional error type `E`, will have the following signature:
//!
//! ```rust,compile_fail
//! fn parser(input: I) -> IResult<I, O, E>;
//! ```
//!
//! Or like this, if you don't want to specify a custom error type (it will be `(I, ErrorKind)` by default):
//!
//! ```rust,compile_fail
//! fn parser(input: I) -> IResult<I, O>;
//! ```
//!
//! `IResult` is an alias for the `Result` type:
//!
//! ```rust
//! use winnow::{Needed, error::Error};
//!
//! type IResult<I, O, E = Error<I>> = Result<(I, O), Err<E>>;
//!
//! enum Err<E> {
//!   Incomplete(Needed),
//!   Error(E),
//!   Failure(E),
//! }
//! ```
//!
//! It can have the following values:
//!
//! - A correct result `Ok((I,O))` with the first element being the remaining of the input (not parsed yet), and the second the output value;
//! - An error `Err(Err::Error(c))` with `c` an error that can be built from the input position and a parser specific error
//! - An error `Err(Err::Incomplete(Needed))` indicating that more input is necessary. `Needed` can indicate how much data is needed
//! - An error `Err(Err::Failure(c))`. It works like the `Error` case, except it indicates an unrecoverable error: We cannot backtrack and test another parser
//!
//! Please refer to the ["choose a combinator" guide][combinator] for an exhaustive list of parsers.
//! See also the rest of the documentation [here](https://github.com/Geal/nom/blob/main/doc).
//!
//! ## Making new parsers with function combinators
//!
//! nom is based on functions that generate parsers, with a signature like
//! this: `(arguments) -> impl Fn(Input) -> IResult<Input, Output, Error>`.
//! The arguments of a combinator can be direct values (like `take` which uses
//! a number of bytes or character as argument) or even other parsers (like
//! `delimited` which takes as argument 3 parsers, and returns the result of
//! the second one if all are successful).
//!
//! Here are some examples:
//!
//! ```rust
//! use winnow::IResult;
//! use winnow::bytes::{tag, take};
//! fn abcd_parser(i: &str) -> IResult<&str, &str> {
//!   tag("abcd")(i) // will consume bytes if the input begins with "abcd"
//! }
//!
//! fn take_10(i: &[u8]) -> IResult<&[u8], &[u8]> {
//!   take(10u8)(i) // will consume and return 10 bytes of input
//! }
//! ```
//!
//! ## Combining parsers
//!
//! There are higher level patterns, like the **`alt`** combinator, which
//! provides a choice between multiple parsers. If one branch fails, it tries
//! the next, and returns the result of the first parser that succeeds:
//!
//! ```rust
//! use winnow::IResult;
//! use winnow::branch::alt;
//! use winnow::bytes::tag;
//!
//! let mut alt_tags = alt((tag("abcd"), tag("efgh")));
//!
//! assert_eq!(alt_tags(&b"abcdxxx"[..]), Ok((&b"xxx"[..], &b"abcd"[..])));
//! assert_eq!(alt_tags(&b"efghxxx"[..]), Ok((&b"xxx"[..], &b"efgh"[..])));
//! assert_eq!(alt_tags(&b"ijklxxx"[..]), Err(winnow::Err::Error((&b"ijklxxx"[..], winnow::error::ErrorKind::Tag))));
//! ```
//!
//! The **`opt`** combinator makes a parser optional. If the child parser returns
//! an error, **`opt`** will still succeed and return None:
//!
//! ```rust
//! use winnow::{IResult, combinator::opt, bytes::tag};
//! fn abcd_opt(i: &[u8]) -> IResult<&[u8], Option<&[u8]>> {
//!   opt(tag("abcd"))(i)
//! }
//!
//! assert_eq!(abcd_opt(&b"abcdxxx"[..]), Ok((&b"xxx"[..], Some(&b"abcd"[..]))));
//! assert_eq!(abcd_opt(&b"efghxxx"[..]), Ok((&b"efghxxx"[..], None)));
//! ```
//!
//! **`many0`** applies a parser 0 or more times, and returns a vector of the aggregated results:
//!
//! ```rust
//! # #[cfg(feature = "alloc")]
//! # fn main() {
//! use winnow::{IResult, multi::many0, bytes::tag};
//! use std::str;
//!
//! fn multi(i: &str) -> IResult<&str, Vec<&str>> {
//!   many0(tag("abcd"))(i)
//! }
//!
//! let a = "abcdef";
//! let b = "abcdabcdef";
//! let c = "azerty";
//! assert_eq!(multi(a), Ok(("ef",     vec!["abcd"])));
//! assert_eq!(multi(b), Ok(("ef",     vec!["abcd", "abcd"])));
//! assert_eq!(multi(c), Ok(("azerty", Vec::new())));
//! # }
//! # #[cfg(not(feature = "alloc"))]
//! # fn main() {}
//! ```
//!
//! Here are some basic combinators available:
//!
//! - **`opt`**: Will make the parser optional (if it returns the `O` type, the new parser returns `Option<O>`)
//! - **`many0`**: Will apply the parser 0 or more times (if it returns the `O` type, the new parser returns `Vec<O>`)
//! - **`many1`**: Will apply the parser 1 or more times
//!
//! There are more complex (and more useful) parsers like tuples, which is
//! used to apply a series of parsers then assemble their results.
//!
//! Example with tuples:
//!
//! ```rust
//! # fn main() {
//! use winnow::prelude::*;
//! use winnow::{
//!     error::ErrorKind, Needed,
//!     number::be_u16,
//!     bytes::{tag, take},
//!     input::Streaming,
//! };
//!
//! let mut tpl = (be_u16, take(3u8), tag("fg"));
//!
//! assert_eq!(
//!   tpl.parse(Streaming(&b"abcdefgh"[..])),
//!   Ok((
//!     Streaming(&b"h"[..]),
//!     (0x6162u16, &b"cde"[..], &b"fg"[..])
//!   ))
//! );
//! assert_eq!(tpl.parse(Streaming(&b"abcde"[..])), Err(winnow::Err::Incomplete(Needed::new(2))));
//! let input = &b"abcdejk"[..];
//! assert_eq!(tpl.parse(Streaming(input)), Err(winnow::Err::Error((Streaming(&input[5..]), ErrorKind::Tag))));
//! # }
//! ```
//!
//! But you can also use a sequence of combinators written in imperative style,
//! thanks to the `?` operator:
//!
//! ```rust
//! # fn main() {
//! use winnow::{IResult, bytes::tag};
//!
//! #[derive(Debug, PartialEq)]
//! struct A {
//!   a: u8,
//!   b: u8
//! }
//!
//! fn ret_int1(i:&[u8]) -> IResult<&[u8], u8> { Ok((i,1)) }
//! fn ret_int2(i:&[u8]) -> IResult<&[u8], u8> { Ok((i,2)) }
//!
//! fn f(i: &[u8]) -> IResult<&[u8], A> {
//!   // if successful, the parser returns `Ok((remaining_input, output_value))` that we can destructure
//!   let (i, _) = tag("abcd")(i)?;
//!   let (i, a) = ret_int1(i)?;
//!   let (i, _) = tag("efgh")(i)?;
//!   let (i, b) = ret_int2(i)?;
//!
//!   Ok((i, A { a, b }))
//! }
//!
//! let r = f(b"abcdefghX");
//! assert_eq!(r, Ok((&b"X"[..], A{a: 1, b: 2})));
//! # }
//! ```
#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![cfg_attr(docsrs, feature(extended_key_value_attributes))]
#![cfg_attr(not(feature = "std"), no_std)]
#![deny(missing_docs)]
// BEGIN - Embark standard lints v6 for Rust 1.55+
// do not change or add/remove here, but one can add exceptions after this section
// for more info see: <https://github.com/EmbarkStudios/rust-ecosystem/issues/59>
// "-Dunsafe_code",
#![warn(clippy::all)]
#![warn(clippy::await_holding_lock)]
#![warn(clippy::char_lit_as_u8)]
#![warn(clippy::checked_conversions)]
#![warn(clippy::dbg_macro)]
#![warn(clippy::debug_assert_with_mut_call)]
#![warn(clippy::doc_markdown)]
#![warn(clippy::empty_enum)]
#![warn(clippy::enum_glob_use)]
#![warn(clippy::exit)]
#![warn(clippy::expl_impl_clone_on_copy)]
#![warn(clippy::explicit_deref_methods)]
#![warn(clippy::explicit_into_iter_loop)]
#![warn(clippy::fallible_impl_from)]
#![warn(clippy::filter_map_next)]
#![warn(clippy::flat_map_option)]
#![warn(clippy::float_cmp_const)]
#![warn(clippy::fn_params_excessive_bools)]
#![warn(clippy::from_iter_instead_of_collect)]
#![warn(clippy::if_let_mutex)]
#![warn(clippy::implicit_clone)]
#![warn(clippy::imprecise_flops)]
#![warn(clippy::inefficient_to_string)]
#![warn(clippy::invalid_upcast_comparisons)]
#![warn(clippy::large_digit_groups)]
#![warn(clippy::large_stack_arrays)]
#![warn(clippy::large_types_passed_by_value)]
#![warn(clippy::let_unit_value)]
#![warn(clippy::linkedlist)]
#![warn(clippy::lossy_float_literal)]
#![warn(clippy::macro_use_imports)]
#![warn(clippy::manual_ok_or)]
#![warn(clippy::map_err_ignore)]
#![warn(clippy::map_flatten)]
#![warn(clippy::map_unwrap_or)]
#![warn(clippy::match_on_vec_items)]
#![warn(clippy::match_same_arms)]
#![warn(clippy::match_wild_err_arm)]
#![warn(clippy::match_wildcard_for_single_variants)]
#![warn(clippy::mem_forget)]
#![warn(clippy::mismatched_target_os)]
#![warn(clippy::missing_enforced_import_renames)]
#![warn(clippy::mut_mut)]
#![warn(clippy::mutex_integer)]
#![warn(clippy::needless_borrow)]
#![warn(clippy::needless_continue)]
#![warn(clippy::needless_for_each)]
#![warn(clippy::option_option)]
#![warn(clippy::path_buf_push_overwrite)]
#![warn(clippy::ptr_as_ptr)]
#![warn(clippy::rc_mutex)]
#![warn(clippy::ref_option_ref)]
#![warn(clippy::rest_pat_in_fully_bound_structs)]
#![warn(clippy::same_functions_in_if_condition)]
#![warn(clippy::semicolon_if_nothing_returned)]
#![warn(clippy::single_match_else)]
#![warn(clippy::string_add_assign)]
#![warn(clippy::string_add)]
#![warn(clippy::string_lit_as_bytes)]
#![warn(clippy::string_to_string)]
#![warn(clippy::todo)]
#![warn(clippy::trait_duplication_in_bounds)]
#![warn(clippy::unimplemented)]
#![warn(clippy::unnested_or_patterns)]
#![warn(clippy::unused_self)]
#![warn(clippy::useless_transmute)]
#![warn(clippy::verbose_file_reads)]
#![warn(clippy::zero_sized_map_values)]
#![warn(future_incompatible)]
#![warn(nonstandard_style)]
#![warn(rust_2018_idioms)]
// END - Embark standard lints v6 for Rust 1.55+
#![allow(clippy::branches_sharing_code)]
#![allow(clippy::collapsible_else_if)]
#![allow(clippy::if_same_then_else)]
#![allow(clippy::bool_assert_comparison)]
#![allow(clippy::let_and_return)]
#![allow(clippy::assertions_on_constants)]
#![allow(clippy::map_unwrap_or)]
#![allow(clippy::single_match_else)]
#![allow(clippy::single_match)]
#[cfg_attr(nightly, warn(rustdoc::missing_doc_code_examples))]
#[cfg(feature = "alloc")]
#[macro_use]
extern crate alloc;
#[cfg(doctest)]
extern crate doc_comment;

#[cfg(doctest)]
doc_comment::doctest!("../README.md");

/// Lib module to re-export everything needed from `std` or `core`/`alloc`. This is how `serde` does
/// it, albeit there it is not public.
#[cfg_attr(nightly, allow(rustdoc::missing_doc_code_examples))]
pub mod lib {
  /// `std` facade allowing `std`/`core` to be interchangeable. Reexports `alloc` crate optionally,
  /// as well as `core` or `std`
  #[cfg(not(feature = "std"))]
  #[cfg_attr(nightly, allow(rustdoc::missing_doc_code_examples))]
  /// internal std exports for no_std compatibility
  pub mod std {
    #[doc(hidden)]
    #[cfg(not(feature = "alloc"))]
    pub use core::borrow;

    #[cfg(feature = "alloc")]
    #[doc(hidden)]
    pub use alloc::{borrow, boxed, string, vec};

    #[doc(hidden)]
    pub use core::{cmp, convert, fmt, iter, mem, ops, option, result, slice, str};

    /// internal reproduction of std prelude
    #[doc(hidden)]
    pub mod prelude {
      pub use core::prelude as v1;
    }
  }

  #[cfg(feature = "std")]
  #[cfg_attr(nightly, allow(rustdoc::missing_doc_code_examples))]
  /// internal std exports for `no_std` compatibility
  pub mod std {
    #[doc(hidden)]
    pub use std::{
      alloc, borrow, boxed, cmp, collections, convert, fmt, hash, iter, mem, ops, option, result,
      slice, str, string, vec,
    };

    /// internal reproduction of std prelude
    #[doc(hidden)]
    pub mod prelude {
      pub use std::prelude as v1;
    }
  }
}

pub use self::parser::*;

#[macro_use]
mod macros;
#[macro_use]
pub mod error;

pub mod branch;
pub mod combinator;
pub mod input;
pub mod multi;
mod parser;
pub mod sequence;

pub mod bits;
pub mod bytes;

pub mod character;

mod str;

pub mod number;

#[cfg(feature = "unstable-doc")]
pub mod _cookbook;
#[cfg(feature = "unstable-doc")]
pub mod _tutorial;

/// Core concepts available for glob import
///
/// Including
/// - [`FinishIResult`]
/// - [`Parser`]
///
/// ## Example
///
/// ```rust
/// use winnow::prelude::*;
///
/// fn parse_data(input: &str) -> IResult<&str, u64> {
///     // ...
/// #   winnow::character::u64(input)
/// }
///
/// fn main() {
///   let result = parse_data.parse("100").finish();
///   assert_eq!(result, Ok(100));
/// }
/// ```
pub mod prelude {
  pub use crate::input::InputIsStreaming as _;
  pub use crate::FinishIResult as _;
  pub use crate::IResult;
  pub use crate::IntoOutputIResult as _;
  pub use crate::Parser as _;
}
