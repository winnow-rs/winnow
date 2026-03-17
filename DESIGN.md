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
