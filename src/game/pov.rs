
use super::Clock;
use super::Color;
use super::Game;

#[derive(RustcDecodable)]
pub struct Pov {
    pub game: Game,
    pub clock: Option<Clock>,
    pub correspondence: Option<CorrespondenceClock>,
    pub url: GameUrl,
    pub player: Player,
    pub opponent: Player,
    pub tv: Option<Tv>,
    pub orientation: Option<Color>, 
}

#[derive(RustcDecodable)]
pub struct Tv {
    pub channel: String,
    pub flip: bool,
}

#[derive(RustcDecodable)]
pub struct Player {
    pub color: Color,
    pub version: Option<i64>,
    pub spectator: Option<bool>,
    pub user: Option<User>,
    pub rating: Option<i64>,
}

#[derive(RustcDecodable)]
pub struct User {
    pub id: String,
    pub username: String,
}

#[derive(RustcDecodable)]
pub struct GameUrl {
    pub socket: String,
    pub round: String,
}

#[derive(RustcDecodable)]
pub struct CorrespondenceClock {
    _todo: Option<String>,
}

impl Pov {
    pub fn orientation(&self) -> Color {
        match self.orientation {
            Some(o) => o,
            None => {
                self.player.color
            }
        }
    }

    pub fn tick(&mut self) {
        // FUTURE: `let` is only needed bc rust borrow checker is lazy
        let color = self.game.player;
        self.clock.as_mut().map(|c| c.tick(color));
    }
}

