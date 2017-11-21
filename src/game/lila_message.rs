use rustc_serialize::json;
use rustc_serialize::Decodable;

use super::Clock;

pub enum LilaMessage {
    Pong(Pong),
    Move(Move),
    Clock(Clock),
}

#[derive(RustcDecodable)]
pub struct Pong {
    pub latency: i64,
}

#[allow(dead_code)]
#[derive(RustcDecodable)]
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
    pub fn decode(obj: &json::Object) -> Option<LilaMessage> {
        fn decode<T: Decodable>(data: &json::Json) -> Option<T> {
            let mut decoder = json::Decoder::new(data.to_owned());
            Decodable::decode(&mut decoder)
                .map_err(|e| error!("could not decode: {}", e)).ok()
        }
        let data = obj.get("d");
        match (obj.get("t").and_then(|t| t.as_string()), data) {
            // TODO: gone, crowd, end, tvSelect, challenges, drop,
            // following_enters, following_leaves, following_onlines,
            // following_playing, following_stopped_plaing,
            // message, analysisProgress, reload, and more
            (Some("n"), Some(data)) => decode(data).map(|d| LilaMessage::Pong(d)),
            (Some("move"), Some(data)) => decode(data).map(|d| LilaMessage::Move(d)),
            (Some("clock"), Some(data)) => decode(data).map(|d| LilaMessage::Clock(d)),
            (Some(ref t), d) => {
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
