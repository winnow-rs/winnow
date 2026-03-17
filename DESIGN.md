## Compatibility Expectations

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
#[deprecated(since = "X.Y.Z", note = "replaced with `ITEM` in Issue #XXX")]`
```

For easier review, use separate commits for:
- Introducing the new API
- Deprecating the old API
- Updating internal and documentation away from the old API

### `Parser`s

Design guidelines
- Generally grammar-level `Parser`s are free-functions and output/error
  conversion are inherent functions on `Parser`.  `Parser::verify` is an
  example of some nuance as the logic is coupled to the `Parser` its applied
  to.
- `Parser`s that directly process tokens must support complete vs streaming
  parsing.
- `Parser`s that work with slices have `take` in their name.
- When taking slices or repeatedly calling a `Parser`, control the number of
  times with a range, rather than hard coding it with the range in the name.
- Where possible, write `Parser`s in a straight-forward manner, reusing other
  `Parser`s, so they may serve as examples for the user.
