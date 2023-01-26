use winnow::bytes::{tag, take_while_m_n};
use winnow::prelude::*;

#[derive(Debug, Eq, PartialEq)]
pub struct Color {
  pub red: u8,
  pub green: u8,
  pub blue: u8,
}

fn from_hex(input: &str) -> Result<u8, std::num::ParseIntError> {
  u8::from_str_radix(input, 16)
}

fn is_hex_digit(c: char) -> bool {
  c.is_ascii_hexdigit()
}

fn hex_primary(input: &str) -> IResult<&str, u8> {
  take_while_m_n(2, 2, is_hex_digit)
    .map_res(from_hex)
    .parse_next(input)
}

fn hex_color(input: &str) -> IResult<&str, Color> {
  let (input, _) = tag("#")(input)?;
  let (input, (red, green, blue)) = (hex_primary, hex_primary, hex_primary).parse_next(input)?;

  Ok((input, Color { red, green, blue }))
}

fn main() {}

#[test]
fn parse_color() {
  assert_eq!(
    hex_color("#2F14DF"),
    Ok((
      "",
      Color {
        red: 47,
        green: 20,
        blue: 223,
      }
    ))
  );
}
