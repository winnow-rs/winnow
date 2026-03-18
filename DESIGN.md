## Compatibility expectations

We aspire to the following release cadence:
- Major releases (breaking changes): 6-9 months between releases
- Minor releases (minor incompatibilities): 2 months between releases
- Patch releases: one per user-facing, user-contributed PR

Most development should fit within a patch release, meaning:
- No breaking changes, this requires a major release
- No MSRV bump, this requires a minor release

Anything outside of that should be coordinated beforehand.

Practices to minimize breaking changes and reduce their impact:

### Minimize breaking changes

Develop new, extremely large features behind an `unstable-<name>` feature flag with a stabilization tracking issue.

### Deprecate, rather than break

Most breaking changes can be introduced today by offering a parallel API,
deprecating the existing one.

When deprecating an API, replace the docstring with the following:
```rust
/// Deprecated in [Issue #XXX](https://github.com/winnow-rs/winnow/issues/XXX), replaced with [intra-doc-link]`
#[doc(alias = "OLD_NAME")]
#[deprecated(since = "X.Y.Z", note = "replaced with `ITEM` in Issue #XXX")]`
```

For easier review, use separate commits for:
- Introducing the new API
- Deprecating the old API
- Updating internal and documentation away from the old API

## `Parser`s

Principles:
- A parser's grammar should be reflected in the Rust grammar
  - Grammar-level `Parser`s should be free functions
- Users can easily discover what they need
  - Flat APIs are easier for users to process
  - The closer to the root of the API, the more generally used it should be
  - Small, composable APIs are easier for users to process

Organization:
- `Parser::*`: limited to non-grammatical output/error conversions
- `combinator`: parser composition and basic building blocks
- `token`: `Stream` agnostic token and token slice processing
- `ascii`: Oriented around ASCII data processing
- `binary`: Oriented around byte and bit data processing
- `impls`: Opaque types that offer no additional behavior than the `Parser` trait

Invariants
- Parsers that loop over user input should `ParseError::assert` if the `Stream` does not advance
  - When multiple inputs are being evaluated in an iteration of the loop, prefer to check if one advances `Stream` so users are more likely to observe and fix the infinite loop

Guidelines
- `Parser`s that directly process tokens must support complete vs streaming
  parsing.
- When taking slices or repeatedly calling a `Parser`, control the number of
  times with a range, rather than hard coding it with the range in the name.
- Name for producing slices is `take`
- Name for the indices into a `Stream` to access a token is `offset`
- Name for the number of values, processed, returned, or expected to return is either a `count` or `occurrences`
- Where possible, write `Parser`s in a straight-forward manner, reusing other
  `Parser`s, so they may serve as examples for the user.
- `doc(alias)` parsers for other names people may expect to use to find the functionality

See also
- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/checklist.html)

## Performance

Keep the code simple unless a real-world benchmark shows benefits to a more complicated design.
Microbenchmarks are not always sufficient as performance can vary from real-world cases.

### Limit code generation for tuples

Implementing traits for tuples can generate a lot of code.

Limit trait implementations for tuples to 10 elements.

Identify if a non-generic core can be split out.

Consider alternatives, like macros.

### Inlined wrapper around specialized cores

The compiler does not always make the right inlining decisions that can lead to optimizations.
One way of helping the compiler is to have a very thin `#[inline(always)]` wrapper that
dispatches to one of several function bodies
(whether syntactically or through monomorphization).
