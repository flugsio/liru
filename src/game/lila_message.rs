use serde_json;

use super::Clock;

pub enum LilaMessage {
    Pong(Pong),
    Move(Move),
    Clock(Clock),
}

#[derive(Deserialize, Debug)]
pub struct Pong {
    pub latency: i64,
}

#[allow(dead_code)]
#[derive(Deserialize)]
pub struct Move {
    pub clock: Option<Clock>,
    // "dests": Object({"a6": String("a5"), "c7": String("b8d8b6a5d6e5f4g3h2"), "d5": String("c6e6"), "f5": String("f4")}),
    //dests: String,
    pub fen: String,
    pub san: String, // Bc7
    pub uci: String, // e5c7
    pub ply: u64,
}


impl LilaMessage {
    pub fn decode(obj: &serde_json::Value) -> Option<LilaMessage> {
        let data = obj.get("d").and_then(|d| serde_json::to_string(d).ok());
        match (obj.get("t").and_then(|t| t.as_str()), data) {
            // TODO: gone, crowd, end, tvSelect, challenges, drop,
            // following_enters, following_leaves, following_onlines,
            // following_playing, following_stopped_plaing,
            // message, analysisProgress, reload, and more
            (Some("n"), Some(data)) => serde_json::from_str(&data).map(|d| LilaMessage::Pong(d)).ok(),
            (Some("move"), Some(data)) => serde_json::from_str(&data).map(|d| LilaMessage::Move(d)).ok(),
            (Some("clock"), Some(data)) => serde_json::from_str(&data).map(|d| LilaMessage::Clock(d)).ok(),
            (Some(ref t), ref d) => {
                warn!("unhandled: {}, {:?}", t, d);
                None
            },
            _ => {
                warn!("unhandled: Missing type");
                None
            }
        }
    }

}
