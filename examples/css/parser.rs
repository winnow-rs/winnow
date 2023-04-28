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
    type Err = winnow::error::Error<String>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        hex_color.parse(s).map_err(winnow::error::Error::into_owned)
    }
}

pub fn hex_color(input: &str) -> IResult<&str, Color> {
    let (input, _) = "#".parse_next(input)?;
    let (input, (red, green, blue)) = (hex_primary, hex_primary, hex_primary).parse_next(input)?;

    Ok((input, Color { red, green, blue }))
}

fn hex_primary(input: &str) -> IResult<&str, u8> {
    take_while(2, |c: char| c.is_ascii_hexdigit())
        .try_map(|input| u8::from_str_radix(input, 16))
        .parse_next(input)
}
