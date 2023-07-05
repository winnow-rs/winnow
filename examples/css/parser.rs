use winnow::prelude::*;
use winnow::token::take_while;

#[derive(Debug, Eq, PartialEq)]
pub struct Color {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

impl std::str::FromStr for Color {
    // The error must be owned
    type Err = winnow::error::ErrorKind;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        hex_color.parse(s)
    }
}

pub fn hex_color(input: &mut &str) -> PResult<Color> {
    let _ = "#".parse_next(input)?;
    let (red, green, blue) = (hex_primary, hex_primary, hex_primary).parse_next(input)?;

    Ok(Color { red, green, blue })
}

fn hex_primary(input: &mut &str) -> PResult<u8> {
    take_while(2, |c: char| c.is_ascii_hexdigit())
        .try_map(|input| u8::from_str_radix(input, 16))
        .parse_next(input)
}
