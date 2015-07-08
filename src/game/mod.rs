pub mod socket;

#[derive(RustcDecodable)]
pub struct Pov {
    pub game: Game,
    pub clock: Option<Clock>,
    pub correspondence: Option<CorrespondenceClock>,
    pub url: GameUrl,
    pub player: Option<Player>,
    pub opponent: Option<Opponent>,
    pub tv: Option<Tv>,
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
    todo: Option<String>,
}

#[derive(RustcDecodable)]
pub struct GameUrl {
    pub socket: String,
    pub round: String,
}

#[derive(RustcDecodable)]
pub struct Game {
    pub id: String,
    pub variant: Variant,
    pub speed: String,
    pub perf: String,
    pub rated: bool,
    pub initialFen: String,
    pub fen: String,
    pub moves: String,
    pub player: String,
    pub turns: i64,
    pub startedAtTurn: i64,
    pub lastMove: Option<String>,
    pub threefold: bool,
    pub source: String,
    pub status: Status,
    pub white: Option<Player>,
    pub black: Option<Player>,
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
    pub version: i64,
    pub spectator: bool,
    pub user: Option<User>,
}

#[derive(RustcDecodable)]
pub struct Opponent {
    pub color: String,
    pub user: Option<User>,
}

#[derive(RustcDecodable)]
pub struct User {
    pub id: String,
    pub username: String,
}

