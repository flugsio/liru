use serde_derive::Deserialize;

mod socket;
mod latency_recorder;
mod clock;
mod connected_pov;
mod color;
mod lila_message;
mod pov;

pub use crate::game::latency_recorder::LatencyRecorder;
pub use crate::game::connected_pov::ConnectedPov;
pub use crate::game::color::Color;
pub use crate::game::pov::{Pov,Player};
use crate::game::clock::Clock;

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

