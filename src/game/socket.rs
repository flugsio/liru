extern crate websocket;

use std;
use std::sync::{Arc, Mutex};
use std::sync::mpsc;
use std::thread;

use websocket::{Message, Sender, Receiver};
use websocket::message::Type;
use websocket::client::request::Url;

use hyper::header::{Cookie, CookieJar};

use rustc_serialize::json;
use rustc_serialize::json::Json;

use time;
use lila;

// making a move
// out {"t":"move","d":{"from":"e2","to":"e4","b":1}}
// in {t: "ack"}
// in {"v":1,"t":"move","d":{"uci":"e2e4","san":"e4","fen":"rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR","ply":1,"clock":{"white":172800,"black":172800},"dests":{"b8":"a6c6","g8":"f6h6","h7":"h6h5","d7":"d6d5","g7":"g6g5","c7":"c6c5","f7":"f6f5","b7":"b6b5","e7":"e6e5","a7":"a6a5"},"crazyhouse":{"pockets":[{},{}]}}}
// other player moved
// in {"v":2,"t":"move","d":{"uci":"g7g5","san":"g5","fen":"rnbqkbnr/pppppp1p/8/6p1/4P3/8/PPPP1PPP/RNBQKBNR","ply":2,"clock":{"white":172800,"black":172800},"dests":{"a2":"a3a4","g1":"f3h3e2","d1":"e2f3g4h5","e1":"e2","d2":"d3d4","b1":"a3c3","e4":"e5","f1":"e2d3c4b5a6","h2":"h3h4","b2":"b3b4","f2":"f3f4","c2":"c3c4","g2":"g3g4"},"crazyhouse":{"pockets":[{},{}]}}}


#[derive(RustcEncodable)]
struct PingPacket {
    t: String,
    v: i64
}

impl PingPacket {
    pub fn new(version: i64) -> PingPacket {
        PingPacket {
            t: "p".to_string(),
            v: version
        }
    }
    
    pub fn to_message(&self) -> Message<'static> {
        Message::text(json::encode(self).unwrap())
    }
}

#[derive(RustcEncodable)]
pub struct MovePacket {
    t: String, // this should be hardcoded
    d: Dest,
}

#[derive(RustcEncodable)]
pub struct Dest {
    pub from: String,
    pub to: String,
    // pub: i8, // bool, but 0 or 1
}

pub fn connect(session: &lila::Session, base_url: String, sri: String, pov: Arc<Mutex<super::Pov>>) {

    let mut url;
    {
        let pov = pov.lock().unwrap();
        let version = match pov.player.version {
            Some(v) => v,
            None => 0
        };
        url = Url::parse(&format!("ws://{}{}?sri={}&version={}", base_url, pov.url.socket.clone(), sri, version)).unwrap();
    }
    let version = Arc::new(Mutex::new(0));

    // TODO: this unwrap fails when url is wrong, port for example
    let mut request = websocket::Client::connect(url).unwrap();
    request.headers.set(Cookie::from_cookie_jar(&session.cjar));

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
                    //println!("Send Loop: {:?}", e);
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
                    //println!("Send Loop: {:?}", e);
                    let _ = sender.send_message(&Message::close());
                    return;
                }
            }
        }
    });

    let pov_1 = pov.clone();
    let _receive_loop = thread::spawn(move || {

        let handle = |obj: json::Object| {
            let mut pov = pov_1.lock().unwrap();
            match obj.get("v") {
                Some(&Json::U64(v)) => {
                    pov.player.version = Some(v as i64);
                },
                _ => ()
            }
            match obj.get("t") {
                Some(&Json::String(ref t)) if t == "move" => {
                    let d = obj.get("d").unwrap().as_object().unwrap();
                    let fen = d.get("fen").unwrap().as_string().unwrap();
                    pov.game.fen = fen.to_string();
                    //cli::render_fen(fen);
                },
                Some(&Json::String(ref t)) if t == "clock" => {
                    let d = obj.get("d").unwrap().as_object().unwrap();
                    match pov.clock.as_mut() {
                        Some(clock) => {
                            clock.white = d.get("white").unwrap().as_f64().unwrap();
                            clock.black = d.get("black").unwrap().as_f64().unwrap();
                        },
                        None => ()
                    };
                },
                Some(&Json::String(ref t)) if t == "end" => {
                    let _ = tx_1.send(Message::close());
                    // exit
                },
                //Some(&Json::String(ref t)) => println!("unhandled: {:?}", obj),
                _ => ()
            }
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
                        let t = obj.get("t");
                        match t {
                            Some(&Json::String(ref t)) if t == "n" => { // pong
                            },
                            Some(&Json::String(ref t)) if t == "b" => { // batch
                                for item in obj.get("d").unwrap().as_array().unwrap().iter() {
                                   let obj = item.as_object().unwrap();
                                   handle(obj.clone());
                                }
                            }
                            _ => {
                                handle(obj.clone());
                            }
                        }
                    }
                }
                _ => debug!("Unhandled message: {:?}", message),
            }
        }
    });

    let tx_2 = tx.clone();

    thread::spawn(move || loop {
        std::thread::sleep_ms(1000);

        pov.lock().ok()
            .and_then(|p| p.player.version )
            .map(|v| PingPacket::new(v).to_message())
            .map(|p| tx_2.send(p).unwrap());
    });

    //let _ = send_loop.join();
    //let _ = receive_loop.join();
}

