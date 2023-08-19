//! # Chapter 3: Sequencing and Alternatives
//!
//! In the last chapter, we saw how to create simple parsers using prebuilt parsers.
//!
//! In this chapter, we explore two other widely used features:
//! alternatives and composition.
//!
//! ## Sequencing
//!
//! Now that we can create more interesting parsers, we can sequence them together, like:
//!
//! ```rust
//! # use winnow::prelude::*;
//! # use winnow::token::take_while;
//! #
//! fn parse_prefix<'s>(input: &mut &'s str) -> PResult<&'s str> {
//!     "0x".parse_next(input)
//! }
//!
//! fn parse_digits<'s>(input: &mut &'s str) -> PResult<&'s str> {
//!     take_while(1.., (
//!         ('0'..='9'),
//!         ('A'..='F'),
//!         ('a'..='f'),
//!     )).parse_next(input)
//! }
//!
//! fn main()  {
//!     let mut input = "0x1a2b Hello";
//!
//!     let prefix = parse_prefix.parse_next(&mut input).unwrap();
//!     let digits = parse_digits.parse_next(&mut input).unwrap();
//!
//!     assert_eq!(prefix, "0x");
//!     assert_eq!(digits, "1a2b");
//!     assert_eq!(input, " Hello");
//! }
//! ```
//!
//! To sequence these together, you can just put them in a tuple:
//! ```rust
//! # use winnow::prelude::*;
//! # use winnow::token::take_while;
//! #
//! # fn parse_prefix<'s>(input: &mut &'s str) -> PResult<&'s str> {
//! #     "0x".parse_next(input)
//! # }
//! #
//! # fn parse_digits<'s>(input: &mut &'s str) -> PResult<&'s str> {
//! #     take_while(1.., (
//! #         ('0'..='9'),
//! #         ('A'..='F'),
//! #         ('a'..='f'),
//! #     )).parse_next(input)
//! # }
//! #
//! //...
//!
//! fn main()  {
//!     let mut input = "0x1a2b Hello";
//!
//!     let (prefix, digits) = (
//!         parse_prefix,
//!         parse_digits
//!     ).parse_next(&mut input).unwrap();
//!
//!     assert_eq!(prefix, "0x");
//!     assert_eq!(digits, "1a2b");
//!     assert_eq!(input, " Hello");
//! }
//! ```
//!
//! Frequently, you won't care about the tag and you can instead use one of the provided combinators,
//! like [`preceded`]:
//! ```rust
//! # use winnow::prelude::*;
//! # use winnow::token::take_while;
//! use winnow::combinator::preceded;
//!
//! # fn parse_prefix<'s>(input: &mut &'s str) -> PResult<&'s str> {
//! #     "0x".parse_next(input)
//! # }
//! #
//! # fn parse_digits<'s>(input: &mut &'s str) -> PResult<&'s str> {
//! #     take_while(1.., (
//! #         ('0'..='9'),
//! #         ('A'..='F'),
//! #         ('a'..='f'),
//! #     )).parse_next(input)
//! # }
//! #
//! //...
//!
//! fn main() {
//!     let mut input = "0x1a2b Hello";
//!
//!     let digits = preceded(
//!         parse_prefix,
//!         parse_digits
//!     ).parse_next(&mut input).unwrap();
//!
//!     assert_eq!(digits, "1a2b");
//!     assert_eq!(input, " Hello");
//! }
//! ```
//!
//! See [`combinator`] for more sequencing parsers.
//!
//! ## Alternatives
//!
//! Sometimes, we might want to choose between two parsers; and we're happy with
//! either being used.
//!
//! [`Stream::checkpoint`] helps us to retry parsing:
//! ```rust
//! # use winnow::prelude::*;
//! # use winnow::token::take_while;
//! use winnow::stream::Stream;
//!
//! fn parse_digits<'s>(input: &mut &'s str) -> PResult<(&'s str, &'s str)> {
//!     let start = input.checkpoint();
//!
//!     if let Ok(output) = ("0b", parse_bin_digits).parse_next(input) {
//!         return Ok(output);
//!     }
//!
//!     input.reset(start);
//!     if let Ok(output) = ("0o", parse_oct_digits).parse_next(input) {
//!         return Ok(output);
//!     }
//!
//!     input.reset(start);
//!     if let Ok(output) = ("0d", parse_dec_digits).parse_next(input) {
//!         return Ok(output);
//!     }
//!
//!     input.reset(start);
//!     ("0x", parse_hex_digits).parse_next(input)
//! }
//!
//! // ...
//! # fn parse_bin_digits<'s>(input: &mut &'s str) -> PResult<&'s str> {
//! #     take_while(1.., (
//! #         ('0'..='7'),
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
//!
//! fn main() {
//!     let mut input = "0x1a2b Hello";
//!
//!     let (prefix, digits) = parse_digits.parse_next(&mut input).unwrap();
//!
//!     assert_eq!(input, " Hello");
//!     assert_eq!(prefix, "0x");
//!     assert_eq!(digits, "1a2b");
//!
//!     assert!(parse_digits(&mut "ghiWorld").is_err());
//! }
//! ```
//!
//! > **Warning:** the above example is for illustrative purposes and relying on `Result::Ok` or
//! > `Result::Err` can lead to incorrect behavior.  This will be clarified in later when covering
//! > [error handling][`chapter_6`#errmode]
//!
//! [`opt`] is a basic building block for correctly handling retrying parsing:
//! ```rust
//! # use winnow::prelude::*;
//! # use winnow::token::take_while;
//! use winnow::combinator::opt;
//!
//! fn parse_digits<'s>(input: &mut &'s str) -> PResult<(&'s str, &'s str)> {
//!     if let Some(output) = opt(("0b", parse_bin_digits)).parse_next(input)? {
//!         Ok(output)
//!     } else if let Some(output) = opt(("0o", parse_oct_digits)).parse_next(input)? {
//!         Ok(output)
//!     } else if let Some(output) = opt(("0d", parse_dec_digits)).parse_next(input)? {
//!         Ok(output)
//!     } else {
//!         ("0x", parse_hex_digits).parse_next(input)
//!     }
//! }
//! #
//! # fn parse_bin_digits<'s>(input: &mut &'s str) -> PResult<&'s str> {
//! #     take_while(1.., (
//! #         ('0'..='7'),
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
//! #
//! # fn main() {
//! #     let mut input = "0x1a2b Hello";
//! #
//! #     let (prefix, digits) = parse_digits.parse_next(&mut input).unwrap();
//! #
//! #     assert_eq!(input, " Hello");
//! #     assert_eq!(prefix, "0x");
//! #     assert_eq!(digits, "1a2b");
//! #
//! #     assert!(parse_digits(&mut "ghiWorld").is_err());
//! # }
//! ```
//!
//! [`alt`] encapsulates this if/else-if ladder pattern, with the last case being the `else`:
//! ```rust
//! # use winnow::prelude::*;
//! # use winnow::token::take_while;
//! use winnow::combinator::alt;
//!
//! fn parse_digits<'s>(input: &mut &'s str) -> PResult<(&'s str, &'s str)> {
//!     alt((
//!         ("0b", parse_bin_digits),
//!         ("0o", parse_oct_digits),
//!         ("0d", parse_dec_digits),
//!         ("0x", parse_hex_digits),
//!     )).parse_next(input)
//! }
//! #
//! # fn parse_bin_digits<'s>(input: &mut &'s str) -> PResult<&'s str> {
//! #     take_while(1.., (
//! #         ('0'..='7'),
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
//! #
//! # fn main() {
//! #     let mut input = "0x1a2b Hello";
//! #
//! #     let (prefix, digits) = parse_digits.parse_next(&mut input).unwrap();
//! #
//! #     assert_eq!(input, " Hello");
//! #     assert_eq!(prefix, "0x");
//! #     assert_eq!(digits, "1a2b");
//! #
//! #     assert!(parse_digits(&mut "ghiWorld").is_err());
//! # }
//! ```
//!
//! > **Note:** [`success`] and [`fail`] are parsers that might be useful in the `else` case.
//!
//! Sometimes a giant if/else-if ladder can be slow and you'd rather have a `match` statement for
//! branches of your parser that have unique prefixes.  In this case, you can use the
//! [`dispatch`][crate::combinator::dispatch] macro:
//!
//! ```rust
//! # use winnow::prelude::*;
//! # use winnow::token::take_while;
//! use winnow::combinator::dispatch;
//! use winnow::token::take;
//! use winnow::combinator::fail;
//!
//! fn parse_digits<'s>(input: &mut &'s str) -> PResult<&'s str> {
//!     dispatch!(take(2usize);
//!         "0b" => parse_bin_digits,
//!         "0o" => parse_oct_digits,
//!         "0d" => parse_dec_digits,
//!         "0x" => parse_hex_digits,
//!         _ => fail,
//!     ).parse_next(input)
//! }
//!
//! // ...
//! # fn parse_bin_digits<'s>(input: &mut &'s str) -> PResult<&'s str> {
//! #     take_while(1.., (
//! #         ('0'..='7'),
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
//!
//! fn main() {
//!     let mut input = "0x1a2b Hello";
//!
//!     let digits = parse_digits.parse_next(&mut input).unwrap();
//!
//!     assert_eq!(input, " Hello");
//!     assert_eq!(digits, "1a2b");
//!
//!     assert!(parse_digits(&mut "ghiWorld").is_err());
//! }
//! ```
//!
//! > **Note:** [`peek`] may be useful when [`dispatch`]ing from hints from each case's parser.
//!
//! See [`combinator`] for more alternative parsers.

#![allow(unused_imports)]
use super::chapter_6;
use crate::combinator;
use crate::combinator::alt;
use crate::combinator::dispatch;
use crate::combinator::fail;
use crate::combinator::opt;
use crate::combinator::peek;
use crate::combinator::preceded;
use crate::combinator::success;
use crate::stream::Stream;

super::tutorial_links![previous: chapter_2, next: chapter_4];
