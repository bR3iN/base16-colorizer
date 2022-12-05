use serde::Deserialize;
use nom::sequence::tuple;
use nom::{Finish, Parser};

use crate::utils::hex_byte;

#[derive(Debug, PartialEq, Deserialize, Clone, Copy)]
#[serde(try_from = "String")]
pub struct Color {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

impl Color {
    pub fn lighten(self, percentage: u8) -> Color {
        self.scale(1.0 + percentage as f64 / 100.0)
    }

    pub fn darken(self, percentage: u8) -> Color {
        self.scale(1.0 - percentage as f64 / 100.0)
    }

    fn scale(self, factor: f64) -> Color {
        let scale_primary = |prim| (prim as f64 * factor).clamp(0.0, 255.0) as u8;

        Color {
            red: scale_primary(self.red),
            green: scale_primary(self.green),
            blue: scale_primary(self.blue),
        }
    }
}

impl TryFrom<String> for Color {
    type Error = String;

    fn try_from(input: String) -> Result<Color, Self::Error> {
        hex_color
            .parse(&input)
            .finish()
            .map(|(_, res)| res)
            .map_err(|_| format!("Invalid hex color \"{}\"", input))
    }
}

impl std::fmt::Display for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        write!(f, "{:02X}{:02X}{:02X}", self.red, self.green, self.blue)
    }
}

fn hex_color(input: &str) -> nom::IResult<&str, Color> {
    tuple((hex_byte, hex_byte, hex_byte))
        .map(|(red, green, blue)| Color { red, green, blue })
        .parse(input)
}
