//! # Error management
//!
//! nom's errors are designed with multiple needs in mind:
//! - indicate which parser failed and where in the input data
//! - accumulate more context as the error goes up the parser chain
//! - have a very low overhead, as errors are often discarded by the calling parser (examples: `many0`, `alt`)
//! - can be modified according to the user's needs, because some languages need a lot more information
//!
//! To match these requirements, nom parsers have to return the following result
//! type:
//!
//! ```rust
//! pub type IResult<I, O, E=nom::error::Error<I>> = Result<(I, O), nom::Err<E>>;
//!
//! #[derive(Debug, PartialEq, Eq, Clone, Copy)]
//! pub enum Needed {
//!   Unknown,
//!   Size(u32)
//! }
//!
//! pub enum Err<E> {
//!     Incomplete(Needed),
//!     Error(E),
//!     Failure(E),
//! }
//! ```
//!
//! The result is either an `Ok((I, O))` containing the remaining input and the
//! parsed value, or an `Err(nom::Err<E>)` with `E` the error type.
//! `nom::Err<E>` is an enum because combinators can have different behaviours
//! depending on the value:
//! - `Error` is a normal parser error. If a child parser of the `alt` combinator returns `Error`, it will try another child parser
//! - `Failure` is an error from which we cannot recover: The `alt` combinator will not try other branches if a child parser returns `Failure`. This is used when we know we were in the right branch of `alt` and do not need to try other branches
//! - `Incomplete` indicates that a parser did not have enough data to decide. This can be returned by parsers found in `streaming` submodules. Parsers in the `complete` submodules assume that they have the entire input data, so if it was not sufficient, they will instead return a `Err::Error`. When a parser returns `Incomplete`, we should accumulate more data in the buffer (example: reading from a socket) and call the parser again
//!
//! If we are running a parser and know it will not return `Err::Incomplete`, we can
//! directly extract the error type from `Err::Error` or `Err::Failure` with the
//! `finish()` method:
//!
//! ```rust,ignore
//! # use nom::IResult;
//! # use nom::Finish;
//! # let parser = nom::bytes::complete::take_while1(|c| c == ' ');
//! # let input = " ";
//! let parser_result: IResult<_, _, _> = parser(input);
//! let result: Result<(_, _), _> = parser_result.finish();
//! ```
//!
//! If we used a borrowed type as input, like `&[u8]` or `&str`, we might want to
//! convert it to an owned type to transmit it somewhere, with the `to_owned()`
//! method:
//!
//! ```rust,ignore
//! # use nom::Err;
//! # type Value<'s> = &'s [u8];
//! # let parser = nom::bytes::complete::take_while1(|c| c == b' ');
//! # let data = " ";
//! let result: Result<(&[u8], Value<'_>), Err<Vec<u8>>> =
//!   parser(data).map_err(|e: E<&[u8]>| e.to_owned());
//! ```
//!
//! nom provides a powerful error system that can adapt to your needs: you can
//! get reduced error information if you want to improve performance, or you can
//! get a precise trace of parser application, with fine grained position information.
//!
//! This is done through the third type parameter of `IResult`, nom's parser result
//! type:
//!
//! ```rust
//! pub type IResult<I, O, E=nom::error::Error<I>> = Result<(I, O), Err<E>>;
//!
//! #[derive(Debug, PartialEq, Eq, Clone, Copy)]
//! pub enum Needed {
//!   Unknown,
//!   Size(u32)
//! }
//!
//! pub enum Err<E> {
//!     Incomplete(Needed),
//!     Error(E),
//!     Failure(E),
//! }
//! ```
//!
//! This error type is completely generic in nom's combinators, so you can choose
//! exactly which error type you want to use when you define your parsers, or
//! directly at the call site.
//! See [the JSON parser](https://github.com/Geal/nom/blob/5405e1173f1052f7e006dcb0b9cfda2b06557b65/examples/json.rs#L209-L286)
//! for an example of choosing different error types at the call site.
//!
//! The `Err<E>` enum expresses 3 conditions for a parser error:
//! - `Incomplete` indicates that a parser did not have enough data to decide. This can be returned by parsers found in `streaming` submodules to indicate that we should buffer more data from a file or socket. Parsers in the `complete` submodules assume that they have the entire input data, so if it was not sufficient, they will instead return a `Err::Error`
//! - `Error` is a normal parser error. If a child parser of the `alt` combinator returns `Error`, it will try another child parser
//! - `Failure` is an error from which we cannot recover: The `alt` combinator will not try other branches if a child parser returns `Failure`. If we know we were in the right branch (example: we found a correct prefix character but input after that was wrong), we can transform a `Err::Error` into a `Err::Failure` with the `cut()` combinator
//!
//! ## Common error types
//!
//! ### the default error type: nom::error::Error
//!
//! ```rust
//! # use nom::error::ErrorKind;
//! #[derive(Debug, PartialEq)]
//! pub struct Error<I> {
//!   /// position of the error in the input data
//!   pub input: I,
//!   /// nom error code
//!   pub code: ErrorKind,
//! }
//! ```
//!
//! This structure contains a `nom::error::ErrorKind` indicating which kind of
//! parser encountered an error (example: `ErrorKind::Tag` for the `tag()`
//! combinator), and the input position of the error.
//!
//! This error type is fast and has very low overhead, so it is suitable for
//! parsers that are called repeatedly, like in network protocols.
//! It is very limited though, it will not tell you about the chain of
//! parser calls, so it is not enough to write user friendly errors.
//!
//! Example error returned in a JSON-like parser (from `examples/json.rs`):
//!
//! ```rust,ignore
//! let data = "  { \"a\"\t: 42,
//! \"b\": [ \"x\", \"y\", 12 ] ,
//! \"c\": { 1\"hello\" : \"world\"
//! }
//! } ";
//!
//! // will print:
//! // Err(
//! //   Failure(
//! //       Error {
//! //           input: "1\"hello\" : \"world\"\n  }\n  } ",
//! //           code: Char,
//! //       },
//! //   ),
//! // )
//! println!(
//!   "{:#?}\n",
//!   json::<Error<&str>>(data)
//! );
//! ```
//!
//! ### getting more information: nom::error::VerboseError
//!
//! The  `VerboseError<I>` type accumulates more information about the chain of
//! parsers that encountered an error:
//!
//! ```rust
//! # use nom::error::ErrorKind;
//! #[derive(Clone, Debug, PartialEq)]
//! pub struct VerboseError<I> {
//!   /// List of errors accumulated by `VerboseError`, containing the affected
//!   /// part of input data, and some context
//!   pub errors: Vec<(I, VerboseErrorKind)>,
//! }
//!
//! #[derive(Clone, Debug, PartialEq)]
//! /// Error context for `VerboseError`
//! pub enum VerboseErrorKind {
//!   /// Static string added by the `context` function
//!   Context(&'static str),
//!   /// Indicates which character was expected by the `char` function
//!   Char(char),
//!   /// Error kind given by various nom parsers
//!   Nom(ErrorKind),
//! }
//! ```
//!
//! It contains the input position and error code for each of those parsers.
//! It does not accumulate errors from the different branches of `alt`, it will
//! only contain errors from the last branch it tried.
//!
//! It can be used along with the `nom::error::context` combinator to inform about
//! the parser chain:
//!
//! ```rust,ignore
//! # use nom::error::context;
//! # use nom::sequence::preceded;
//! # use nom::character::complete::char;
//! # use nom::combinator::cut;
//! # use nom::sequence::terminated;
//! # let parse_str = nom::bytes::complete::take_while1(|c| c == ' ');
//! # let i = " ";
//! context(
//!   "string",
//!   preceded(char('\"'), cut(terminated(parse_str, char('\"')))),
//! )(i);
//! ```
//!
//! It is not very usable if printed directly:
//!
//! ```rust,ignore
//! // parsed verbose: Err(
//! //   Failure(
//! //       VerboseError {
//! //           errors: [
//! //               (
//! //                   "1\"hello\" : \"world\"\n  }\n  } ",
//! //                   Char(
//! //                       '}',
//! //                   ),
//! //               ),
//! //               (
//! //                   "{ 1\"hello\" : \"world\"\n  }\n  } ",
//! //                   Context(
//! //                       "map",
//! //                   ),
//! //               ),
//! //               (
//! //                   "{ \"a\"\t: 42,\n  \"b\": [ \"x\", \"y\", 12 ] ,\n  \"c\": { 1\"hello\" : \"world\"\n  }\n  } ",
//! //                   Context(
//! //                       "map",
//! //                   ),
//! //               ),
//! //           ],
//! //       },
//! //   ),
//! // )
//! println!("parsed verbose: {:#?}", json::<VerboseError<&str>>(data));
//! ```
//!
//! But by looking at the original input and the chain of errors, we can build
//! a more user friendly error message. The `nom::error::convert_error` function
//! can build such a message.
//!
//! ```rust,ignore
//! let e = json::<VerboseError<&str>>(data).finish().err().unwrap();
//! // here we use the `convert_error` function, to transform a `VerboseError<&str>`
//! // into a printable trace.
//! //
//! // This will print:
//! // verbose errors - `json::<VerboseError<&str>>(data)`:
//! // 0: at line 2:
//! //   "c": { 1"hello" : "world"
//! //          ^
//! // expected '}', found 1
//! //
//! // 1: at line 2, in map:
//! //   "c": { 1"hello" : "world"
//! //        ^
//! //
//! // 2: at line 0, in map:
//! //   { "a" : 42,
//! //   ^
//! println!(
//!   "verbose errors - `json::<VerboseError<&str>>(data)`:\n{}",
//!   convert_error(data, e)
//! );
//! ```
//!
//! Note that `VerboseError` and `convert_error` are meant as a starting point for
//! language errors, but that they cannot cover all use cases. So a custom
//! `convert_error` function should probably be written.
//!
//! ### Improving usability: nom_locate and nom-supreme
//!
//! These crates were developed to improve the user experience when writing nom
//! parsers.
//!
//! #### nom_locate
//!
//! [nom_locate](https://docs.rs/nom_locate/) wraps the input data in a `Span`
//! type that can be understood by nom parsers. That type provides location
//! information, like line and column.
//!
//! #### nom-supreme
//!
//! [nom-supreme](https://docs.rs/nom-supreme/) provides the `ErrorTree<I>` error
//! type, that provides the same chain of parser errors as `VerboseError`, but also
//! accumulates errors from the various branches tried by `alt`.
//!
//! With this error type, you can explore everything that has been tried by the
//! parser.
//!
//! ## The `ParseError` trait
//!
//! If those error types are not enough, we can define our own, by implementing
//! the `ParseError<I>` trait. All nom combinators are generic over that trait
//! for their errors, so we only need to define it in the parser result type,
//! and it will be used everywhere.
//!
//! ```rust
//! # use nom::error::ErrorKind;
//! pub trait ParseError<I>: Sized {
//!     /// Creates an error from the input position and an [ErrorKind]
//!     fn from_error_kind(input: I, kind: ErrorKind) -> Self;
//!
//!     /// Combines an existing error with a new one created from the input
//!     /// position and an [ErrorKind]. This is useful when backtracking
//!     /// through a parse tree, accumulating error context on the way
//!     fn append(input: I, kind: ErrorKind, other: Self) -> Self;
//!
//!     /// Creates an error from an input position and an expected character
//!     fn from_char(input: I, _: char) -> Self {
//!         Self::from_error_kind(input, ErrorKind::Char)
//!     }
//!
//!     /// Combines two existing errors. This function is used to compare errors
//!     /// generated in various branches of `alt`
//!     fn or(self, other: Self) -> Self {
//!         other
//!     }
//! }
//! ```
//!
//! Any error type has to implement that trait, that requires ways to build an
//! error:
//! - `from_error_kind`: From the input position and the `ErrorKind` enum that indicates in which parser we got an error
//! - `append`: Allows the creation of a chain of errors as we backtrack through the parser tree (various combinators will add more context)
//! - `from_char`: Creates an error that indicates which character we were expecting
//! - `or`: In combinators like `alt`, allows choosing between errors from various branches (or accumulating them)
//!
//! We can also implement the `ContextError` trait to support the `context()`
//! combinator used by `VerboseError<I>`:
//!
//! ```rust
//! pub trait ContextError<I>: Sized {
//!     fn add_context(_input: I, _ctx: &'static str, other: Self) -> Self {
//!         other
//!     }
//! }
//! ```
//!
//! And there is also the `FromExternalError<I, E>` used by `map_res` to wrap
//! errors returned by other functions:
//!
//! ```rust
//! # use nom::error::ErrorKind;
//! pub trait FromExternalError<I, ExternalError> {
//!   fn from_external_error(input: I, kind: ErrorKind, e: ExternalError) -> Self;
//! }
//! ```
//!
//! ### Example usage
//!
//! Let's define a debugging error type, that will print something every time an
//! error is generated. This will give us a good insight into what the parser tried.
//! Since errors can be combined with each other, we want it to keep some info on
//! the error that was just returned. We'll just store that in a string:
//!
//! ```rust
//! struct DebugError {
//!     message: String,
//! }
//! ```
//!
//! Now let's implement `ParseError` and `ContextError` on it:
//!
//! ```rust
//! # use nom::error::ParseError;
//! # use nom::error::ErrorKind;
//! # use nom::error::ContextError;
//! # struct DebugError {
//! #     message: String,
//! # }
//! impl ParseError<&str> for DebugError {
//!     // on one line, we show the error code and the input that caused it
//!     fn from_error_kind(input: &str, kind: ErrorKind) -> Self {
//!         let message = format!("{:?}:\t{:?}\n", kind, input);
//!         println!("{}", message);
//!         DebugError { message }
//!     }
//!
//!     // if combining multiple errors, we show them one after the other
//!     fn append(input: &str, kind: ErrorKind, other: Self) -> Self {
//!         let message = format!("{}{:?}:\t{:?}\n", other.message, kind, input);
//!         println!("{}", message);
//!         DebugError { message }
//!     }
//!
//!     fn from_char(input: &str, c: char) -> Self {
//!         let message = format!("'{}':\t{:?}\n", c, input);
//!         println!("{}", message);
//!         DebugError { message }
//!     }
//!
//!     fn or(self, other: Self) -> Self {
//!         let message = format!("{}\tOR\n{}\n", self.message, other.message);
//!         println!("{}", message);
//!         DebugError { message }
//!     }
//! }
//!
//! impl ContextError<&str> for DebugError {
//!     fn add_context(input: &str, ctx: &'static str, other: Self) -> Self {
//!         let message = format!("{}\"{}\":\t{:?}\n", other.message, ctx, input);
//!         println!("{}", message);
//!         DebugError { message }
//!     }
//! }
//! ```
//!
//! So when calling our JSON parser with this error type, we will get a trace
//! of all the times a parser stoppped and backtracked:
//!
//! ```rust,ignore
//! println!("debug: {:#?}", root::<DebugError>(data));
//! ```
//!
//! ```text
//! AlphaNumeric:   "\"\t: 42,\n  \"b\": [ \"x\", \"y\", 12 ] ,\n  \"c\": { 1\"hello\" : \"world\"\n  }\n  } "
//!
//! '{':    "42,\n  \"b\": [ \"x\", \"y\", 12 ] ,\n  \"c\": { 1\"hello\" : \"world\"\n  }\n  } "
//!
//! '{':    "42,\n  \"b\": [ \"x\", \"y\", 12 ] ,\n  \"c\": { 1\"hello\" : \"world\"\n  }\n  } "
//! "map":  "42,\n  \"b\": [ \"x\", \"y\", 12 ] ,\n  \"c\": { 1\"hello\" : \"world\"\n  }\n  } "
//!
//! [..]
//!
//! AlphaNumeric:   "\": { 1\"hello\" : \"world\"\n  }\n  } "
//!
//! '"':    "1\"hello\" : \"world\"\n  }\n  } "
//!
//! '"':    "1\"hello\" : \"world\"\n  }\n  } "
//! "string":       "1\"hello\" : \"world\"\n  }\n  } "
//!
//! '}':    "1\"hello\" : \"world\"\n  }\n  } "
//!
//! '}':    "1\"hello\" : \"world\"\n  }\n  } "
//! "map":  "{ 1\"hello\" : \"world\"\n  }\n  } "
//!
//! '}':    "1\"hello\" : \"world\"\n  }\n  } "
//! "map":  "{ 1\"hello\" : \"world\"\n  }\n  } "
//! "map":  "{ \"a\"\t: 42,\n  \"b\": [ \"x\", \"y\", 12 ] ,\n  \"c\": { 1\"hello\" : \"world\"\n  }\n  } "
//!
//! debug: Err(
//!     Failure(
//!         DebugError {
//!             message: "'}':\t\"1\\\"hello\\\" : \\\"world\\\"\\n  }\\n  } \"\n\"map\":\t\"{ 1\\\"hello\\\" : \\\"world
//! \\"\\n  }\\n  } \"\n\"map\":\t\"{ \\\"a\\\"\\t: 42,\\n  \\\"b\\\": [ \\\"x\\\", \\\"y\\\", 12 ] ,\\n  \\\"c\\\": { 1\
//! \"hello\\\" : \\\"world\\\"\\n  }\\n  } \"\n",
//!         },
//!     ),
//! )
//! ```
//!
//! Here we can see that when parsing `{ 1\"hello\" : \"world\"\n  }\n  }`, after
//! getting past the initial `{`, we tried:
//! - parsing a `"` because we're expecting a key name, and that parser was part of the
//! "string" parser
//! - parsing a `}` because the map might be empty. When this fails, we backtrack,
//! through 2 recursive map parsers:
//!
//! ```text
//! '}':    "1\"hello\" : \"world\"\n  }\n  } "
//! "map":  "{ 1\"hello\" : \"world\"\n  }\n  } "
//! "map":  "{ \"a\"\t: 42,\n  \"b\": [ \"x\", \"y\", 12 ] ,\n  \"c\": { 1\"hello\" : \"world\"\n  }\n  } "
//! ```
//!
//! ## Debugging parsers
//!
//! While you are writing your parsers, you will sometimes need to follow
//! which part of the parser sees which part of the input.
//!
//! To that end, nom provides the `dbg_dmp` function that will observe
//! a parser's input and output, and print a hexdump of the input if there was an
//! error. Here is what it could return:
//!
#![cfg_attr(feature = "std", doc = "```")]
#![cfg_attr(not(feature = "std"), doc = "```ignore")]
//! # use nom::IResult;
//! # use nom::error::dbg_dmp;
//! # use nom::bytes::complete::tag;
//! fn f(i: &[u8]) -> IResult<&[u8], &[u8]> {
//!     dbg_dmp(tag("abcd"), "tag").parse(i)
//! }
//!
//! let a = &b"efghijkl"[..];
//!
//! // Will print the following message:
//! // tag: Error(Error(Error { input: [101, 102, 103, 104, 105, 106, 107, 108], code: Tag })) at:
//! // 00000000        65 66 67 68 69 6a 6b 6c         efghijkl
//! f(a);
//! ```
//!
//! You can go further with the [nom-trace crate](https://github.com/rust-bakery/nom-trace)

use crate::internal::Parser;
use crate::lib::std::fmt;

/// This trait must be implemented by the error type of a nom parser.
///
/// There are already implementations of it for `(Input, ErrorKind)`
/// and `VerboseError<Input>`.
///
/// It provides methods to create an error from some combinators,
/// and combine existing errors in combinators like `alt`.
pub trait ParseError<I>: Sized {
  /// Creates an error from the input position and an [ErrorKind]
  fn from_error_kind(input: I, kind: ErrorKind) -> Self;

  /// Combines an existing error with a new one created from the input
  /// position and an [ErrorKind]. This is useful when backtracking
  /// through a parse tree, accumulating error context on the way
  fn append(input: I, kind: ErrorKind, other: Self) -> Self;

  /// Creates an error from an input position and an expected character
  fn from_char(input: I, _: char) -> Self {
    Self::from_error_kind(input, ErrorKind::Char)
  }

  /// Combines two existing errors. This function is used to compare errors
  /// generated in various branches of `alt`.
  fn or(self, other: Self) -> Self {
    other
  }
}

/// This trait is required by the `context` combinator to add a static string
/// to an existing error
pub trait ContextError<I>: Sized {
  /// Creates a new error from an input position, a static string and an existing error.
  /// This is used mainly in the [context] combinator, to add user friendly information
  /// to errors when backtracking through a parse tree
  fn add_context(_input: I, _ctx: &'static str, other: Self) -> Self {
    other
  }
}

/// This trait is required by the `map_res` combinator to integrate
/// error types from external functions, like [std::str::FromStr]
pub trait FromExternalError<I, E> {
  /// Creates a new error from an input position, an [ErrorKind] indicating the
  /// wrapping parser, and an external error
  fn from_external_error(input: I, kind: ErrorKind, e: E) -> Self;
}

/// default error type, only contains the error' location and code
#[derive(Debug, PartialEq)]
pub struct Error<I> {
  /// position of the error in the input data
  pub input: I,
  /// nom error code
  pub code: ErrorKind,
}

impl<I> Error<I> {
  /// creates a new basic error
  pub fn new(input: I, code: ErrorKind) -> Error<I> {
    Error { input, code }
  }
}

impl<I> ParseError<I> for Error<I> {
  fn from_error_kind(input: I, kind: ErrorKind) -> Self {
    Error { input, code: kind }
  }

  fn append(_: I, _: ErrorKind, other: Self) -> Self {
    other
  }
}

impl<I> ContextError<I> for Error<I> {}

impl<I, E> FromExternalError<I, E> for Error<I> {
  /// Create a new error from an input position and an external error
  fn from_external_error(input: I, kind: ErrorKind, _e: E) -> Self {
    Error { input, code: kind }
  }
}

/// The Display implementation allows the std::error::Error implementation
impl<I: fmt::Display> fmt::Display for Error<I> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "error {:?} at: {}", self.code, self.input)
  }
}

#[cfg(feature = "std")]
impl<I: fmt::Debug + fmt::Display> std::error::Error for Error<I> {}

// for backward compatibility, keep those trait implementations
// for the previously used error type
impl<I> ParseError<I> for (I, ErrorKind) {
  fn from_error_kind(input: I, kind: ErrorKind) -> Self {
    (input, kind)
  }

  fn append(_: I, _: ErrorKind, other: Self) -> Self {
    other
  }
}

impl<I> ContextError<I> for (I, ErrorKind) {}

impl<I, E> FromExternalError<I, E> for (I, ErrorKind) {
  fn from_external_error(input: I, kind: ErrorKind, _e: E) -> Self {
    (input, kind)
  }
}

impl<I> ParseError<I> for () {
  fn from_error_kind(_: I, _: ErrorKind) -> Self {}

  fn append(_: I, _: ErrorKind, _: Self) -> Self {}
}

impl<I> ContextError<I> for () {}

impl<I, E> FromExternalError<I, E> for () {
  fn from_external_error(_input: I, _kind: ErrorKind, _e: E) -> Self {}
}

/// Creates an error from the input position and an [ErrorKind]
pub fn make_error<I, E: ParseError<I>>(input: I, kind: ErrorKind) -> E {
  E::from_error_kind(input, kind)
}

/// Combines an existing error with a new one created from the input
/// position and an [ErrorKind]. This is useful when backtracking
/// through a parse tree, accumulating error context on the way
pub fn append_error<I, E: ParseError<I>>(input: I, kind: ErrorKind, other: E) -> E {
  E::append(input, kind, other)
}

/// This error type accumulates errors and their position when backtracking
/// through a parse tree. With some post processing (cf `examples/json.rs`),
/// it can be used to display user friendly error messages
#[cfg(feature = "alloc")]
#[derive(Clone, Debug, PartialEq)]
pub struct VerboseError<I> {
  /// List of errors accumulated by `VerboseError`, containing the affected
  /// part of input data, and some context
  pub errors: crate::lib::std::vec::Vec<(I, VerboseErrorKind)>,
}

#[cfg(feature = "alloc")]
#[derive(Clone, Debug, PartialEq)]
/// Error context for `VerboseError`
pub enum VerboseErrorKind {
  /// Static string added by the `context` function
  Context(&'static str),
  /// Indicates which character was expected by the `char` function
  Char(char),
  /// Error kind given by various nom parsers
  Nom(ErrorKind),
}

#[cfg(feature = "alloc")]
impl<I> ParseError<I> for VerboseError<I> {
  fn from_error_kind(input: I, kind: ErrorKind) -> Self {
    VerboseError {
      errors: vec![(input, VerboseErrorKind::Nom(kind))],
    }
  }

  fn append(input: I, kind: ErrorKind, mut other: Self) -> Self {
    other.errors.push((input, VerboseErrorKind::Nom(kind)));
    other
  }

  fn from_char(input: I, c: char) -> Self {
    VerboseError {
      errors: vec![(input, VerboseErrorKind::Char(c))],
    }
  }
}

#[cfg(feature = "alloc")]
impl<I> ContextError<I> for VerboseError<I> {
  fn add_context(input: I, ctx: &'static str, mut other: Self) -> Self {
    other.errors.push((input, VerboseErrorKind::Context(ctx)));
    other
  }
}

#[cfg(feature = "alloc")]
impl<I, E> FromExternalError<I, E> for VerboseError<I> {
  /// Create a new error from an input position and an external error
  fn from_external_error(input: I, kind: ErrorKind, _e: E) -> Self {
    Self::from_error_kind(input, kind)
  }
}

#[cfg(feature = "alloc")]
impl<I: fmt::Display> fmt::Display for VerboseError<I> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    writeln!(f, "Parse error:")?;
    for (input, error) in &self.errors {
      match error {
        VerboseErrorKind::Nom(e) => writeln!(f, "{:?} at: {}", e, input)?,
        VerboseErrorKind::Char(c) => writeln!(f, "expected '{}' at: {}", c, input)?,
        VerboseErrorKind::Context(s) => writeln!(f, "in section '{}', at: {}", s, input)?,
      }
    }

    Ok(())
  }
}

#[cfg(feature = "std")]
impl<I: fmt::Debug + fmt::Display> std::error::Error for VerboseError<I> {}

use crate::internal::{Err, IResult};

/// Create a new error from an input position, a static string and an existing error.
/// This is used mainly in the [context] combinator, to add user friendly information
/// to errors when backtracking through a parse tree
pub fn context<P, I, O, E>(context: &'static str, parser: P) -> Context<P, I, O, E> {
  Context {
    context,
    parser,
    i: Default::default(),
    o: Default::default(),
    e: Default::default(),
  }
}

/// Implementation of [`context`]
pub struct Context<P, I, O, E> {
  context: &'static str,
  parser: P,
  i: core::marker::PhantomData<I>,
  o: core::marker::PhantomData<O>,
  e: core::marker::PhantomData<E>,
}

impl<P, I, O, E> Context<P, I, O, E>
where
  P: Parser<I, O, E>,
  I: Clone,
  E: ContextError<I>,
{
  /// See [`Parser::parse`]
  pub fn parse(&mut self, input: I) -> IResult<I, O, E> {
    match self.parser.parse(input.clone()) {
      Ok(o) => Ok(o),
      Err(Err::Incomplete(i)) => Err(Err::Incomplete(i)),
      Err(Err::Error(e)) => Err(Err::Error(E::add_context(input, self.context, e))),
      Err(Err::Failure(e)) => Err(Err::Failure(E::add_context(input, self.context, e))),
    }
  }
}

impl<P, I, O, E> Parser<I, O, E> for Context<P, I, O, E>
where
  P: Parser<I, O, E>,
  I: Clone,
  E: ContextError<I>,
{
  fn parse(&mut self, input: I) -> IResult<I, O, E> {
    self.parse(input)
  }
}

/// Transforms a `VerboseError` into a trace with input position information
#[cfg(feature = "alloc")]
pub fn convert_error<I: core::ops::Deref<Target = str>>(
  input: I,
  e: VerboseError<I>,
) -> crate::lib::std::string::String {
  use crate::lib::std::fmt::Write;
  use crate::traits::Offset;

  let mut result = crate::lib::std::string::String::new();

  for (i, (substring, kind)) in e.errors.iter().enumerate() {
    let offset = input.offset(substring);

    if input.is_empty() {
      match kind {
        VerboseErrorKind::Char(c) => {
          write!(&mut result, "{}: expected '{}', got empty input\n\n", i, c)
        }
        VerboseErrorKind::Context(s) => write!(&mut result, "{}: in {}, got empty input\n\n", i, s),
        VerboseErrorKind::Nom(e) => write!(&mut result, "{}: in {:?}, got empty input\n\n", i, e),
      }
    } else {
      let prefix = &input.as_bytes()[..offset];

      // Count the number of newlines in the first `offset` bytes of input
      let line_number = prefix.iter().filter(|&&b| b == b'\n').count() + 1;

      // Find the line that includes the subslice:
      // Find the *last* newline before the substring starts
      let line_begin = prefix
        .iter()
        .rev()
        .position(|&b| b == b'\n')
        .map(|pos| offset - pos)
        .unwrap_or(0);

      // Find the full line after that newline
      let line = input[line_begin..]
        .lines()
        .next()
        .unwrap_or(&input[line_begin..])
        .trim_end();

      // The (1-indexed) column number is the offset of our substring into that line
      let column_number = line.offset(substring) + 1;

      match kind {
        VerboseErrorKind::Char(c) => {
          if let Some(actual) = substring.chars().next() {
            write!(
              &mut result,
              "{i}: at line {line_number}:\n\
               {line}\n\
               {caret:>column$}\n\
               expected '{expected}', found {actual}\n\n",
              i = i,
              line_number = line_number,
              line = line,
              caret = '^',
              column = column_number,
              expected = c,
              actual = actual,
            )
          } else {
            write!(
              &mut result,
              "{i}: at line {line_number}:\n\
               {line}\n\
               {caret:>column$}\n\
               expected '{expected}', got end of input\n\n",
              i = i,
              line_number = line_number,
              line = line,
              caret = '^',
              column = column_number,
              expected = c,
            )
          }
        }
        VerboseErrorKind::Context(s) => write!(
          &mut result,
          "{i}: at line {line_number}, in {context}:\n\
             {line}\n\
             {caret:>column$}\n\n",
          i = i,
          line_number = line_number,
          context = s,
          line = line,
          caret = '^',
          column = column_number,
        ),
        VerboseErrorKind::Nom(e) => write!(
          &mut result,
          "{i}: at line {line_number}, in {nom_err:?}:\n\
             {line}\n\
             {caret:>column$}\n\n",
          i = i,
          line_number = line_number,
          nom_err = e,
          line = line,
          caret = '^',
          column = column_number,
        ),
      }
    }
    // Because `write!` to a `String` is infallible, this `unwrap` is fine.
    .unwrap();
  }

  result
}

/// Indicates which parser returned an error
#[rustfmt::skip]
#[derive(Debug,PartialEq,Eq,Hash,Clone,Copy)]
#[allow(deprecated,missing_docs)]
pub enum ErrorKind {
  Tag,
  MapRes,
  MapOpt,
  Alt,
  IsNot,
  IsA,
  SeparatedList,
  SeparatedNonEmptyList,
  Many0,
  Many1,
  ManyTill,
  Count,
  TakeUntil,
  LengthValue,
  TagClosure,
  Alpha,
  Digit,
  HexDigit,
  OctDigit,
  AlphaNumeric,
  Space,
  MultiSpace,
  LengthValueFn,
  Eof,
  Switch,
  TagBits,
  OneOf,
  NoneOf,
  Char,
  CrLf,
  RegexpMatch,
  RegexpMatches,
  RegexpFind,
  RegexpCapture,
  RegexpCaptures,
  TakeWhile1,
  Complete,
  Fix,
  Escaped,
  EscapedTransform,
  NonEmpty,
  ManyMN,
  Not,
  Permutation,
  Verify,
  TakeTill1,
  TakeWhileMN,
  TooLarge,
  Many0Count,
  Many1Count,
  Float,
  Satisfy,
  Fail,
}

#[rustfmt::skip]
#[allow(deprecated)]
/// Converts an ErrorKind to a number
pub fn error_to_u32(e: &ErrorKind) -> u32 {
  match *e {
    ErrorKind::Tag                       => 1,
    ErrorKind::MapRes                    => 2,
    ErrorKind::MapOpt                    => 3,
    ErrorKind::Alt                       => 4,
    ErrorKind::IsNot                     => 5,
    ErrorKind::IsA                       => 6,
    ErrorKind::SeparatedList             => 7,
    ErrorKind::SeparatedNonEmptyList     => 8,
    ErrorKind::Many1                     => 9,
    ErrorKind::Count                     => 10,
    ErrorKind::TakeUntil                 => 12,
    ErrorKind::LengthValue               => 15,
    ErrorKind::TagClosure                => 16,
    ErrorKind::Alpha                     => 17,
    ErrorKind::Digit                     => 18,
    ErrorKind::AlphaNumeric              => 19,
    ErrorKind::Space                     => 20,
    ErrorKind::MultiSpace                => 21,
    ErrorKind::LengthValueFn             => 22,
    ErrorKind::Eof                       => 23,
    ErrorKind::Switch                    => 27,
    ErrorKind::TagBits                   => 28,
    ErrorKind::OneOf                     => 29,
    ErrorKind::NoneOf                    => 30,
    ErrorKind::Char                      => 40,
    ErrorKind::CrLf                      => 41,
    ErrorKind::RegexpMatch               => 42,
    ErrorKind::RegexpMatches             => 43,
    ErrorKind::RegexpFind                => 44,
    ErrorKind::RegexpCapture             => 45,
    ErrorKind::RegexpCaptures            => 46,
    ErrorKind::TakeWhile1                => 47,
    ErrorKind::Complete                  => 48,
    ErrorKind::Fix                       => 49,
    ErrorKind::Escaped                   => 50,
    ErrorKind::EscapedTransform          => 51,
    ErrorKind::NonEmpty                  => 56,
    ErrorKind::ManyMN                    => 57,
    ErrorKind::HexDigit                  => 59,
    ErrorKind::OctDigit                  => 61,
    ErrorKind::Many0                     => 62,
    ErrorKind::Not                       => 63,
    ErrorKind::Permutation               => 64,
    ErrorKind::ManyTill                  => 65,
    ErrorKind::Verify                    => 66,
    ErrorKind::TakeTill1                 => 67,
    ErrorKind::TakeWhileMN               => 69,
    ErrorKind::TooLarge                  => 70,
    ErrorKind::Many0Count                => 71,
    ErrorKind::Many1Count                => 72,
    ErrorKind::Float                     => 73,
    ErrorKind::Satisfy                   => 74,
    ErrorKind::Fail                      => 75,
  }
}

impl ErrorKind {
  #[rustfmt::skip]
  #[allow(deprecated)]
  /// Converts an ErrorKind to a text description
  pub fn description(&self) -> &str {
    match *self {
      ErrorKind::Tag                       => "Tag",
      ErrorKind::MapRes                    => "Map on Result",
      ErrorKind::MapOpt                    => "Map on Option",
      ErrorKind::Alt                       => "Alternative",
      ErrorKind::IsNot                     => "IsNot",
      ErrorKind::IsA                       => "IsA",
      ErrorKind::SeparatedList             => "Separated list",
      ErrorKind::SeparatedNonEmptyList     => "Separated non empty list",
      ErrorKind::Many0                     => "Many0",
      ErrorKind::Many1                     => "Many1",
      ErrorKind::Count                     => "Count",
      ErrorKind::TakeUntil                 => "Take until",
      ErrorKind::LengthValue               => "Length followed by value",
      ErrorKind::TagClosure                => "Tag closure",
      ErrorKind::Alpha                     => "Alphabetic",
      ErrorKind::Digit                     => "Digit",
      ErrorKind::AlphaNumeric              => "AlphaNumeric",
      ErrorKind::Space                     => "Space",
      ErrorKind::MultiSpace                => "Multiple spaces",
      ErrorKind::LengthValueFn             => "LengthValueFn",
      ErrorKind::Eof                       => "End of file",
      ErrorKind::Switch                    => "Switch",
      ErrorKind::TagBits                   => "Tag on bitstream",
      ErrorKind::OneOf                     => "OneOf",
      ErrorKind::NoneOf                    => "NoneOf",
      ErrorKind::Char                      => "Char",
      ErrorKind::CrLf                      => "CrLf",
      ErrorKind::RegexpMatch               => "RegexpMatch",
      ErrorKind::RegexpMatches             => "RegexpMatches",
      ErrorKind::RegexpFind                => "RegexpFind",
      ErrorKind::RegexpCapture             => "RegexpCapture",
      ErrorKind::RegexpCaptures            => "RegexpCaptures",
      ErrorKind::TakeWhile1                => "TakeWhile1",
      ErrorKind::Complete                  => "Complete",
      ErrorKind::Fix                       => "Fix",
      ErrorKind::Escaped                   => "Escaped",
      ErrorKind::EscapedTransform          => "EscapedTransform",
      ErrorKind::NonEmpty                  => "NonEmpty",
      ErrorKind::ManyMN                    => "Many(m, n)",
      ErrorKind::HexDigit                  => "Hexadecimal Digit",
      ErrorKind::OctDigit                  => "Octal digit",
      ErrorKind::Not                       => "Negation",
      ErrorKind::Permutation               => "Permutation",
      ErrorKind::ManyTill                  => "ManyTill",
      ErrorKind::Verify                    => "predicate verification",
      ErrorKind::TakeTill1                 => "TakeTill1",
      ErrorKind::TakeWhileMN               => "TakeWhileMN",
      ErrorKind::TooLarge                  => "Needed data size is too large",
      ErrorKind::Many0Count                => "Count occurrence of >=0 patterns",
      ErrorKind::Many1Count                => "Count occurrence of >=1 patterns",
      ErrorKind::Float                     => "Float",
      ErrorKind::Satisfy                   => "Satisfy",
      ErrorKind::Fail                      => "Fail",
    }
  }
}

/// Creates a parse error from a [`ErrorKind`]
/// and the position in the input
#[allow(unused_variables)]
#[macro_export(local_inner_macros)]
macro_rules! error_position(
  ($input:expr, $code:expr) => ({
    $crate::error::make_error($input, $code)
  });
);

/// Creates a parse error from a [`ErrorKind`],
/// the position in the input and the next error in
/// the parsing tree
#[allow(unused_variables)]
#[macro_export(local_inner_macros)]
macro_rules! error_node_position(
  ($input:expr, $code:expr, $next:expr) => ({
    $crate::error::append_error($input, $code, $next)
  });
);

/// Prints a message and the input if the parser fails.
///
/// The message prints the `Error` or `Incomplete`
/// and the parser's calling code.
///
/// It also displays the input in hexdump format
///
/// ```rust
/// use nom::{IResult, error::dbg_dmp, bytes::complete::tag};
///
/// fn f(i: &[u8]) -> IResult<&[u8], &[u8]> {
///   dbg_dmp(tag("abcd"), "tag").parse(i)
/// }
///
///   let a = &b"efghijkl"[..];
///
/// // Will print the following message:
/// // Error(Position(0, [101, 102, 103, 104, 105, 106, 107, 108])) at l.5 by ' tag ! ( "abcd" ) '
/// // 00000000        65 66 67 68 69 6a 6b 6c         efghijkl
/// f(a);
/// ```
#[cfg(feature = "std")]
pub fn dbg_dmp<P, O, E>(parser: P, context: &'static str) -> DbgDmp<P, O, E> {
  DbgDmp {
    parser,
    context,
    o: Default::default(),
    e: Default::default(),
  }
}

/// Implementation of [`dbg_dmp`]
#[cfg(feature = "std")]
pub struct DbgDmp<P, O, E> {
  parser: P,
  context: &'static str,
  o: core::marker::PhantomData<O>,
  e: core::marker::PhantomData<E>,
}

#[cfg(feature = "std")]
impl<'i, P, O, E> DbgDmp<P, O, E>
where
  P: Parser<&'i [u8], O, E>,
  E: std::fmt::Debug,
{
  /// See [`Parser::parse`]
  pub fn parse(&mut self, input: &'i [u8]) -> IResult<&'i [u8], O, E> {
    use crate::HexDisplay;
    match self.parser.parse(input) {
      Err(e) => {
        println!("{}: Error({:?}) at:\n{}", self.context, e, input.to_hex(8));
        Err(e)
      }
      a => a,
    }
  }
}

#[cfg(feature = "std")]
impl<'i, P, O, E> Parser<&'i [u8], O, E> for DbgDmp<P, O, E>
where
  P: Parser<&'i [u8], O, E>,
  E: std::fmt::Debug,
{
  fn parse(&mut self, input: &'i [u8]) -> IResult<&'i [u8], O, E> {
    self.parse(input)
  }
}

#[cfg(test)]
#[cfg(feature = "alloc")]
mod tests {
  use super::*;
  use crate::character::complete::char;

  #[test]
  fn convert_error_panic() {
    let input = "";

    let _result: IResult<_, _, VerboseError<&str>> = char('x')(input);
  }
}

/*
#[cfg(feature = "alloc")]
use lib::std::{vec::Vec, collections::HashMap};

#[cfg(feature = "std")]
use lib::std::hash::Hash;

#[cfg(feature = "std")]
pub fn add_error_pattern<'a, I: Clone + Hash + Eq, O, E: Clone + Hash + Eq>(
  h: &mut HashMap<VerboseError<I>, &'a str>,
  e: VerboseError<I>,
  message: &'a str,
) -> bool {
  h.insert(e, message);
  true
}

pub fn slice_to_offsets(input: &[u8], s: &[u8]) -> (usize, usize) {
  let start = input.as_ptr();
  let off1 = s.as_ptr() as usize - start as usize;
  let off2 = off1 + s.len();
  (off1, off2)
}

#[cfg(feature = "std")]
pub fn prepare_errors<O, E: Clone>(input: &[u8], e: VerboseError<&[u8]>) -> Option<Vec<(ErrorKind, usize, usize)>> {
  let mut v: Vec<(ErrorKind, usize, usize)> = Vec::new();

  for (p, kind) in e.errors.drain(..) {
    let (o1, o2) = slice_to_offsets(input, p);
    v.push((kind, o1, o2));
  }

  v.reverse();
  Some(v)
}

#[cfg(feature = "std")]
pub fn print_error<O, E: Clone>(input: &[u8], res: VerboseError<&[u8]>) {
  if let Some(v) = prepare_errors(input, res) {
    let colors = generate_colors(&v);
    println!("parser codes: {}", print_codes(&colors, &HashMap::new()));
    println!("{}", print_offsets(input, 0, &v));
  } else {
    println!("not an error");
  }
}

#[cfg(feature = "std")]
pub fn generate_colors<E>(v: &[(ErrorKind, usize, usize)]) -> HashMap<u32, u8> {
  let mut h: HashMap<u32, u8> = HashMap::new();
  let mut color = 0;

  for &(ref c, _, _) in v.iter() {
    h.insert(error_to_u32(c), color + 31);
    color = color + 1 % 7;
  }

  h
}

pub fn code_from_offset(v: &[(ErrorKind, usize, usize)], offset: usize) -> Option<u32> {
  let mut acc: Option<(u32, usize, usize)> = None;
  for &(ref ek, s, e) in v.iter() {
    let c = error_to_u32(ek);
    if s <= offset && offset <= e {
      if let Some((_, start, end)) = acc {
        if start <= s && e <= end {
          acc = Some((c, s, e));
        }
      } else {
        acc = Some((c, s, e));
      }
    }
  }
  if let Some((code, _, _)) = acc {
    return Some(code);
  } else {
    return None;
  }
}

#[cfg(feature = "alloc")]
pub fn reset_color(v: &mut Vec<u8>) {
  v.push(0x1B);
  v.push(b'[');
  v.push(0);
  v.push(b'm');
}

#[cfg(feature = "alloc")]
pub fn write_color(v: &mut Vec<u8>, color: u8) {
  v.push(0x1B);
  v.push(b'[');
  v.push(1);
  v.push(b';');
  let s = color.to_string();
  let bytes = s.as_bytes();
  v.extend(bytes.iter().cloned());
  v.push(b'm');
}

#[cfg(feature = "std")]
#[cfg_attr(feature = "cargo-clippy", allow(implicit_hasher))]
pub fn print_codes(colors: &HashMap<u32, u8>, names: &HashMap<u32, &str>) -> String {
  let mut v = Vec::new();
  for (code, &color) in colors {
    if let Some(&s) = names.get(code) {
      let bytes = s.as_bytes();
      write_color(&mut v, color);
      v.extend(bytes.iter().cloned());
    } else {
      let s = code.to_string();
      let bytes = s.as_bytes();
      write_color(&mut v, color);
      v.extend(bytes.iter().cloned());
    }
    reset_color(&mut v);
    v.push(b' ');
  }
  reset_color(&mut v);

  String::from_utf8_lossy(&v[..]).into_owned()
}

#[cfg(feature = "std")]
pub fn print_offsets(input: &[u8], from: usize, offsets: &[(ErrorKind, usize, usize)]) -> String {
  let mut v = Vec::with_capacity(input.len() * 3);
  let mut i = from;
  let chunk_size = 8;
  let mut current_code: Option<u32> = None;
  let mut current_code2: Option<u32> = None;

  let colors = generate_colors(&offsets);

  for chunk in input.chunks(chunk_size) {
    let s = format!("{:08x}", i);
    for &ch in s.as_bytes().iter() {
      v.push(ch);
    }
    v.push(b'\t');

    let mut k = i;
    let mut l = i;
    for &byte in chunk {
      if let Some(code) = code_from_offset(&offsets, k) {
        if let Some(current) = current_code {
          if current != code {
            reset_color(&mut v);
            current_code = Some(code);
            if let Some(&color) = colors.get(&code) {
              write_color(&mut v, color);
            }
          }
        } else {
          current_code = Some(code);
          if let Some(&color) = colors.get(&code) {
            write_color(&mut v, color);
          }
        }
      }
      v.push(CHARS[(byte >> 4) as usize]);
      v.push(CHARS[(byte & 0xf) as usize]);
      v.push(b' ');
      k = k + 1;
    }

    reset_color(&mut v);

    if chunk_size > chunk.len() {
      for _ in 0..(chunk_size - chunk.len()) {
        v.push(b' ');
        v.push(b' ');
        v.push(b' ');
      }
    }
    v.push(b'\t');

    for &byte in chunk {
      if let Some(code) = code_from_offset(&offsets, l) {
        if let Some(current) = current_code2 {
          if current != code {
            reset_color(&mut v);
            current_code2 = Some(code);
            if let Some(&color) = colors.get(&code) {
              write_color(&mut v, color);
            }
          }
        } else {
          current_code2 = Some(code);
          if let Some(&color) = colors.get(&code) {
            write_color(&mut v, color);
          }
        }
      }
      if (byte >= 32 && byte <= 126) || byte >= 128 {
        v.push(byte);
      } else {
        v.push(b'.');
      }
      l = l + 1;
    }
    reset_color(&mut v);

    v.push(b'\n');
    i = i + chunk_size;
  }

  String::from_utf8_lossy(&v[..]).into_owned()
}
*/
