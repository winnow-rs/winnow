//! # Chapter 7: Error Reporting
//!
//! ## Context
//!
//! With [`Parser::parse`] we get errors that point to the failure but don't explain the reason for
//! the failure:
//! ```rust
//! # use winnow::prelude::*;
//! # use winnow::token::take_while;
//! # use winnow::combinator::alt;
//! # use winnow::token::take;
//! # use winnow::combinator::fail;
//! # use winnow::Parser;
//! #
//! # #[derive(Debug, PartialEq, Eq)]
//! # pub struct Hex(usize);
//! #
//! # impl std::str::FromStr for Hex {
//! #     type Err = String;
//! #
//! #     fn from_str(input: &str) -> Result<Self, Self::Err> {
//! #         parse_digits
//! #             .try_map(|(t, v)| match t {
//! #                "0b" => usize::from_str_radix(v, 2),
//! #                "0o" => usize::from_str_radix(v, 8),
//! #                "0d" => usize::from_str_radix(v, 10),
//! #                "0x" => usize::from_str_radix(v, 16),
//! #                _ => unreachable!("`parse_digits` doesn't return `{t}`"),
//! #              })
//! #             .map(Hex)
//! #             .parse(input)
//! #             .map_err(|e| e.to_string())
//! #     }
//! # }
//! #
//! // ...
//!
//! # fn parse_digits<'s>(input: &mut &'s str) -> PResult<(&'s str, &'s str)> {
//! #     alt((
//! #         ("0b", parse_bin_digits),
//! #         ("0o", parse_oct_digits),
//! #         ("0d", parse_dec_digits),
//! #         ("0x", parse_hex_digits),
//! #     )).parse_next(input)
//! # }
//! #
//! # fn parse_bin_digits<'s>(input: &mut &'s str) -> PResult<&'s str> {
//! #     take_while(1.., (
//! #         ('0'..='1'),
//! #     )).parse_next(input)
//! # }
//! #
//! # fn parse_oct_digits<'s>(input: &mut &'s str) -> PResult<&'s str> {
//! #     take_while(1.., (
//! #         ('0'..='7'),
//! #     )).parse_next(input)
//! # }
//! #
//! # fn parse_dec_digits<'s>(input: &mut &'s str) -> PResult<&'s str> {
//! #     take_while(1.., (
//! #         ('0'..='9'),
//! #     )).parse_next(input)
//! # }
//! #
//! # fn parse_hex_digits<'s>(input: &mut &'s str) -> PResult<&'s str> {
//! #     take_while(1.., (
//! #         ('0'..='9'),
//! #         ('A'..='F'),
//! #         ('a'..='f'),
//! #     )).parse_next(input)
//! # }
//! fn main() {
//!     let input = "0xZZ";
//!     let error = "\
//! 0xZZ
//!   ^
//! ";
//!     assert_eq!(input.parse::<Hex>().unwrap_err(), error);
//! }
//! ```
//!
//! Back in [`chapter_1`], we glossed over the `Err` variant of [`PResult`].  `PResult<O>` is
//! actually short for `PResult<O, E=ContextError>` where [`ContextError`] is a relatively cheap
//! way of building up reasonable errors for humans.
//!
//! You can use [`Parser::context`] to annotate the error with custom types
//! while unwinding to further clarify the error:
//! ```rust
//! # use winnow::prelude::*;
//! # use winnow::token::take_while;
//! # use winnow::combinator::alt;
//! # use winnow::token::take;
//! # use winnow::combinator::fail;
//! # use winnow::Parser;
//! use winnow::error::StrContext;
//!
//! #
//! # #[derive(Debug, PartialEq, Eq)]
//! # pub struct Hex(usize);
//! #
//! # impl std::str::FromStr for Hex {
//! #     type Err = String;
//! #
//! #     fn from_str(input: &str) -> Result<Self, Self::Err> {
//! #         parse_digits
//! #             .try_map(|(t, v)| match t {
//! #                "0b" => usize::from_str_radix(v, 2),
//! #                "0o" => usize::from_str_radix(v, 8),
//! #                "0d" => usize::from_str_radix(v, 10),
//! #                "0x" => usize::from_str_radix(v, 16),
//! #                _ => unreachable!("`parse_digits` doesn't return `{t}`"),
//! #              })
//! #             .map(Hex)
//! #             .parse(input)
//! #             .map_err(|e| e.to_string())
//! #     }
//! # }
//! #
//! fn parse_digits<'s>(input: &mut &'s str) -> PResult<(&'s str, &'s str)> {
//!     alt((
//!         ("0b", parse_bin_digits).context(StrContext::Label("binary")),
//!         ("0o", parse_oct_digits).context(StrContext::Label("octal")),
//!         ("0d", parse_dec_digits).context(StrContext::Label("decimal")),
//!         ("0x", parse_hex_digits).context(StrContext::Label("hexadecimal")),
//!     )).parse_next(input)
//! }
//!
//! // ...
//!
//! #
//! # fn parse_bin_digits<'s>(input: &mut &'s str) -> PResult<&'s str> {
//! #     take_while(1.., (
//! #         ('0'..='1'),
//! #     )).parse_next(input)
//! # }
//! #
//! # fn parse_oct_digits<'s>(input: &mut &'s str) -> PResult<&'s str> {
//! #     take_while(1.., (
//! #         ('0'..='7'),
//! #     )).parse_next(input)
//! # }
//! #
//! # fn parse_dec_digits<'s>(input: &mut &'s str) -> PResult<&'s str> {
//! #     take_while(1.., (
//! #         ('0'..='9'),
//! #     )).parse_next(input)
//! # }
//! #
//! # fn parse_hex_digits<'s>(input: &mut &'s str) -> PResult<&'s str> {
//! #     take_while(1.., (
//! #         ('0'..='9'),
//! #         ('A'..='F'),
//! #         ('a'..='f'),
//! #     )).parse_next(input)
//! # }
//! fn main() {
//!     let input = "0xZZ";
//!     let error = "\
//! 0xZZ
//!   ^
//! invalid hexadecimal";
//!     assert_eq!(input.parse::<Hex>().unwrap_err(), error);
//! }
//! ```
//!
//! At first glance, this looks correct but what `context` will be reported when parsing `"0b5"`?
//! ```rust
//! # use winnow::prelude::*;
//! # use winnow::token::take_while;
//! # use winnow::combinator::alt;
//! # use winnow::token::take;
//! # use winnow::combinator::fail;
//! # use winnow::Parser;
//! # use winnow::error::StrContext;
//! #
//! #
//! # #[derive(Debug, PartialEq, Eq)]
//! # pub struct Hex(usize);
//! #
//! # impl std::str::FromStr for Hex {
//! #     type Err = String;
//! #
//! #     fn from_str(input: &str) -> Result<Self, Self::Err> {
//! #         parse_digits
//! #             .try_map(|(t, v)| match t {
//! #                "0b" => usize::from_str_radix(v, 2),
//! #                "0o" => usize::from_str_radix(v, 8),
//! #                "0d" => usize::from_str_radix(v, 10),
//! #                "0x" => usize::from_str_radix(v, 16),
//! #                _ => unreachable!("`parse_digits` doesn't return `{t}`"),
//! #              })
//! #             .map(Hex)
//! #             .parse(input)
//! #             .map_err(|e| e.to_string())
//! #     }
//! # }
//! #
//! # fn parse_digits<'s>(input: &mut &'s str) -> PResult<(&'s str, &'s str)> {
//! #     alt((
//! #         ("0b", parse_bin_digits).context(StrContext::Label("binary")),
//! #         ("0o", parse_oct_digits).context(StrContext::Label("octal")),
//! #         ("0d", parse_dec_digits).context(StrContext::Label("decimal")),
//! #         ("0x", parse_hex_digits).context(StrContext::Label("hexadecimal")),
//! #     )).parse_next(input)
//! # }
//! #
//! # fn parse_bin_digits<'s>(input: &mut &'s str) -> PResult<&'s str> {
//! #     take_while(1.., (
//! #         ('0'..='1'),
//! #     )).parse_next(input)
//! # }
//! #
//! # fn parse_oct_digits<'s>(input: &mut &'s str) -> PResult<&'s str> {
//! #     take_while(1.., (
//! #         ('0'..='7'),
//! #     )).parse_next(input)
//! # }
//! #
//! # fn parse_dec_digits<'s>(input: &mut &'s str) -> PResult<&'s str> {
//! #     take_while(1.., (
//! #         ('0'..='9'),
//! #     )).parse_next(input)
//! # }
//! #
//! # fn parse_hex_digits<'s>(input: &mut &'s str) -> PResult<&'s str> {
//! #     take_while(1.., (
//! #         ('0'..='9'),
//! #         ('A'..='F'),
//! #         ('a'..='f'),
//! #     )).parse_next(input)
//! # }
//! fn main() {
//!     let input = "0b5";
//!     let error = "\
//! 0b5
//! ^
//! invalid hexadecimal";
//!     assert_eq!(input.parse::<Hex>().unwrap_err(), error);
//! }
//! ```
//! If you remember back to [`chapter_3`], [`alt`] will only report the last error when what we
//! want is the error from `parse_bin_digits.
//!
//! ## Error Cuts
//!
//! Let's break down `PResult<O, E>` one step further:
//! ```rust
//! # use winnow::error::ErrorKind;
//! # use winnow::error::ErrMode;
//! pub type PResult<O, E = ErrorKind> = Result<O, ErrMode<E>>;
//! ```
//! [`PResult`] is just a fancy wrapper around `Result` that wraps our error in an [`ErrMode`]
//! type.
//!
//! [`ErrMode`] is an enum with [`Backtrack`] and [`Cut`] variants (ignore [`Incomplete`] as its only
//! relevant for [streaming][_topic::stream]). By default, errors are [`Backtrack`], meaning that
//! other parsing branches will be attempted on failure, like the next case of an [`alt`].  [`Cut`]
//! shortcircuits all other branches, immediately reporting the error.
//!
//! So we can get the correct `context` by modifying the above example with [`cut_err`]:
//! ```rust
//! # use winnow::prelude::*;
//! # use winnow::token::take_while;
//! # use winnow::combinator::alt;
//! # use winnow::combinator::cut_err;
//! # use winnow::token::take;
//! # use winnow::combinator::fail;
//! # use winnow::Parser;
//! use winnow::error::StrContext;
//!
//! #
//! # #[derive(Debug, PartialEq, Eq)]
//! # pub struct Hex(usize);
//! #
//! # impl std::str::FromStr for Hex {
//! #     type Err = String;
//! #
//! #     fn from_str(input: &str) -> Result<Self, Self::Err> {
//! #         parse_digits
//! #             .try_map(|(t, v)| match t {
//! #                "0b" => usize::from_str_radix(v, 2),
//! #                "0o" => usize::from_str_radix(v, 8),
//! #                "0d" => usize::from_str_radix(v, 10),
//! #                "0x" => usize::from_str_radix(v, 16),
//! #                _ => unreachable!("`parse_digits` doesn't return `{t}`"),
//! #              })
//! #             .map(Hex)
//! #             .parse(input)
//! #             .map_err(|e| e.to_string())
//! #     }
//! # }
//! #
//! fn parse_digits<'s>(input: &mut &'s str) -> PResult<(&'s str, &'s str)> {
//!     alt((
//!         ("0b", cut_err(parse_bin_digits)).context(StrContext::Label("binary")),
//!         ("0o", cut_err(parse_oct_digits)).context(StrContext::Label("octal")),
//!         ("0d", cut_err(parse_dec_digits)).context(StrContext::Label("decimal")),
//!         ("0x", cut_err(parse_hex_digits)).context(StrContext::Label("hexadecimal")),
//!     )).parse_next(input)
//! }
//!
//! // ...
//!
//! #
//! # fn parse_bin_digits<'s>(input: &mut &'s str) -> PResult<&'s str> {
//! #     take_while(1.., (
//! #         ('0'..='1'),
//! #     )).parse_next(input)
//! # }
//! #
//! # fn parse_oct_digits<'s>(input: &mut &'s str) -> PResult<&'s str> {
//! #     take_while(1.., (
//! #         ('0'..='7'),
//! #     )).parse_next(input)
//! # }
//! #
//! # fn parse_dec_digits<'s>(input: &mut &'s str) -> PResult<&'s str> {
//! #     take_while(1.., (
//! #         ('0'..='9'),
//! #     )).parse_next(input)
//! # }
//! #
//! # fn parse_hex_digits<'s>(input: &mut &'s str) -> PResult<&'s str> {
//! #     take_while(1.., (
//! #         ('0'..='9'),
//! #         ('A'..='F'),
//! #         ('a'..='f'),
//! #     )).parse_next(input)
//! # }
//! fn main() {
//!     let input = "0b5";
//!     let error = "\
//! 0b5
//!   ^
//! invalid binary";
//!     assert_eq!(input.parse::<Hex>().unwrap_err(), error);
//! }
//! ```

#![allow(unused_imports)]
use super::chapter_1;
use super::chapter_3;
use crate::combinator::alt;
use crate::combinator::cut_err;
use crate::error::ContextError;
use crate::error::ErrMode;
use crate::error::ErrMode::*;
use crate::error::ErrorKind;
use crate::PResult;
use crate::Parser;
use crate::_topic;

pub use super::chapter_6 as previous;
pub use super::chapter_8 as next;
pub use crate::_tutorial as table_of_contents;
