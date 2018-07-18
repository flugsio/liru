mod socket;
mod latency_recorder;
mod clock;
mod connected_pov;
mod color;
mod lila_message;
mod pov;

pub use game::latency_recorder::LatencyRecorder;
pub use game::connected_pov::ConnectedPov;
pub use game::color::Color;
pub use game::pov::{Pov,Player};
use game::clock::Clock;

#[allow(non_snake_case)]
#[derive(Deserialize)]
pub struct Game {
    pub id: String,
    pub variant: Variant,
    pub speed: String,
    pub perf: String,
    pub rated: bool,
    pub initialFen: String,
    pub fen: String,
    pub player: Color,
    pub turns: u64,
    pub startedAtTurn: i64,
    pub lastMove: Option<String>,
    pub threefold: Option<bool>,
    pub source: String,
    pub status: Status,
}

#[derive(Deserialize)]
pub struct Variant {
    pub key: String,
    pub name: String,
    pub short: String,
}

#[derive(Deserialize)]
pub struct Status {
    pub id: i64,
    pub name: String,
}

