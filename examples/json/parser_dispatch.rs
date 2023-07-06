use std::collections::HashMap;
use std::str;

use winnow::prelude::*;
use winnow::{
    ascii::float,
    combinator::cut_err,
    combinator::fail,
    combinator::peek,
    combinator::success,
    combinator::{alt, dispatch},
    combinator::{delimited, preceded, separated_pair, terminated},
    combinator::{fold_repeat, separated0},
    error::{ContextError, ParseError},
    token::{any, none_of, take, take_while},
};

use crate::json::JsonValue;

pub type Stream<'i> = &'i str;

/// The root element of a JSON parser is any value
///
/// A parser has the following signature:
/// `&mut Stream -> PResult<Output, Error>`, with `PResult` defined as:
/// `type PResult<O, E = ErrorKind> = Result<O, ErrMode<E>>;`
///
/// most of the times you can ignore the error type and use the default (but this
/// examples shows custom error types later on!)
///
/// Here we use `&str` as input type, but parsers can be generic over
/// the input type, work directly with `&[u8]`, or any other type that
/// implements the required traits.
pub fn json<'i, E: ParseError<Stream<'i>> + ContextError<Stream<'i>, &'static str>>(
    input: &mut Stream<'i>,
) -> PResult<JsonValue, E> {
    delimited(ws, json_value, ws).parse_next(input)
}

/// `alt` is a combinator that tries multiple parsers one by one, until
/// one of them succeeds
fn json_value<'i, E: ParseError<Stream<'i>> + ContextError<Stream<'i>, &'static str>>(
    input: &mut Stream<'i>,
) -> PResult<JsonValue, E> {
    // `dispatch` gives you `match`-like behavior compared to `alt` successively trying different
    // implementations.
    dispatch!(peek(any);
        'n' => null.value(JsonValue::Null),
        't' => true_.map(JsonValue::Boolean),
        'f' => false_.map(JsonValue::Boolean),
        '"' => string.map(JsonValue::Str),
        '+' => float.map(JsonValue::Num),
        '-' => float.map(JsonValue::Num),
        '0'..='9' => float.map(JsonValue::Num),
        '[' => array.map(JsonValue::Array),
        '{' => object.map(JsonValue::Object),
        _ => fail,
    )
    .parse_next(input)
}

/// `tag(string)` generates a parser that recognizes the argument string.
///
/// This also shows returning a sub-slice of the original input
fn null<'i, E: ParseError<Stream<'i>>>(input: &mut Stream<'i>) -> PResult<&'i str, E> {
    // This is a parser that returns `"null"` if it sees the string "null", and
    // an error otherwise
    "null".parse_next(input)
}

/// We can combine `tag` with other functions, like `value` which returns a given constant value on
/// success.
fn true_<'i, E: ParseError<Stream<'i>>>(input: &mut Stream<'i>) -> PResult<bool, E> {
    // This is a parser that returns `true` if it sees the string "true", and
    // an error otherwise
    "true".value(true).parse_next(input)
}

/// We can combine `tag` with other functions, like `value` which returns a given constant value on
/// success.
fn false_<'i, E: ParseError<Stream<'i>>>(input: &mut Stream<'i>) -> PResult<bool, E> {
    // This is a parser that returns `false` if it sees the string "false", and
    // an error otherwise
    "false".value(false).parse_next(input)
}

/// This parser gathers all `char`s up into a `String`with a parse to recognize the double quote
/// character, before the string (using `preceded`) and after the string (using `terminated`).
fn string<'i, E: ParseError<Stream<'i>> + ContextError<Stream<'i>, &'static str>>(
    input: &mut Stream<'i>,
) -> PResult<String, E> {
    preceded(
        '\"',
        // `cut_err` transforms an `ErrMode::Backtrack(e)` to `ErrMode::Cut(e)`, signaling to
        // combinators like  `alt` that they should not try other parsers. We were in the
        // right branch (since we found the `"` character) but encountered an error when
        // parsing the string
        cut_err(terminated(
            fold_repeat(0.., character, String::new, |mut string, c| {
                string.push(c);
                string
            }),
            '\"',
        )),
    )
    // `context` lets you add a static string to errors to provide more information in the
    // error chain (to indicate which parser had an error)
    .context("string")
    .parse_next(input)
}

/// You can mix the above declarative parsing with an imperative style to handle more unique cases,
/// like escaping
fn character<'i, E: ParseError<Stream<'i>>>(input: &mut Stream<'i>) -> PResult<char, E> {
    let c = none_of('\"').parse_next(input)?;
    if c == '\\' {
        dispatch!(any;
          '"' => success('"'),
          '\\' => success('\\'),
          '/'  => success('/'),
          'b' => success('\x08'),
          'f' => success('\x0C'),
          'n' => success('\n'),
          'r' => success('\r'),
          't' => success('\t'),
          'u' => unicode_escape,
          _ => fail,
        )
        .parse_next(input)
    } else {
        Ok(c)
    }
}

fn unicode_escape<'i, E: ParseError<Stream<'i>>>(input: &mut Stream<'i>) -> PResult<char, E> {
    alt((
        // Not a surrogate
        u16_hex
            .verify(|cp| !(0xD800..0xE000).contains(cp))
            .map(|cp| cp as u32),
        // See https://en.wikipedia.org/wiki/UTF-16#Code_points_from_U+010000_to_U+10FFFF for details
        separated_pair(u16_hex, "\\u", u16_hex)
            .verify(|(high, low)| (0xD800..0xDC00).contains(high) && (0xDC00..0xE000).contains(low))
            .map(|(high, low)| {
                let high_ten = (high as u32) - 0xD800;
                let low_ten = (low as u32) - 0xDC00;
                (high_ten << 10) + low_ten + 0x10000
            }),
    ))
    .verify_map(
        // Could be probably replaced with .unwrap() or _unchecked due to the verify checks
        std::char::from_u32,
    )
    .parse_next(input)
}

fn u16_hex<'i, E: ParseError<Stream<'i>>>(input: &mut Stream<'i>) -> PResult<u16, E> {
    take(4usize)
        .verify_map(|s| u16::from_str_radix(s, 16).ok())
        .parse_next(input)
}

/// Some combinators, like `separated0` or `many0`, will call a parser repeatedly,
/// accumulating results in a `Vec`, until it encounters an error.
/// If you want more control on the parser application, check out the `iterator`
/// combinator (cf `examples/iterator.rs`)
fn array<'i, E: ParseError<Stream<'i>> + ContextError<Stream<'i>, &'static str>>(
    input: &mut Stream<'i>,
) -> PResult<Vec<JsonValue>, E> {
    preceded(
        ('[', ws),
        cut_err(terminated(separated0(json_value, (ws, ',', ws)), (ws, ']'))),
    )
    .context("array")
    .parse_next(input)
}

fn object<'i, E: ParseError<Stream<'i>> + ContextError<Stream<'i>, &'static str>>(
    input: &mut Stream<'i>,
) -> PResult<HashMap<String, JsonValue>, E> {
    preceded(
        ('{', ws),
        cut_err(terminated(separated0(key_value, (ws, ',', ws)), (ws, '}'))),
    )
    .context("object")
    .parse_next(input)
}

fn key_value<'i, E: ParseError<Stream<'i>> + ContextError<Stream<'i>, &'static str>>(
    input: &mut Stream<'i>,
) -> PResult<(String, JsonValue), E> {
    separated_pair(string, cut_err((ws, ':', ws)), json_value).parse_next(input)
}

/// Parser combinators are constructed from the bottom up:
/// first we write parsers for the smallest elements (here a space character),
/// then we'll combine them in larger parsers
fn ws<'i, E: ParseError<Stream<'i>>>(input: &mut Stream<'i>) -> PResult<&'i str, E> {
    // Combinators like `take_while` return a function. That function is the
    // parser,to which we can pass the input
    take_while(0.., WS).parse_next(input)
}

const WS: &[char] = &[' ', '\t', '\r', '\n'];

#[cfg(test)]
mod test {
    #[allow(clippy::useless_attribute)]
    #[allow(dead_code)] // its dead for benches
    use super::*;

    #[allow(clippy::useless_attribute)]
    #[allow(dead_code)] // its dead for benches
    type Error<'i> = winnow::error::Error<&'i str>;

    #[test]
    fn json_string() {
        assert_eq!(
            string::<Error<'_>>.parse_peek("\"\""),
            Ok(("", "".to_string()))
        );
        assert_eq!(
            string::<Error<'_>>.parse_peek("\"abc\""),
            Ok(("", "abc".to_string()))
        );
        assert_eq!(
            string::<Error<'_>>
                .parse_peek("\"abc\\\"\\\\\\/\\b\\f\\n\\r\\t\\u0001\\u2014\u{2014}def\""),
            Ok(("", "abc\"\\/\x08\x0C\n\r\t\x01——def".to_string())),
        );
        assert_eq!(
            string::<Error<'_>>.parse_peek("\"\\uD83D\\uDE10\""),
            Ok(("", "😐".to_string()))
        );

        assert!(string::<Error<'_>>.parse_peek("\"").is_err());
        assert!(string::<Error<'_>>.parse_peek("\"abc").is_err());
        assert!(string::<Error<'_>>.parse_peek("\"\\\"").is_err());
        assert!(string::<Error<'_>>.parse_peek("\"\\u123\"").is_err());
        assert!(string::<Error<'_>>.parse_peek("\"\\uD800\"").is_err());
        assert!(string::<Error<'_>>
            .parse_peek("\"\\uD800\\uD800\"")
            .is_err());
        assert!(string::<Error<'_>>.parse_peek("\"\\uDC00\"").is_err());
    }

    #[test]
    fn json_object() {
        use JsonValue::{Num, Object, Str};

        let input = r#"{"a":42,"b":"x"}"#;

        let expected = Object(
            vec![
                ("a".to_string(), Num(42.0)),
                ("b".to_string(), Str("x".to_string())),
            ]
            .into_iter()
            .collect(),
        );

        assert_eq!(json::<Error<'_>>.parse_peek(input), Ok(("", expected)));
    }

    #[test]
    fn json_array() {
        use JsonValue::{Array, Num, Str};

        let input = r#"[42,"x"]"#;

        let expected = Array(vec![Num(42.0), Str("x".to_string())]);

        assert_eq!(json::<Error<'_>>.parse_peek(input), Ok(("", expected)));
    }

    #[test]
    fn json_whitespace() {
        use JsonValue::{Array, Boolean, Null, Num, Object, Str};

        let input = r#"
  {
    "null" : null,
    "true"  :true ,
    "false":  false  ,
    "number" : 123e4 ,
    "string" : " abc 123 " ,
    "array" : [ false , 1 , "two" ] ,
    "object" : { "a" : 1.0 , "b" : "c" } ,
    "empty_array" : [  ] ,
    "empty_object" : {   }
  }
  "#;

        assert_eq!(
            json::<Error<'_>>.parse_peek(input),
            Ok((
                "",
                Object(
                    vec![
                        ("null".to_string(), Null),
                        ("true".to_string(), Boolean(true)),
                        ("false".to_string(), Boolean(false)),
                        ("number".to_string(), Num(123e4)),
                        ("string".to_string(), Str(" abc 123 ".to_string())),
                        (
                            "array".to_string(),
                            Array(vec![Boolean(false), Num(1.0), Str("two".to_string())])
                        ),
                        (
                            "object".to_string(),
                            Object(
                                vec![
                                    ("a".to_string(), Num(1.0)),
                                    ("b".to_string(), Str("c".to_string())),
                                ]
                                .into_iter()
                                .collect()
                            )
                        ),
                        ("empty_array".to_string(), Array(vec![]),),
                        ("empty_object".to_string(), Object(HashMap::new()),),
                    ]
                    .into_iter()
                    .collect()
                )
            ))
        );
    }
}
