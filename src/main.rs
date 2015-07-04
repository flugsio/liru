extern crate uuid;
extern crate websocket;
extern crate rustc_serialize;

use std::sync::{Arc, Mutex};
use std::thread;
use std::sync::mpsc::channel;
use std::io::stdin;
use uuid::Uuid;

use websocket::{Message, Sender, Receiver};
use websocket::client::request::Url;
use websocket::Client;

use rustc_serialize::json;
use rustc_serialize::json::Json;

#[derive(RustcEncodable)]
pub struct PingPacket {
    t: String,
    v: u64
}

fn main() {
    println!("Hello, world!");
    let base_url = "";
    let lobby_url = "";
    let game_url = "";

    let sri = Uuid::new_v4().to_simple_string();

    connect(base_url, game_url, sri);
}

fn connect(base_url: &str, url: &str, sri: String) {

    let version = Arc::new(Mutex::new(1u64));

    let url = Url::parse(&format!("ws://{}/{}?sri={}&version={}", base_url, url, sri, 0)).unwrap();
    
    println!("Connecting to {}", url);
    
    let request = Client::connect(url).unwrap(); 
    
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
                    println!("Send Loop: {:?}", e);
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
                    println!("Send Loop: {:?}", e);
                    let _ = sender.send_message(Message::Close(None));
                    return;
                }
            }
        }
    });

    let version_1 = version.clone();
    let receive_loop = thread::spawn(move || {
        for message in receiver.incoming_messages() {
            let message = match message {
                Ok(m) => m,
                Err(e) => {
                    println!("Receive Loop: {:?}", e);
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
                        println!("Receive Loop: {:?}", e);
                        return;
                    }
                },
                Message::Text(data) => {
                    println!("Receive Loop: {:?}", data);
                    let json = Json::from_str(&data).unwrap();
                    if json.is_object() {
                        let obj = json.as_object().unwrap();
                        match obj.get("v") {
                            Some(&Json::U64(v)) => {
                                println!("  version: {}", v);
                                let mut version = version_1.lock().unwrap();
                                *version = v;
                            },
                            _ => ()
                        }
                        let t = obj.get("t").unwrap();
                        println!("  type: {:?}", t);
                    }
                }
                _ => println!("Receive Loop: {:?}", message),
            }
        }
    });

    let tx_2 = tx.clone();

    // ping loop
    thread::spawn(move || {
        loop {
            std::thread::sleep_ms(1000);

            let ping_packet = PingPacket {
                t: "p".to_string(),
                v: *version.lock().unwrap()
            };

            let message = Message::Text(json::encode(&ping_packet).unwrap());
        
            match tx_2.send(message) {
                Ok(()) => (),
                Err(e) => {
                    println!("Main Loop: {:?}", e);
                }
            }
        }
    });
    
    
    loop {
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
                println!("Main Loop: {:?}", e);
                break;
            }
        }
    }
    
    let _ = send_loop.join();
    let _ = receive_loop.join();
}
