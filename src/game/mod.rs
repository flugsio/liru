pub mod socket;

use std::ops::Not;

use rustc_serialize::json;

use lila;

#[derive(RustcDecodable)]
pub struct Pov {
    pub game: Game,
    pub clock: Option<Clock>,
    pub correspondence: Option<CorrespondenceClock>,
    pub url: GameUrl,
    pub player: Player,
    pub opponent: Player,
    pub tv: Option<Tv>,
    pub orientation: Orientation, 
}

#[allow(non_camel_case_types)]
#[derive(PartialEq, Eq, Copy, Clone)]
#[derive(RustcDecodable)]
pub enum Orientation {
    white,
    black,
}

#[derive(RustcDecodable)]
pub struct Tv {
    pub channel: String,
    pub flip: bool,
}

#[derive(RustcDecodable)]
pub struct Clock {
    pub white: f64,
    pub black: f64,
}

#[derive(RustcDecodable)]
pub struct CorrespondenceClock {
    _todo: Option<String>,
}

#[derive(RustcDecodable)]
pub struct GameUrl {
    pub socket: String,
    pub round: String,
}

#[allow(non_snake_case)]
#[derive(RustcDecodable)]
pub struct Game {
    pub id: String,
    pub variant: Variant,
    pub speed: String,
    pub perf: String,
    pub rated: bool,
    pub initialFen: String,
    pub fen: String,
    pub player: String,
    pub turns: i64,
    pub startedAtTurn: i64,
    pub lastMove: Option<String>,
    pub threefold: bool,
    pub source: String,
    pub status: Status,
}

#[derive(RustcDecodable)]
pub struct Variant {
    pub key: String,
    pub name: String,
    pub short: String,
    pub title: String,
}

#[derive(RustcDecodable)]
pub struct Status {
    pub id: i64,
    pub name: String,
}

#[derive(RustcDecodable)]
pub struct Player {
    pub color: String,
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

impl Pov {
    pub fn new(session: &lila::Session, base_url: String, game_id: String) -> Option<Pov> {
        let mut body = String::new();
        let url = format!("https://{}/{}", base_url, game_id);
        session.get(url, &mut body);
        debug!("GET response: {}", body);
        json::decode(&body).unwrap()
    }
}

impl Not for Orientation {
    type Output = Orientation;
    fn not(self) -> Orientation {
        if self == Orientation::white {
            Orientation::black
        } else {
            Orientation::white
        }
    }
}

impl Clock {
    pub fn from(&self, orientation: Orientation) -> f64 {
        if orientation == Orientation::white {
            self.white
        } else {
            self.black
        }
    }
}

