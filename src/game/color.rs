use std::ops::Not;

#[allow(non_camel_case_types)]
#[derive(PartialEq, Eq, Copy, Clone)]
#[derive(Deserialize, Debug)]
pub enum Color {
    white,
    black,
}

impl Not for Color {
    type Output = Color;
    fn not(self) -> Color {
        match self {
            Color::white => Color::black,
            Color::black => Color::white,
        }
    }
}
