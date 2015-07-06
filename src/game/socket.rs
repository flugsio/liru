extern crate websocket;

use std;
use std::sync::{Arc, Mutex};
use std::thread;
use std::sync::mpsc::channel;

use websocket::{Message, Sender, Receiver};
use websocket::client::request::Url;

use rustc_serialize::json;
use rustc_serialize::json::Json;


#[derive(RustcEncodable)]
pub struct PingPacket {
    t: String,
    v: i64
}

pub fn connect(base_url: String, sri: String, pov: Arc<Mutex<super::Pov>>) {

    let mut url;
    {
        let pov = pov.lock().unwrap();
        let version = match pov.player {
            Some(ref player) => player.version,
            None => 0
        };
        url = Url::parse(&format!("ws://{}{}?sri={}&version={}", base_url, pov.url.socket.clone(), sri, version)).unwrap();
    }
    let version = Arc::new(Mutex::new(0));

    println!("Connecting to {}", url);

    // TODO: this unwrap fails when url is wrong, port for example
    let request = websocket::Client::connect(url).unwrap();

    let response = request.send().unwrap();

    match response.validate() {
        Ok(()) => (),
        Err(e) => {
            println!("reponse validation error: {:?}", e);
            println!("reponse status: {:?}, headers: {:?}", response.status, response.headers);
            return;
        }
    };

    println!("Successfully connected");

    let (mut sender, mut receiver) = response.begin().split();

    let (tx, rx) = channel();

    let tx_1 = tx.clone();

    let send_loop = thread::spawn(move || {
        loop {
            let message = match rx.recv() {
                Ok(m) => m,
                Err(e) => {
                    //println!("Send Loop: {:?}", e);
                    return;
                }
            };
            match message {
                Message::Close(_) => {
                    let _ = sender.send_message(message);
                    return;
                }
                _ => (),
            }
            match sender.send_message(message) {
                Ok(()) => (),
                Err(e) => {
                    //println!("Send Loop: {:?}", e);
                    let _ = sender.send_message(Message::Close(None));
                    return;
                }
            }
        }
    });

    let pov_1 = pov.clone();
    let receive_loop = thread::spawn(move || {

        let handle = |obj: json::Object| {
            let mut pov = pov_1.lock().unwrap();
            match obj.get("v") {
                Some(&Json::I64(v)) => {
                    match pov.player.as_mut() {
                        Some(player) => player.version = v,
                        None => ()
                    }
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
                Some(&Json::String(ref t)) if t == "end" => {
                    let _ = tx_1.send(Message::Close(None));
                    // exit
                },
                //Some(&Json::String(ref t)) => println!("unhandled: {:?}", obj),
                _ => ()
            }
        };

        for message in receiver.incoming_messages() {
            let message = match message {
                Ok(m) => m,
                Err(e) => {
                    //println!("Receive Loop: {:?}", e);
                    let _ = tx_1.send(Message::Close(None));
                    return;
                }
            };
            match message {
                Message::Close(_) => {
                    let _ = tx_1.send(Message::Close(None));
                    return;
                }
                Message::Ping(data) => match tx_1.send(Message::Pong(data)) {
                    Ok(()) => (),
                    Err(e) => {
                        //println!("Receive Loop: {:?}", e);
                        return;
                    }
                },
                Message::Text(data) => {
                    //println!("Receive Loop: {:?}", data);
                    let json = Json::from_str(&data).unwrap();
                    if json.is_object() {
                        let obj = json.as_object().unwrap();
                        let t = obj.get("t");
                        //println!("  type: {:?}", t);
                        //println!("full: {:?}", obj);
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
                _ => ()//println!("Receive Loop: {:?}", message),
            }
        }
    });

    let tx_2 = tx.clone();

    // ping loop
    thread::spawn(move || {
        loop {
            std::thread::sleep_ms(1000);

            let pov = pov.lock().unwrap();
            match pov.player {
                Some(ref player) => {
                    let ping_packet = PingPacket {
                        t: "p".to_string(),
                        v: player.version
                    };

                    let message = Message::Text(json::encode(&ping_packet).unwrap());

                    match tx_2.send(message) {
                        Ok(()) => (),
                        Err(e) => {
                            //println!("Main Loop: {:?}", e);
                        }
                    }
                },
                None => () // TODO: error
            };
        }
    });


    /*loop {
        let mut input = String::new();

        stdin().read_line(&mut input).unwrap();

        let trimmed = input.trim();

        let message = match trimmed {
            "/close" => {
                let _ = tx.send(Message::Close(None));
                break;
            }
            "/ping" => Message::Ping(b"{t: 'p', v: 300}".to_vec()),
            _ => Message::Text(trimmed.to_string()),
        };

        match tx.send(message) {
            Ok(()) => (),
            Err(e) => {
                //println!("Main Loop: {:?}", e);
                break;
            }
        }
    }*/

    //let _ = send_loop.join();
    //let _ = receive_loop.join();
}

