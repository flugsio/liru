use serde_derive::Deserialize;

use super::Color;

#[derive(Deserialize, Debug)]
pub struct Crowd {
    pub white: bool,
    pub black: bool,
    pub watchers: Watchers,
}

#[derive(Deserialize, Debug)]
pub struct Watchers {
    pub nb: i64,
    pub users: Option<Vec<String>>,
    pub anons: Option<i64>,
}

impl Crowd {
    pub fn opponent_from(&self, color: Color) -> bool {
        match color {
            Color::white => self.black,
            Color::black => self.white,
        }
    }

    pub fn player_from(&self, color: Color) -> bool {
        match color {
            Color::white => self.white,
            Color::black => self.black,
        }
    }
}
