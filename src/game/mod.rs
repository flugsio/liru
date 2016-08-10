pub mod socket;

use std::collections::BTreeMap;

use std::ops::Not;
use std::sync::{Arc, Mutex};
use std::sync::mpsc;
use std::thread;

use rustc_serialize::json;
use rustc_serialize::Decodable;

use lila;

pub struct ConnectedPov {
    pub pov: Arc<Mutex<Pov>>,
}

#[derive(RustcDecodable)]
pub struct Pov {
    pub game: Game,
    pub clock: Option<Clock>,
    pub correspondence: Option<CorrespondenceClock>,
    pub url: GameUrl,
    pub player: Player,
    pub opponent: Player,
    pub tv: Option<Tv>,
    pub orientation: Option<Orientation>, 
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
    pub color: Orientation,
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

impl ConnectedPov {
    pub fn new(session: &lila::Session, path: &str) -> ConnectedPov {
        let mut body = String::new();
        session.get(path, &mut body);
        debug!("GET response: {}", body);
        let pov: Pov = json::decode(&body).unwrap();
        let version = match pov.player.version {
            Some(v) => v as u64,
            None => 0
        };
        let socket_path = pov.url.socket.clone();
        let pov_1 = Arc::new(Mutex::new(pov));
        let (game_tx, game_rx) = mpsc::channel();

        let pov_2 = pov_1.clone();
        session.connect(version, socket_path, game_tx);
        thread::spawn(move || loop {
            let obj = game_rx.recv().unwrap();
            let mut pov = pov_2.lock().unwrap();
            match LilaMessage::decode(&obj) {
                Some(LilaMessage::Move(m)) => {
                    pov.game.fen = m.fen;
                    if let Some(c) = m.clock {
                        pov.clock = Some(c);
                    };
                },
                Some(LilaMessage::Clock(c)) => {
                    pov.clock = Some(c);
                },
                //LilaMessage::End => tx_1.send(Message::close()).unwrap(),
                _ => ()
            };
        });

        ConnectedPov {
            pov: pov_1,
        }
    }
}

impl Pov {
    pub fn orientation(&self) -> Orientation {
        match self.orientation {
            Some(o) => o,
            None => {
                self.player.color
            }
        }
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

enum LilaMessage {
    Move(Move),
    Clock(Clock),
}

impl LilaMessage {
    fn decode(obj: &BTreeMap<String, json::Json>) -> Option<LilaMessage> {
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

#[derive(RustcDecodable)]
struct Move {
    clock: Option<Clock>,
    fen: String,
}

