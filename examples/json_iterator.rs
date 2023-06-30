use std::collections::HashMap;

use winnow::prelude::*;
use winnow::{
    ascii::{alphanumeric1 as alphanumeric, escaped, float},
    combinator::alt,
    combinator::cut_err,
    combinator::separated0,
    combinator::{preceded, separated_pair, terminated},
    error::ParseError,
    token::one_of,
    token::{tag, take_while},
    IResult,
};

use std::cell::Cell;
use std::str;

#[derive(Clone, Debug)]
pub struct JsonValue<'a, 'b> {
    input: &'a str,
    pub offset: &'b Cell<usize>,
}

impl<'a, 'b: 'a> JsonValue<'a, 'b> {
    pub fn new(input: &'a str, offset: &'b Cell<usize>) -> JsonValue<'a, 'b> {
        JsonValue { input, offset }
    }

    pub fn offset(&self, input: &'a str) {
        let offset = input.as_ptr() as usize - self.input.as_ptr() as usize;
        self.offset.set(offset);
    }

    pub fn data(&self) -> &'a str {
        &self.input[self.offset.get()..]
    }

    pub fn string(&self) -> Option<&'a str> {
        println!("string()");
        match string(self.data()) {
            Ok((i, s)) => {
                self.offset(i);
                println!("-> {}", s);
                Some(s)
            }
            _ => None,
        }
    }

    pub fn boolean(&self) -> Option<bool> {
        println!("boolean()");
        match boolean(self.data()) {
            Ok((i, o)) => {
                self.offset(i);
                println!("-> {}", o);
                Some(o)
            }
            _ => None,
        }
    }

    pub fn number(&self) -> Option<f64> {
        println!("number()");
        match float::<_, _, ()>(self.data()) {
            Ok((i, o)) => {
                self.offset(i);
                println!("-> {}", o);
                Some(o)
            }
            _ => None,
        }
    }

    pub fn array(&self) -> Option<impl Iterator<Item = JsonValue<'a, 'b>>> {
        println!("array()");

        match tag::<_, _, ()>("[").parse_peek(self.data()) {
            Err(_) => None,
            Ok((i, _)) => {
                println!("[");
                self.offset(i);
                let mut first = true;
                let mut done = false;
                let mut previous = std::usize::MAX;

                let v = self.clone();

                Some(std::iter::from_fn(move || {
                    if done {
                        return None;
                    }

                    // if we ignored one of the items, skip over the value
                    if v.offset.get() == previous {
                        println!("skipping value");
                        if let Ok((i, _)) = value(v.data()) {
                            v.offset(i);
                        }
                    }

                    if let Ok((i, _)) = tag::<_, _, ()>("]").parse_peek(v.data()) {
                        println!("]");
                        v.offset(i);
                        done = true;
                        return None;
                    }

                    if first {
                        first = false;
                    } else {
                        match tag::<_, _, ()>(",").parse_peek(v.data()) {
                            Ok((i, _)) => {
                                println!(",");
                                v.offset(i);
                            }
                            Err(_) => {
                                done = true;
                                return None;
                            }
                        }
                    }

                    println!("-> {}", v.data());
                    previous = v.offset.get();
                    Some(v.clone())
                }))
            }
        }
    }

    pub fn object(&self) -> Option<impl Iterator<Item = (&'a str, JsonValue<'a, 'b>)>> {
        println!("object()");
        match tag::<_, _, ()>("{").parse_peek(self.data()) {
            Err(_) => None,
            Ok((i, _)) => {
                self.offset(i);

                println!("{{");

                let mut first = true;
                let mut done = false;
                let mut previous = std::usize::MAX;

                let v = self.clone();

                Some(std::iter::from_fn(move || {
                    if done {
                        return None;
                    }

                    // if we ignored one of the items, skip over the value
                    if v.offset.get() == previous {
                        println!("skipping value");
                        if let Ok((i, _)) = value(v.data()) {
                            v.offset(i);
                        }
                    }

                    if let Ok((i, _)) = tag::<_, _, ()>("}").parse_peek(v.data()) {
                        println!("}}");
                        v.offset(i);
                        done = true;
                        return None;
                    }

                    if first {
                        first = false;
                    } else {
                        match tag::<_, _, ()>(",").parse_peek(v.data()) {
                            Ok((i, _)) => {
                                println!(",");
                                v.offset(i);
                            }
                            Err(_) => {
                                done = true;
                                return None;
                            }
                        }
                    }

                    match string(v.data()) {
                        Ok((i, key)) => {
                            v.offset(i);

                            match tag::<_, _, ()>(":").parse_peek(v.data()) {
                                Err(_) => None,
                                Ok((i, _)) => {
                                    v.offset(i);

                                    previous = v.offset.get();

                                    println!("-> {} => {}", key, v.data());
                                    Some((key, v.clone()))
                                }
                            }
                        }
                        _ => None,
                    }
                }))
            }
        }
    }
}

fn sp<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, &'a str, E> {
    let chars = " \t\r\n";

    take_while(0.., move |c| chars.contains(c)).parse_peek(i)
}

fn parse_str<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, &'a str, E> {
    escaped(alphanumeric, '\\', one_of(['"', 'n', '\\'])).parse_peek(i)
}

fn string(i: &str) -> IResult<&str, &str> {
    preceded('\"', cut_err(terminated(parse_str, '\"')))
        .context("string")
        .parse_peek(i)
}

fn boolean(input: &str) -> IResult<&str, bool> {
    alt(("false".map(|_| false), "true".map(|_| true))).parse_peek(input)
}

fn array(i: &str) -> IResult<&str, ()> {
    preceded(
        '[',
        cut_err(terminated(
            separated0(value, preceded(sp, ',')),
            preceded(sp, ']'),
        )),
    )
    .context("array")
    .parse_peek(i)
}

fn key_value(i: &str) -> IResult<&str, (&str, ())> {
    separated_pair(preceded(sp, string), cut_err(preceded(sp, ':')), value).parse_peek(i)
}

fn hash(i: &str) -> IResult<&str, ()> {
    preceded(
        '{',
        cut_err(terminated(
            separated0(key_value, preceded(sp, ',')),
            preceded(sp, '}'),
        )),
    )
    .context("map")
    .parse_peek(i)
}

fn value(i: &str) -> IResult<&str, ()> {
    preceded(
        sp,
        alt((
            hash,
            array,
            string.map(|_| ()),
            float::<_, f64, _>.map(|_| ()),
            boolean.map(|_| ()),
        )),
    )
    .parse_peek(i)
}

/// object(input) -> iterator over (key, `JsonValue`)
/// array(input) -> iterator over `JsonValue`
///
/// JsonValue.string -> iterator over String (returns None after first successful call)
///
/// object(input).filter(|(k, _)| k == "users").flatten(|(_, v)| v.object()).filter(|(k, _)| k == "city").flatten(|(_,v)| v.string())
fn main() {
    /*let data = "{
    \"users\": {
      \"user1\" : { \"city\": \"Nantes\", \"country\": \"France\" },
      \"user2\" : { \"city\": \"Bruxelles\", \"country\": \"Belgium\" },
      \"user3\": { \"city\": \"Paris\", \"country\": \"France\", \"age\": 30 }
    },
    \"countries\": [\"France\", \"Belgium\"]
    }";
    */
    let data = "{\"users\":{\"user1\":{\"city\":\"Nantes\",\"country\":\"France\"},\"user2\":{\"city\":\"Bruxelles\",\"country\":\"Belgium\"},\"user3\":{\"city\":\"Paris\",\"country\":\"France\",\"age\":30}},\"countries\":[\"France\",\"Belgium\"]}";

    let offset = Cell::new(0);
    {
        let parser = JsonValue::new(data, &offset);

        if let Some(o) = parser.object() {
            let s: HashMap<&str, &str> = o
                .filter(|(k, _)| *k == "users")
                .filter_map(|(_, v)| v.object())
                .flatten()
                .filter_map(|(user, v)| v.object().map(|o| (user, o)))
                .flat_map(|(user, o)| {
                    o.filter(|(k, _)| *k == "city")
                        .filter_map(move |(_, v)| v.string().map(|s| (user, s)))
                })
                .collect();

            println!("res = {:?}", s);
        }
    };
}
