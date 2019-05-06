use std::sync::{Arc, Mutex};
use std::sync::mpsc;
use std::thread;

use serde_json;
use uuid::Uuid;

use crate::lila;
use crate::lila::Session;

use super::LatencyRecorder;
use super::Pov;
use super::Color;

use crate::game::socket;
use crate::game::lila_message::LilaMessage;

pub struct ConnectedPov {
    pub pov: Arc<Mutex<Pov>>,
    pub latency: Arc<Mutex<LatencyRecorder>>,
    send_tx: mpsc::Sender<String>,
}

impl ConnectedPov {
    pub fn new(session: &lila::Session, path: &str) -> ConnectedPov {
        let body = session.get(path);
        debug!("GET response: {}", body);
        let pov: Pov = serde_json::from_str(&body).unwrap();
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
        let c = session.cookie.clone();
        let sri = Uuid::new_v4();
        debug!("SRI set to {}", sri);
        //let new_path = str::replace(path, "/v1", "/v2");
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
            t: "move".into(),
            l: None, // TODO
            d: Dest {
                from: from,
                to: to,
                promotion: None,
            },
        };
        let message = serde_json::to_string(&move_packet).unwrap();
        self.send_tx.send(message).unwrap();
    }
}

#[derive(Serialize, Debug)]
pub struct MovePacket {
    t: String,
    d: Dest,
    l: Option<i64>,
}

#[derive(Serialize, Debug)]
pub struct Dest {
    pub from: String,
    pub to: String,
    pub promotion: Option<String>,
}

