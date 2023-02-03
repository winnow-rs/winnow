# winnow, making parsing a breeze

[![LICENSE](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Build Status](https://github.com/winnow-rs/winnow/actions/workflows/ci.yml/badge.svg)](https://github.com/winnow-rs/winnow/actions/workflows/ci.yml)
[![Coverage Status](https://coveralls.io/repos/github/winnow-rs/winnow/badge.svg?branch=main)](https://coveralls.io/github/winnow-rs/winnow?branch=main)
[![Crates.io Version](https://img.shields.io/crates/v/winnow.svg)](https://crates.io/crates/winnow)

winnow is a parser combinators library written in Rust. Its goal is to provide tools
to build safe parsers without compromising the speed or memory consumption. To
that end, it uses extensively Rust's *strong typing* and *memory safety* to produce
fast and correct parsers, and provides functions, macros and traits to abstract most of the
error prone plumbing.

<!-- toc -->

- [Example](#example)
- [Documentation](#documentation)
- [Why use winnow?](#why-use-winnow)
    - [Binary format parsers](#binary-format-parsers)
    - [Text format parsers](#text-format-parsers)
    - [Programming language parsers](#programming-language-parsers)
    - [Streaming formats](#streaming-formats)
- [Parser combinators](#parser-combinators)
- [Technical features](#technical-features)
- [Rust version requirements](#rust-version-requirements-msrv)
- [Installation](#installation)
- [Related projects](#related-projects)
- [Parsers written with winnow](#parsers-written-with-winnow)
- [Contributors](#contributors)

<!-- tocstop -->

## Example

[Hexadecimal color](https://developer.mozilla.org/en-US/docs/Web/CSS/color) parser:

```rust
use winnow::prelude::*;
use winnow::{
  IResult,
  bytes::complete::{tag, take_while_m_n},
  sequence::tuple
};

#[derive(Debug,PartialEq)]
pub struct Color {
  pub red:   u8,
  pub green: u8,
  pub blue:  u8,
}

fn from_hex(input: &str) -> Result<u8, std::num::ParseIntError> {
  u8::from_str_radix(input, 16)
}

fn is_hex_digit(c: char) -> bool {
  c.is_digit(16)
}

fn hex_primary(input: &str) -> IResult<&str, u8> {
  take_while_m_n(2, 2, is_hex_digit).map_res(from_hex).parse(input)
}

fn hex_color(input: &str) -> IResult<&str, Color> {
  let (input, _) = tag("#")(input)?;
  let (input, (red, green, blue)) = tuple((hex_primary, hex_primary, hex_primary))(input)?;

  Ok((input, Color { red, green, blue }))
}

fn main() {}

#[test]
fn parse_color() {
  assert_eq!(hex_color("#2F14DF"), Ok(("", Color {
    red: 47,
    green: 20,
    blue: 223,
  })));
}
```

## Documentation

- [Reference documentation](https://docs.rs/winnow)
- [Various design documents and tutorials](https://github.com/winnow-rs/winnow/tree/main/doc)
- [List of combinators and their behaviour](https://docs.rs/winno/latest/winno/combinator/index.html)

## Why use winnow

If you want to write:

### Binary format parsers

winnow was designed to properly parse binary formats from the beginning. Compared
to the usual handwritten C parsers, winnow parsers are just as fast, free from
buffer overflow vulnerabilities, and handle common patterns for you:

- [TLV](https://en.wikipedia.org/wiki/Type-length-value)
- Bit level parsing
- Hexadecimal viewer in the debugging macros for easy data analysis
- Streaming parsers for network formats and huge files

Example projects:

- [FLV parser](https://github.com/rust-av/flavors)
- [Matroska parser](https://github.com/rust-av/matroska)
- [tar parser](https://github.com/Keruspe/tar-parser.rs)

### Text format parsers

While winnow was made for binary format at first, it soon grew to work just as
well with text formats. From line based formats like CSV, to more complex, nested
formats such as JSON, winnow can manage it, and provides you with useful tools:

- Fast case insensitive comparison
- Recognizers for escaped strings
- Regular expressions can be embedded in winnow parsers to represent complex character patterns succinctly
- Special care has been given to managing non ASCII characters properly

Example projects:

- [HTTP proxy](https://github.com/sozu-proxy/sozu/tree/main/lib/src/protocol/http/parser)
- [TOML parser](https://github.com/joelself/tomllib)

### Programming language parsers

While programming language parsers are usually written manually for more
flexibility and performance, winnow can be (and has been successfully) used
as a prototyping parser for a language.

winnow will get you started quickly with powerful custom error types, that you
can use to
pinpoint the exact line and column of the error. No need for separate
tokenizing, lexing and parsing phases: winnow can automatically handle whitespace
parsing, and construct an AST in place.

### Streaming formats

While a lot of formats (and the code handling them) assume that they can fit
the complete data in memory, there are formats for which we only get a part
of the data at once, like network formats, or huge files.
winnow has been designed for a correct behaviour with partial data: If there is
not enough data to decide, winnow will tell you it needs more instead of silently
returning a wrong result. Whether your data comes entirely or in chunks, the
result should be the same.

It allows you to build powerful, deterministic state machines for your protocols.

Example projects:

- [HTTP proxy](https://github.com/sozu-proxy/sozu/tree/main/lib/src/protocol/http/parser)
- [Using nom with generators](https://github.com/Geal/generator_nom)

## Parser combinators

Parser combinators are an approach to parsers that is very different from
software like [lex](https://en.wikipedia.org/wiki/Lex_(software)) and
[yacc](https://en.wikipedia.org/wiki/Yacc). Instead of writing the grammar
in a separate file and generating the corresponding code, you use very
small functions with very specific purpose, like "take 5 bytes", or
"recognize the word 'HTTP'", and assemble them in meaningful patterns
like "recognize 'HTTP', then a space, then a version".
The resulting code is small, and looks like the grammar you would have
written with other parser approaches.

This has a few advantages:

- The parsers are small and easy to write
- The parsers components are easy to reuse (if they're general enough, please add them to winnow!)
- The parsers components are easy to test separately (unit tests and property-based tests)
- The parser combination code looks close to the grammar you would have written
- You can build partial parsers, specific to the data you need at the moment, and ignore the rest

## Technical features

winnow parsers are for:
- [x] **byte-oriented**: The basic type is `&[u8]` and parsers will work as much as possible on byte array slices (but are not limited to them)
- [x] **bit-oriented**: winnow can address a byte slice as a bit stream
- [x] **string-oriented**: The same kind of combinators can apply on UTF-8 strings as well
- [x] **zero-copy**: If a parser returns a subset of its input data, it will return a slice of that input, without copying
- [x] **streaming**: winnow can work on partial data and detect when it needs more data to produce a correct result
- [x] **descriptive errors**: The parsers can aggregate a list of error codes with pointers to the incriminated input slice. Those error lists can be pattern matched to provide useful messages.
- [x] **custom error types**: You can provide a specific type to improve errors returned by parsers
- [x] **safe parsing**: winnow leverages Rust's safe memory handling and powerful types, and parsers are routinely fuzzed and tested with real world data. So far, the only flaws found by fuzzing were in code written outside of winnow
- [x] **speed**: Benchmarks have shown that winnow parsers often outperform many parser combinators library like Parsec and attoparsec, some regular expression engines and even handwritten C parsers

Some benchmarks are available on [Github](https://github.com/rosetta-rs/parser-rosetta-rs).

## Rust version requirements (MSRV)

The 7.0 series of wdnnow has **a minimum-supported Rust version (MSRV) of 1.51.0**. It is known to work properly on Rust 1.41.1 but there is no guarantee it will stay the case through this major release.

The current policy is that this will only be updated in the next major winnow release.

## Installation

winnow is available on [crates.io](https://crates.io/crates/winnow) and can be included in your Cargo enabled project like this:

```console
$ cargo add winnow
```

There are a few compilation features:

* `alloc`: (activated by default) if disabled, winnow can work in `no_std` builds without memory allocators. If enabled, combinators that allocate (like `many0`) will be available
* `std`: (activated by default, activates `alloc` too) if disabled, winnow can work in `no_std` builds

You can configure those features like this:

```toml
$ cargo add winnow --no-default-features --features alloc
```

# Related projects

- [Using nom as lexer and parser](https://github.com/Rydgel/monkey-rust)

# Parsers written with winnow

Here is a (non exhaustive) list of known projects using winnow:

Want to create a new parser using `winnow`? A list of not yet implemented formats is available [here](https://github.com/winnow-rs/winnow/issues/14).

Want to add your parser here? Create a pull request for it!

# Contributors

winnow is the fruit of the work of many contributors over the years, many
thanks for your help!  In particular, thanks to [Geal](https://github.com/Geal)
for the original [`nom` crate](https://crates.io/crates/nom).

<a href="https://github.com/winnow-rs/winnow/graphs/contributors">
  <img src="https://contributors-img.web.app/image?repo=winnow-rs/winnow" />
</a>
