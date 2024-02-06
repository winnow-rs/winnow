/// One of the 128 Unicode characters from U+0000 through U+007F,
/// often known as the [ASCII] subset.
///
/// Officially, this is the first [block] in Unicode, _Basic Latin_.
/// For details, see the [*C0 Controls and Basic Latin*][chart] code chart.
///
/// This block was based on older 7-bit character code standards such as
/// ANSI X3.4-1977, ISO 646-1973, and [NIST FIPS 1-2].
///
/// **Note:** This is a polyfill for [`ascii::Char`][std::ascii::Char].
///
/// # When to use this
///
/// The main advantage of this subset is that it's always valid UTF-8.  As such,
/// the `&[ascii::Char]` -> `&str` conversion function (as well as other related
/// ones) are O(1): *no* runtime checks are needed.
///
/// If you're consuming strings, you should usually handle Unicode and thus
/// accept `str`s, not limit yourself to `ascii::Char`s.
///
/// However, certain formats are intentionally designed to produce ASCII-only
/// output in order to be 8-bit-clean.  In those cases, it can be simpler and
/// faster to generate `ascii::Char`s instead of dealing with the variable width
/// properties of general UTF-8 encoded strings, while still allowing the result
/// to be used freely with other Rust things that deal in general `str`s.
///
/// For example, a UUID library might offer a way to produce the string
/// representation of a UUID as an `[ascii::Char; 36]` to avoid memory
/// allocation yet still allow it to be used as UTF-8 via `as_str` without
/// paying for validation (or needing `unsafe` code) the way it would if it
/// were provided as a `[u8; 36]`.
///
/// # Layout
///
/// This type is guaranteed to have a size and alignment of 1 byte.
///
/// # Names
///
/// The variants on this type are [Unicode names][NamesList] of the characters
/// in upper camel case, with a few tweaks:
/// - For `<control>` characters, the primary alias name is used.
/// - `LATIN` is dropped, as this block has no non-latin letters.
/// - `LETTER` is dropped, as `CAPITAL`/`SMALL` suffices in this block.
/// - `DIGIT`s use a single digit rather than writing out `ZERO`, `ONE`, etc.
///
/// [ASCII]: https://www.unicode.org/glossary/index.html#ASCII
/// [block]: https://www.unicode.org/glossary/index.html#block
/// [chart]: https://www.unicode.org/charts/PDF/U0000.pdf
/// [NIST FIPS 1-2]: https://nvlpubs.nist.gov/nistpubs/Legacy/FIPS/fipspub1-2-1977.pdf
/// [NamesList]: https://www.unicode.org/Public/15.0.0/ucd/NamesList.txt
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AsciiChar(u8);

impl AsciiChar {
    /// Creates an ascii character from the byte `b`,
    /// or returns `None` if it's too large.
    #[inline(always)]
    pub const fn from_u8(b: u8) -> Option<Self> {
        if b <= 127 {
            // SAFETY: Just checked that `b` is in-range
            Some(unsafe { Self::from_u8_unchecked(b) })
        } else {
            None
        }
    }

    /// Creates an ASCII character from the byte `b`,
    /// without checking whether it's valid.
    ///
    /// # Safety
    ///
    /// `b` must be in `0..=127`, or else this is UB.
    #[inline(always)]
    pub const unsafe fn from_u8_unchecked(b: u8) -> Self {
        Self(b)
    }

    /// Gets this ASCII character as a `char` Unicode Scalar Value.
    #[inline(always)]
    const fn to_char(self) -> char {
        self.0 as char
    }
}

impl crate::lib::std::fmt::Display for AsciiChar {
    fn fmt(&self, f: &mut crate::lib::std::fmt::Formatter<'_>) -> crate::lib::std::fmt::Result {
        self.to_char().fmt(f)
    }
}

impl crate::lib::std::fmt::Debug for AsciiChar {
    fn fmt(&self, f: &mut crate::lib::std::fmt::Formatter<'_>) -> crate::lib::std::fmt::Result {
        self.to_char().fmt(f)
    }
}
