pub mod socket;

use std::collections::BTreeMap;

use std::ops::Not;
use std::sync::{Arc, Mutex};
use std::sync::mpsc;
use std::thread;

use rustc_serialize::json;
use rustc_serialize::Decoder;
use rustc_serialize::Decodable;
use uuid::Uuid;

use time;

use lila;
use lila::Session;

pub struct LatencyRecorder {
    pub last: i64,
    history: Vec<i64>,
}

impl LatencyRecorder {
    pub fn new() -> LatencyRecorder {
        LatencyRecorder {
            last: 0,
            history: vec!(),
        }
    }

    pub fn add(&mut self, latency: i64) {
        self.last = latency;
        self.history.push(latency);
    }
}


pub struct ConnectedPov {
    pub pov: Arc<Mutex<Pov>>,
    pub latency: Arc<Mutex<LatencyRecorder>>,
    send_tx: mpsc::Sender<String>,
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
    pub orientation: Option<Color>, 
}

#[allow(non_camel_case_types)]
#[derive(PartialEq, Eq, Copy, Clone)]
#[derive(RustcDecodable)]
pub enum Color {
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
    last_update: Time,
}

// TODO: ehm, maybe not like this
// wrap to implement trait on foreign type in this version of rust
pub struct Time(time::Tm);
impl Decodable for Time {
    fn decode<D: Decoder>(_d: &mut D) -> Result<Time, D::Error> {
        Ok(Time(time::now_utc()))
    }
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
    pub player: Color,
    pub turns: u64,
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
        let (send_tx, send_rx) = mpsc::channel();
        let send_rx = Arc::new(Mutex::new(send_rx));

        let pov_2 = pov_1.clone();
        let c = session.cookie();
        let sri = Uuid::new_v4();
        debug!("SRI set to {}", sri);
        let url = Session::socket_url(&format!("{}?sri={}&version={}", socket_path, sri, version));
        thread::spawn(move || {
            socket::Client::connect(&c, url.clone(), version, game_tx.clone(), send_rx);
        });

        let latency_1 = Arc::new(Mutex::new(LatencyRecorder::new()));
        let latency_2 = latency_1.clone();
        thread::spawn(move || loop {
            let obj = game_rx.recv().unwrap();
            let mut pov = pov_2.lock().unwrap();
            match LilaMessage::decode(&obj) {
                Some(LilaMessage::Pong(p)) => {
                    latency_2.lock().unwrap().add(p.latency);
                },
                Some(LilaMessage::Move(m)) => {
                    pov.game.fen = m.fen;
                    pov.game.turns = m.ply;
                    pov.game.player = if m.ply % 2 == 0 { Color::white } else { Color::black };
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
            latency: latency_1,
            send_tx: send_tx,
        }
    }

    pub fn send_move(&mut self, from: String, to: String) {
        let move_packet = MovePacket {
            t: "move".to_string(),
            l: None, // TODO
            d: Dest {
                from: from,
                to: to,
                promotion: None,
            },
        };
        let message = json::encode(&move_packet).unwrap();
        self.send_tx.send(message).unwrap();
    }
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

impl Not for Color {
    type Output = Color;
    fn not(self) -> Color {
        if self == Color::white {
            Color::black
        } else {
            Color::white
        }
    }
}

impl Clock {
    pub fn from(&self, color: Color) -> f64 {
        match color {
            Color::white => self.white,
            Color::black => self.black,
        }
    }
    
    pub fn tick(&mut self, color: Color) {
        let now = time::now_utc();
        let Time(updated) = self.last_update;
        let passed = ((now - updated).num_milliseconds() as f64) / 1000.0;
        self.last_update = Time(now);
        match color {
            // TODO: only tick when gameIsRunning instead of using max
            Color::white => self.white = (self.white - passed).max(0.0),
            Color::black => self.black = (self.black - passed).max(0.0),
        };
    }
}

enum LilaMessage {
    Pong(Pong),
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

#[derive(RustcDecodable)]
struct Pong {
    latency: i64,
}

#[derive(RustcDecodable)]
struct Move {
    clock: Option<Clock>,
    fen: String,
    ply: u64,
}

#[derive(RustcEncodable)]
pub struct MovePacket {
    t: String,
    d: Dest,
    l: Option<i64>,
}

#[derive(RustcEncodable)]
pub struct Dest {
    pub from: String,
    pub to: String,
    pub promotion: Option<String>,
}

