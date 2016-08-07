extern crate websocket;

use std;
use std::sync::{Arc, Mutex};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;
use std::collections::BTreeMap;

use websocket::{Message, Sender, Receiver};
use websocket::message::Type;
use websocket::client::request::Url;

use hyper::header::{Cookie, CookieJar};

use time;

use rustc_serialize::json;
use rustc_serialize::json::Json;
use rustc_serialize::Decodable;

use game::Clock;

// making a move
// out {"t":"move","d":{"from":"e2","to":"e4","b":1}}
// in {t: "ack"}
// in {"v":1,"t":"move","d":{"uci":"e2e4","san":"e4","fen":"rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR","ply":1,"clock":{"white":172800,"black":172800},"dests":{"b8":"a6c6","g8":"f6h6","h7":"h6h5","d7":"d6d5","g7":"g6g5","c7":"c6c5","f7":"f6f5","b7":"b6b5","e7":"e6e5","a7":"a6a5"},"crazyhouse":{"pockets":[{},{}]}}}
// other player moved
// in {"v":2,"t":"move","d":{"uci":"g7g5","san":"g5","fen":"rnbqkbnr/pppppp1p/8/6p1/4P3/8/PPPP1PPP/RNBQKBNR","ply":2,"clock":{"white":172800,"black":172800},"dests":{"a2":"a3a4","g1":"f3h3e2","d1":"e2f3g4h5","e1":"e2","d2":"d3d4","b1":"a3c3","e4":"e5","f1":"e2d3c4b5a6","h2":"h3h4","b2":"b3b4","f2":"f3f4","c2":"c3c4","g2":"g3g4"},"crazyhouse":{"pockets":[{},{}]}}}

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


#[derive(RustcEncodable)]
struct PingPacket {
    t: String,
    v: u64
}

impl PingPacket {
    pub fn new(version: u64) -> PingPacket {
        PingPacket {
            t: "p".to_string(),
            v: version
        }
    }
    
    pub fn to_message(&self) -> Message<'static> {
        Message::text(json::encode(self).unwrap())
    }
}

#[allow(dead_code)]
#[derive(RustcEncodable)]
pub struct MovePacket {
    t: String,
    d: Dest,
}

#[allow(dead_code)]
#[derive(RustcEncodable)]
pub struct Dest {
    pub from: String,
    pub to: String,
    // pub: i8, // bool, but 0 or 1
}

pub fn connect(cjar: &CookieJar, url: String, version: u64, pov: Arc<Mutex<super::Pov>>) {
    // wraps version for usage in receive and ping threads
    let version = Arc::new(Mutex::new(version));
    let url = Url::parse(&url).unwrap();
    // TODO: this unwrap fails when url is wrong, port for example
    let mut request = websocket::Client::connect(url).unwrap();
    request.headers.set(Cookie::from_cookie_jar(&cjar));

    let response = request.send().unwrap();

    match response.validate() {
        Ok(()) => (),
        Err(e) => {
            error!("reponse validation error: {:?}", e);
            debug!("reponse status: {:?}, headers: {:?}", response.status, response.headers);
            return;
        }
    };

    //println!("Successfully connected");

    let (mut sender, mut receiver) = response.begin().split();

    let (tx, rx) = mpsc::channel();

    let tx_1 = tx.clone();

    let _send_loop = thread::spawn(move || {
        loop {
            let message: Message = match rx.recv() {
                Ok(m) => m,
                Err(e) => {
                    error!("Send loop, could not read next message: {:?}", e);
                    return;
                }
            };
            trace!("Transmit raw: {:?}", message);
            debug!("Transmit msg: {}",
                   &std::str::from_utf8(&message.payload).unwrap());
            match message.opcode {
                Type::Close => {
                    let _ = sender.send_message(&message);
                    return;
                }
                _ => (),
            }
            match sender.send_message(&message) {
                Ok(()) => (),
                Err(e) => {
                    error!("Send loop: could not send message: {:?}", e);
                    let _ = sender.send_message(&Message::close());
                    return;
                }
            }
        }
    });

    let (pong_tx, pong_rx) = mpsc::channel();
    let pov_1 = pov.clone();
    let version_1 = version.clone();
    let _receive_loop = thread::spawn(move || {

        let handle = |obj: &BTreeMap<String, json::Json>| {
            if let (Some(v), Ok(mut version)) = (obj.get("v").and_then(|v| v.as_u64()), version_1.lock()) {
                if v <= *version {
                    debug!("Already has event {}", v);
                    return;
                } else if *version + 1 < v {
                    debug!("Event gap detected from {} to {}", *version, v);
                    return;
                }
                *version = v;
            }
            let mut pov = pov_1.lock().unwrap();
            match LilaMessage::decode(obj) {
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
        };

        for message in receiver.incoming_messages() {
            let message: Message = match message {
                Ok(m) => m,
                Err(e) => {
                    warn!("Receive loop error: {:?}", e);
                    let _ = tx_1.send(Message::close());
                    return;
                }
            };
            match message.opcode {
                Type::Close => {
                    let _ = tx_1.send(Message::close());
                    return;
                }
                Type::Ping => match tx_1.send(Message::pong(message.payload)) {
                    Ok(()) => (),
                    Err(e) => {
                        warn!("Could not send pong: {:?}", e);
                        return;
                    }
                },
                Type::Text => {
                    let str_payload = std::str::from_utf8(&message.payload).unwrap();
                    trace!("Received raw: {:?}", &message.payload);
                    trace!("Received str: {}", str_payload);
                    let json = Json::from_str(str_payload).unwrap();
                    debug!("Received obj: {:?}", json);
                    if json.is_object() {
                        let obj = json.as_object().unwrap();
                        match obj.get("t").and_then(|t| t.as_string()) {
                            Some("n") => pong_tx.send(()).unwrap(),
                            Some("b") => { // batch
                                let data = obj.get("d").unwrap();
                                for item in data.as_array().unwrap().iter() {
                                    let obj = item.as_object().unwrap();
                                    handle(obj);
                                }
                            },
                            _ => handle(obj)
                        }
                    }
                }
                _ => debug!("Unhandled message: {:?}", message),
            }
        }
    });

    let tx_2 = tx.clone();
    let version_2 = version.clone();
    thread::spawn(move || loop {
        thread::sleep(Duration::from_millis(2000));

        let ping;
        { ping = PingPacket::new(*version_2.lock().unwrap()).to_message(); }
        let before = time::now_utc();
        tx_2.send(ping).unwrap(); // TODO: stop pinging when socket is closed
        pong_rx.recv().unwrap();
        let after = time::now_utc();
        debug!("Ping time: {}", after-before);
    });

    //let _ = send_loop.join();
    //let _ = receive_loop.join();
}

