//! ## Why use winnow
//!
//! If you want to write:
//!
//! ### Binary format parsers
//!
//! winnow was designed to properly parse binary formats from the beginning. Compared
//! to the usual handwritten C parsers, winnow parsers are just as fast, free from
//! buffer overflow vulnerabilities, and handle common patterns for you:
//!
//! - [TLV](https://en.wikipedia.org/wiki/Type-length-value)
//! - Bit level parsing
//! - Hexadecimal viewer in the debugging macros for easy data analysis
//! - Streaming parsers for network formats and huge files
//!
//! Example projects:
//!
//! - [FLV parser](https://github.com/rust-av/flavors)
//! - [Matroska parser](https://github.com/rust-av/matroska)
//! - [tar parser](https://github.com/Keruspe/tar-parser.rs)
//!
//! ### Text format parsers
//!
//! While winnow was made for binary format at first, it soon grew to work just as
//! well with text formats. From line based formats like CSV, to more complex, nested
//! formats such as JSON, winnow can manage it, and provides you with useful tools:
//!
//! - Fast case insensitive comparison
//! - Recognizers for escaped strings
//! - Regular expressions can be embedded in winnow parsers to represent complex character patterns succinctly
//! - Special care has been given to managing non ASCII characters properly
//!
//! Example projects:
//!
//! - [HTTP proxy](https://github.com/sozu-proxy/sozu/tree/main/lib/src/protocol/http/parser)
//! - [TOML parser](https://github.com/joelself/tomllib)
//!
//! ### Programming language parsers
//!
//! While programming language parsers are usually written manually for more
//! flexibility and performance, winnow can be (and has been successfully) used
//! as a prototyping parser for a language.
//!
//! winnow will get you started quickly with powerful custom error types, that you
//! can use to
//! pinpoint the exact line and column of the error. No need for separate
//! tokenizing, lexing and parsing phases: winnow can automatically handle whitespace
//! parsing, and construct an AST in place.
//!
//! ### Streaming formats
//!
//! While a lot of formats (and the code handling them) assume that they can fit
//! the complete data in memory, there are formats for which we only get a part
//! of the data at once, like network formats, or huge files.
//! winnow has been designed for a correct behaviour with partial data: If there is
//! not enough data to decide, winnow will tell you it needs more instead of silently
//! returning a wrong result. Whether your data comes entirely or in chunks, the
//! result should be the same.
//!
//! It allows you to build powerful, deterministic state machines for your protocols.
//!
//! Example projects:
//!
//! - [HTTP proxy](https://github.com/sozu-proxy/sozu/tree/main/lib/src/protocol/http/parser)
//! - [Using winnow with generators](https://github.com/Geal/generator_winnow)
//!
//! ## Parser combinators
//!
//! Parser combinators are an approach to parsers that is very different from
//! software like [lex](https://en.wikipedia.org/wiki/Lex_(software)) and
//! [yacc](https://en.wikipedia.org/wiki/Yacc). Instead of writing the grammar
//! in a separate file and generating the corresponding code, you use very
//! small functions with very specific purpose, like "take 5 bytes", or
//! "recognize the word 'HTTP'", and assemble them in meaningful patterns
//! like "recognize 'HTTP', then a space, then a version".
//! The resulting code is small, and looks like the grammar you would have
//! written with other parser approaches.
//!
//! This has a few advantages:
//!
//! - The parsers are small and easy to write
//! - The parsers components are easy to reuse (if they're general enough, please add them to winnow!)
//! - The parsers components are easy to test separately (unit tests and property-based tests)
//! - The parser combination code looks close to the grammar you would have written
//! - You can build partial parsers, specific to the data you need at the moment, and ignore the rest
//!
//! ## Technical features
//!
//! winnow parsers are for:
//! - [x] **byte-oriented**: The basic type is `&[u8]` and parsers will work as much as possible on byte array slices (but are not limited to them)
//! - [x] **bit-oriented**: winnow can address a byte slice as a bit stream
//! - [x] **string-oriented**: The same kind of combinators can apply on UTF-8 strings as well
//! - [x] **zero-copy**: If a parser returns a subset of its input data, it will return a slice of that input, without copying
//! - [x] **streaming**: winnow can work on partial data and detect when it needs more data to produce a correct result
//! - [x] **descriptive errors**: The parsers can aggregate a list of error codes with pointers to the incriminated input slice. Those error lists can be pattern matched to provide useful messages.
//! - [x] **custom error types**: You can provide a specific type to improve errors returned by parsers
//! - [x] **safe parsing**: winnow leverages Rust's safe memory handling and powerful types, and parsers are routinely fuzzed and tested with real world data. So far, the only flaws found by fuzzing were in code written outside of winnow
//! - [x] **speed**: Benchmarks have shown that winnow parsers often outperform many parser combinators library like Parsec and attoparsec, some regular expression engines and even handwritten C parsers
//!
//! Some benchmarks are available on [Github](https://github.com/rosetta-rs/parser-rosetta-rs).
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
//! - The parsers components are easy to reuse (if they're general enough, please add them to winnow!)
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
//! Here is another parser, written without using winnow's combinators this time:
//!
//! ```rust
//! use winnow::{IResult, error::ErrMode, error::Needed};
//!
//! # fn main() {
//! fn take4(i: &[u8]) -> IResult<&[u8], &[u8]>{
//!   if i.len() < 4 {
//!     Err(ErrMode::Incomplete(Needed::new(4)))
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
//! winnow provides a list of functions to help in developing parsers.
//!
//! With functions, you would write it like this:
//!
//! ```rust
//! use winnow::{IResult, bytes::take, stream::Partial};
//! fn take4(input: Partial<&str>) -> IResult<Partial<&str>, &str> {
//!   take(4u8)(input)
//! }
//! ```
//!
//! A parser in winnow is a function which, for an input type `I`, an output type `O`
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
//! use winnow::{error::Needed, error::Error};
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
//! - An error `Err(ErrMode::Backtrack(c))` with `c` an error that can be built from the input position and a parser specific error
//! - An error `Err(ErrMode::Incomplete(Needed))` indicating that more input is necessary. `Needed` can indicate how much data is needed
//! - An error `Err(ErrMode::Cut(c))`. It works like the `Backtrack` case, except it indicates an unrecoverable error: We cannot backtrack and test another parser
//!
//! Please refer to the ["choose a combinator" guide][crate::combinator] for an exhaustive list of parsers.
//!
//! ## Making new parsers with function combinators
//!
//! winnow is based on functions that generate parsers, with a signature like
//! this: `(arguments) -> impl Fn(Stream) -> IResult<Stream, Output, Error>`.
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
//! assert_eq!(alt_tags(&b"ijklxxx"[..]), Err(winnow::error::ErrMode::Backtrack(winnow::error::Error::new(&b"ijklxxx"[..], winnow::error::ErrorKind::Tag))));
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
//!     error::ErrorKind, error::Error, error::Needed,
//!     number::be_u16,
//!     bytes::{tag, take},
//!     stream::Partial,
//! };
//!
//! let mut tpl = (be_u16, take(3u8), tag("fg"));
//!
//! assert_eq!(
//!   tpl.parse_next(Partial::new(&b"abcdefgh"[..])),
//!   Ok((
//!     Partial::new(&b"h"[..]),
//!     (0x6162u16, &b"cde"[..], &b"fg"[..])
//!   ))
//! );
//! assert_eq!(tpl.parse_next(Partial::new(&b"abcde"[..])), Err(winnow::error::ErrMode::Incomplete(Needed::new(2))));
//! let input = &b"abcdejk"[..];
//! assert_eq!(tpl.parse_next(Partial::new(input)), Err(winnow::error::ErrMode::Backtrack(Error::new(Partial::new(&input[5..]), ErrorKind::Tag))));
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
//! # Making a new parser from scratch
//!
//! Writing a parser is a very fun, interactive process, but sometimes a daunting
//! task. How do you test it? How to see ambiguities in specifications?
//!
//! winnow is designed to abstract data manipulation (counting array offsets,
//! converting to structures, etc) while providing a safe, composable API. It also
//! takes care of making the code easy to test and read, but it can be confusing at
//! first, if you are not familiar with parser combinators, or if you are not used
//! to Rust generic functions.
//!
//! This document is here to help you in getting started with winnow. You can also find
//! [winnow recipes for common short parsing tasks here](crate::_cookbook). If you need
//! more specific help, please ping `geal` on IRC (libera, geeknode,
//! oftc), go to `#winnow-parsers` on Libera IRC, or on the
//! [Gitter chat room](https://gitter.im/Geal/winnow).
//!
//! # First step: the initial research
//!
//! A big part of the initial work lies in accumulating enough documentation and
//! samples to understand the format. The specification is useful, but specifications
//! represent an "official" point of view, that may not be the real world usage. Any
//! blog post or open source code is useful, because it shows how people understand
//! the format, and how they work around each other's bugs (if you think a
//! specification ensures every implementation is consistent with the others, think again).
//!
//! You should get a lot of samples (file or network traces) to test your code. The
//! easy way is to use a small number of samples coming from the same source and
//! develop everything around them, to realize later that they share a very specific
//! bug.
//!
//! # Code organization
//!
//! While it is tempting to insert the parsing code right inside the rest of the
//! logic, it usually results in  unmaintainable code, and makes testing challenging.
//! Parser combinators, the parsing technique used in winnow, assemble a lot of small
//! functions to make powerful parsers. This means that those functions only depend
//! on their input, not on an external state. This makes it easy to parse the input
//! partially, and to test those functions independently.
//!
//! Usually, you can separate the parsing functions in their own module, so you
//! could have a `src/lib.rs` file containing this:
//!
//! ```rust,ignore
//! pub mod parser;
//! ```
//!
//! And the `src/parser.rs` file:
//!
//! ```rust
//! use winnow::IResult;
//! use winnow::number::be_u16;
//! use winnow::bytes::take;
//!
//! pub fn length_value(input: &[u8]) -> IResult<&[u8],&[u8]> {
//!     let (input, length) = be_u16(input)?;
//!     take(length)(input)
//! }
//! ```
//!
//! # Writing a first parser
//!
//! Let's parse a simple expression like `(12345)`. winnow parsers are functions that
//! use the `winnow::IResult` type everywhere. As an example, a parser taking a byte
//! slice `&[u8]` and returning a 32 bits unsigned integer `u32` would have this
//! signature: `fn parse_u32(input: &[u8]) -> IResult<&[u8], u32>`.
//!
//! The `IResult` type depends on the input and output types, and an optional custom
//! error type. This enum can either be `Ok((i,o))` containing the remaining input
//! and the output value, or, on the `Err` side, an error or an indication that more
//! data is needed.
//!
//! ```rust
//! # use winnow::error::ErrorKind;
//! pub type IResult<I, O, E=(I,ErrorKind)> = Result<(I, O), Err<E>>;
//!
//! #[derive(Debug, PartialEq, Eq, Clone, Copy)]
//! pub enum Needed {
//!   Unknown,
//!   Size(u32)
//! }
//!
//! #[derive(Debug, Clone, PartialEq)]
//! pub enum Err<E> {
//!   Incomplete(Needed),
//!   Error(E),
//!   Failure(E),
//! }
//! ```
//!
//! winnow uses this type everywhere. Every combination of parsers will pattern match
//! on this to know if it must return a value, an error, consume more data, etc.
//! But this is done behind the scenes most of the time.
//!
//! Parsers are usually built from the bottom up, by first writing parsers for the
//! smallest elements, then assembling them in more complex parsers by using
//! combinators.
//!
//! As an example, here is how we could build a (non spec compliant) HTTP request
//! line parser:
//!
//! ```rust
//! # use winnow::prelude::*;
//! # use winnow::bytes::take_while1;
//! # use winnow::bytes::tag;
//! # use winnow::sequence::preceded;
//! # use winnow::stream::AsChar;
//! struct Request<'s> {
//!     method: &'s [u8],
//!     url: &'s [u8],
//!     version: &'s [u8],
//! }
//!
//! // combine all previous parsers in one function
//! fn request_line(i: &[u8]) -> IResult<&[u8], Request> {
//!   // first implement the basic parsers
//!   let method = take_while1(AsChar::is_alpha);
//!   let space = |i| take_while1(|c| c == b' ')(i);
//!   let url = take_while1(|c| c != b' ');
//!   let is_version = |c| c >= b'0' && c <= b'9' || c == b'.';
//!   let version = take_while1(is_version);
//!   let line_ending = tag("\r\n");
//!
//!   // combine http and version to extract the version string
//!   // preceded will return the result of the second parser
//!   // if both succeed
//!   let http_version = preceded("HTTP/", version);
//!
//!   // A tuple of parsers will evaluate each parser sequentally and return a tuple of the results
//!   let (input, (method, _, url, _, version, _)) =
//!     (method, space, url, space, http_version, line_ending).parse_next(i)?;
//!
//!   Ok((input, Request { method, url, version }))
//! }
//! ```
//!
//! Since it is easy to combine small parsers, I encourage you to write small
//! functions corresponding to specific parts of the format, test them
//! independently, then combine them in more general parsers.
//!
//! # Finding the right combinator
//!
//! winnow has a lot of different combinators, depending on the use case. They are all
//! described in the [reference][crate::combinator].
//!
//! Basic functions are available. They deal mostly
//! in recognizing character types, like `alphanumeric` or `digit`. They also parse
//! big endian and little endian integers and floats of multiple sizes.
//!
//! Most of the functions are there to combine parsers, and they are generic over
//! the input type.
//!
//! # Testing the parsers
//!
//! Once you have a parser function, a good trick is to test it on a lot of the
//! samples you gathered, and integrate this to your unit tests. To that end, put
//! all of the test files in a folder like `assets` and refer to test files like
//! this:
//!
//! ```rust
//! #[test]
//! fn header_test() {
//!   let data = include_bytes!("../assets/axolotl-piano.gif");
//!   println!("bytes:\n{}", &data[0..100].to_hex(8));
//!   let res = header(data);
//!   // ...
//! }
//! ```
//!
//! The `include_bytes!` macro (provided by Rust's standard library) will integrate
//! the file as a byte slice in your code. You can then just refer to the part of
//! the input the parser has to handle via its offset. Here, we take the first 100
//! bytes of a GIF file to parse its header
//! (complete code [here](https://github.com/Geal/gif.rs/blob/master/src/parser.rs#L305-L309)).
//!
//! If your parser handles textual data, you can just use a lot of strings directly
//! in the test, like this:
//!
//! ```rust
//! #[test]
//! fn factor_test() {
//!   assert_eq!(factor("3"),       Ok(("", 3)));
//!   assert_eq!(factor(" 12"),     Ok(("", 12)));
//!   assert_eq!(factor("537  "),   Ok(("", 537)));
//!   assert_eq!(factor("  24   "), Ok(("", 24)));
//! }
//! ```
//!
//! The more samples and test cases you get, the more you can experiment with your
//! parser design.
