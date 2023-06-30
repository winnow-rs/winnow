use winnow::{ascii::line_ending, combinator::repeat, token::take_while, IResult, Parser};

pub type Stream<'i> = &'i [u8];

#[rustfmt::skip]
#[derive(Debug)]
#[allow(dead_code)]
pub struct Request<'a> {
  method:  &'a [u8],
  uri:     &'a [u8],
  version: &'a [u8],
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct Header<'a> {
    name: &'a [u8],
    value: Vec<&'a [u8]>,
}

pub fn parse(data: &[u8]) -> Option<Vec<(Request<'_>, Vec<Header<'_>>)>> {
    let mut buf = data;
    let mut v = Vec::new();
    loop {
        match request(buf) {
            Ok((b, r)) => {
                buf = b;
                v.push(r);

                if b.is_empty() {
                    //println!("{}", i);
                    break;
                }
            }
            Err(e) => {
                println!("error: {:?}", e);
                return None;
            }
        }
    }

    Some(v)
}

fn request(input: Stream<'_>) -> IResult<Stream<'_>, (Request<'_>, Vec<Header<'_>>)> {
    let (input, req) = request_line(input)?;
    let (input, h) = repeat(1.., message_header).parse_peek(input)?;
    let (input, _) = line_ending.parse_peek(input)?;

    Ok((input, (req, h)))
}

fn request_line(input: Stream<'_>) -> IResult<Stream<'_>, Request<'_>> {
    let (input, method) = take_while(1.., is_token).parse_peek(input)?;
    let (input, _) = take_while(1.., is_space).parse_peek(input)?;
    let (input, uri) = take_while(1.., is_not_space).parse_peek(input)?;
    let (input, _) = take_while(1.., is_space).parse_peek(input)?;
    let (input, version) = http_version(input)?;
    let (input, _) = line_ending.parse_peek(input)?;

    Ok((
        input,
        Request {
            method,
            uri,
            version,
        },
    ))
}

fn http_version(input: Stream<'_>) -> IResult<Stream<'_>, &[u8]> {
    let (input, _) = "HTTP/".parse_peek(input)?;
    let (input, version) = take_while(1.., is_version).parse_peek(input)?;

    Ok((input, version))
}

fn message_header_value(input: Stream<'_>) -> IResult<Stream<'_>, &[u8]> {
    let (input, _) = take_while(1.., is_horizontal_space).parse_peek(input)?;
    let (input, data) = take_while(1.., not_line_ending).parse_peek(input)?;
    let (input, _) = line_ending.parse_peek(input)?;

    Ok((input, data))
}

fn message_header(input: Stream<'_>) -> IResult<Stream<'_>, Header<'_>> {
    let (input, name) = take_while(1.., is_token).parse_peek(input)?;
    let (input, _) = ':'.parse_peek(input)?;
    let (input, value) = repeat(1.., message_header_value).parse_peek(input)?;

    Ok((input, Header { name, value }))
}

#[rustfmt::skip]
#[allow(clippy::match_same_arms)]
#[allow(clippy::match_like_matches_macro)]
fn is_token(c: u8) -> bool {
  match c {
    128..=255 => false,
    0..=31    => false,
    b'('      => false,
    b')'      => false,
    b'<'      => false,
    b'>'      => false,
    b'@'      => false,
    b','      => false,
    b';'      => false,
    b':'      => false,
    b'\\'     => false,
    b'"'      => false,
    b'/'      => false,
    b'['      => false,
    b']'      => false,
    b'?'      => false,
    b'='      => false,
    b'{'      => false,
    b'}'      => false,
    b' '      => false,
    _         => true,
  }
}

fn is_version(c: u8) -> bool {
    (b'0'..=b'9').contains(&c) || c == b'.'
}

fn not_line_ending(c: u8) -> bool {
    c != b'\r' && c != b'\n'
}

fn is_space(c: u8) -> bool {
    c == b' '
}

fn is_not_space(c: u8) -> bool {
    c != b' '
}

fn is_horizontal_space(c: u8) -> bool {
    c == b' ' || c == b'\t'
}
