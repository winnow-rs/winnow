#[macro_use]
extern crate criterion;

use criterion::Criterion;
use winnow::{
  branch::alt,
  bytes::{any, none_of, one_of, tag, take},
  character::{f64, multispace0},
  error::ParseError,
  multi::{fold_many0, separated_list0},
  sequence::{delimited, preceded, separated_pair},
  IResult, Parser,
};

use std::collections::HashMap;

type Input<'i> = &'i str;

#[derive(Debug, PartialEq, Clone)]
pub enum JsonValue {
  Null,
  Bool(bool),
  Str(String),
  Num(f64),
  Array(Vec<JsonValue>),
  Object(HashMap<String, JsonValue>),
}

fn boolean(input: Input<'_>) -> IResult<Input<'_>, bool> {
  alt((tag("false").value(false), tag("true").value(true)))(input)
}

fn u16_hex(input: Input<'_>) -> IResult<Input<'_>, u16> {
  take(4usize)
    .map_res(|s| u16::from_str_radix(s, 16))
    .parse_next(input)
}

fn unicode_escape(input: Input<'_>) -> IResult<Input<'_>, char> {
  alt((
    // Not a surrogate
    u16_hex
      .verify(|cp| !(0xD800..0xE000).contains(cp))
      .map(|cp| cp as u32),
    // See https://en.wikipedia.org/wiki/UTF-16#Code_points_from_U+010000_to_U+10FFFF for details
    separated_pair(u16_hex, tag("\\u"), u16_hex)
      .verify(|(high, low)| (0xD800..0xDC00).contains(high) && (0xDC00..0xE000).contains(low))
      .map(|(high, low)| {
        let high_ten = (high as u32) - 0xD800;
        let low_ten = (low as u32) - 0xDC00;
        (high_ten << 10) + low_ten + 0x10000
      }),
  ))
  .map_opt(
    // Could probably be replaced with .unwrap() or _unchecked due to the verify checks
    std::char::from_u32,
  )
  .parse_next(input)
}

fn character(input: Input<'_>) -> IResult<Input<'_>, char> {
  let (input, c) = none_of("\"")(input)?;
  if c == '\\' {
    alt((
      any.map_res(|c| {
        Ok(match c {
          '"' | '\\' | '/' => c,
          'b' => '\x08',
          'f' => '\x0C',
          'n' => '\n',
          'r' => '\r',
          't' => '\t',
          _ => return Err(()),
        })
      }),
      preceded(one_of('u'), unicode_escape),
    ))(input)
  } else {
    Ok((input, c))
  }
}

fn string(input: Input<'_>) -> IResult<Input<'_>, String> {
  delimited(
    one_of('"'),
    fold_many0(character, String::new, |mut string, c| {
      string.push(c);
      string
    }),
    one_of('"'),
  )(input)
}

fn ws<'a, O, E: ParseError<Input<'a>>, F: Parser<Input<'a>, O, E>>(
  f: F,
) -> impl Parser<Input<'a>, O, E> {
  delimited(multispace0, f, multispace0)
}

fn array(input: Input<'_>) -> IResult<Input<'_>, Vec<JsonValue>> {
  delimited(
    one_of('['),
    ws(separated_list0(ws(one_of(',')), json_value)),
    one_of(']'),
  )(input)
}

fn object(input: Input<'_>) -> IResult<Input<'_>, HashMap<String, JsonValue>> {
  delimited(
    one_of('{'),
    ws(separated_list0(
      ws(one_of(',')),
      separated_pair(string, ws(one_of(':')), json_value),
    )),
    one_of('}'),
  )
  .map(|key_values| key_values.into_iter().collect())
  .parse_next(input)
}

fn json_value(input: Input<'_>) -> IResult<Input<'_>, JsonValue> {
  use JsonValue::{Array, Bool, Null, Num, Object, Str};

  alt((
    tag("null").value(Null),
    boolean.map(Bool),
    string.map(Str),
    f64.map(Num),
    array.map(Array),
    object.map(Object),
  ))(input)
}

fn json(input: Input<'_>) -> IResult<Input<'_>, JsonValue> {
  ws(json_value).parse_next(input)
}

fn json_bench(c: &mut Criterion) {
  let data = "  { \"a\"\t: 42,
  \"b\": [ \"x\", \"y\", 12 ,\"\\u2014\", \"\\uD83D\\uDE10\"] ,
  \"c\": { \"hello\" : \"world\"
  }
  }  ";

  // println!("data:\n{:?}", json(data));
  c.bench_function("json", |b| {
    b.iter(|| json(data).unwrap());
  });
}

criterion_group!(benches, json_bench,);
criterion_main!(benches);
