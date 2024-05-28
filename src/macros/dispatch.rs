/// `match` for parsers
///
/// When parsers have unique prefixes to test for, this offers better performance over
/// [`alt`][crate::combinator::alt] though it might be at the cost of duplicating parts of your grammar
/// if you needed to [`peek`][crate::combinator::peek].
///
/// For tight control over the error in a catch-all case, use [`fail`][crate::combinator::fail].
///
/// # Example
///
/// ```rust
/// use winnow::prelude::*;
/// use winnow::combinator::dispatch;
/// # use winnow::token::any;
/// # use winnow::combinator::peek;
/// # use winnow::combinator::preceded;
/// # use winnow::combinator::empty;
/// # use winnow::combinator::fail;
///
/// fn escaped(input: &mut &str) -> PResult<char> {
///     preceded('\\', escape_seq_char).parse_next(input)
/// }
///
/// fn escape_seq_char(input: &mut &str) -> PResult<char> {
///     dispatch! {any;
///         'b' => empty.value('\u{8}'),
///         'f' => empty.value('\u{c}'),
///         'n' => empty.value('\n'),
///         'r' => empty.value('\r'),
///         't' => empty.value('\t'),
///         '\\' => empty.value('\\'),
///         '"' => empty.value('"'),
///         _ => fail::<_, char, _>,
///     }
///     .parse_next(input)
/// }
///
/// assert_eq!(escaped.parse_peek("\\nHello"), Ok(("Hello", '\n')));
/// ```
#[macro_export]
#[doc(hidden)] // forced to be visible in intended location
macro_rules! __dispatch_cases {
    ([ ] $i:tt $initial:tt $($acc:tt)*) => {
        match $initial {
            $($acc)*
        }
    };
    ([ $pat:pat $(if $pred:expr)? => $expr:expr $(,)? ] $i:tt $($acc:tt)*) => {
        $crate::combinator::__dispatch_cases!([ ] $i $($acc)* $pat $(if $pred)? => $expr.parse_next($i),)
    };
    ([ $pat:pat $(if $pred:expr)? => $expr:expr, $($rest:tt)+ ] $i:tt $($acc:tt)*) => {
        $crate::combinator::__dispatch_cases!([ $($rest)+ ] $i $($acc)* $pat $(if $pred)? => $expr.parse_next($i),)
    };
    // NOTE: Must be the last rule otherwise the `{ ... },` (trailing comma) will fail to parse.
    ([ $pat:pat $(if $pred:expr)? => { $($body:tt)* } $($rest:tt)+ ] $i:tt $($acc:tt)*) => {
        $crate::combinator::__dispatch_cases!([ $($rest)+ ] $i $($acc)* $pat $(if $pred)? => { $($body)* }.parse_next($i),)
    };
}

#[macro_export]
#[doc(hidden)] // forced to be visible in intended location
macro_rules! dispatch {
    ($match_parser:expr; $($cases:tt)+) => {
        $crate::combinator::trace("dispatch", move |i: &mut _| {
            use $crate::Parser;
            let initial = $match_parser.parse_next(i)?;
            $crate::combinator::__dispatch_cases!([ $($cases)+ ] i initial)
        })
    };
}

#[cfg(test)]
mod test {
    #[test]
    fn parse_block_with_trailing_comma() {
        use crate::combinator::{dispatch, empty, fail, preceded};
        use crate::prelude::*;
        use crate::token::any;

        #[allow(unused)]
        fn escaped(input: &mut &str) -> PResult<char> {
            preceded('\\', escape_seq_char).parse_next(input)
        }

        #[allow(unused)]
        fn escape_seq_char(input: &mut &str) -> PResult<char> {
            dispatch! {any;
                'b' => {
                    empty.value('\u{8}')
                },
                'f' => empty.value('\u{c}'),
                _ => fail::<_, char, _>,
            }
            .parse_next(input)
        }
    }

    #[test]
    fn parse_block_sans_trailing_comma() {
        use crate::combinator::{dispatch, empty, fail, preceded};
        use crate::prelude::*;
        use crate::token::any;

        #[allow(unused)]
        fn escaped(input: &mut &str) -> PResult<char> {
            preceded('\\', escape_seq_char).parse_next(input)
        }

        #[allow(unused)]
        fn escape_seq_char(input: &mut &str) -> PResult<char> {
            dispatch! {any;
                'b' => {
                    empty.value('\u{8}')
                }
                'f' => empty.value('\u{c}'),
                _ => fail::<_, char, _>,
            }
            .parse_next(input)
        }
    }
}
