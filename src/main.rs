extern crate uuid;
extern crate websocket;
extern crate rustc_serialize;
extern crate hyper;

use std::sync::{Arc, Mutex};
use std::thread;
use std::sync::mpsc::channel;
use std::io::stdin;
use uuid::Uuid;

use websocket::{Message, Sender, Receiver};
use websocket::client::request::Url;

use rustc_serialize::json;
use rustc_serialize::json::Json;

use std::process::Command;

use std::io::Read;
use hyper::Client;
use hyper::header::Connection;

use hyper::header::{Headers, Accept, qitem};
use hyper::mime::{Mime, TopLevel, SubLevel};

#[derive(RustcEncodable)]
pub struct PingPacket {
    t: String,
    v: u64
}

fn main() {
    let sri = Uuid::new_v4().to_simple_string();
    let base_url = "";
    let lobby_url = "";
    let game_url = "";

    get_pov("".to_string());
    return ();
    // tv
    loop {
        let output = Command::new("tv").output().unwrap();
        let game_url = String::from_utf8_lossy(&output.stdout).to_string();
        connect(base_url, game_url, sri.clone());
        println!("disconnected");
        std::thread::sleep_ms(3000);
        println!("get next tv");
    }
}

fn connect(base_url: &str, url: String, sri: String) {

    let version = Arc::new(Mutex::new(1u64));

    let url = Url::parse(&format!("ws://{}/{}?sri={}&version={}", base_url, url, sri, 0)).unwrap();

    //println!("Connecting to {}", url);

    let request = websocket::Client::connect(url).unwrap();

    let response = request.send().unwrap();

    match response.validate() {
        Ok(()) => (),
        Err(e) => {
            //println!("reponse validation error: {:?}", e);
            //println!("reponse status: {:?}, headers: {:?}", response.status, response.headers);
            return;
        }
    };

    //println!("Successfully connected");

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

    let version_1 = version.clone();
    let receive_loop = thread::spawn(move || {

        let handle = |obj: json::Object| {
            match obj.get("v") {
                Some(&Json::U64(v)) => {
                    //println!("  version: {}", v);
                    let mut version = version_1.lock().unwrap();
                    *version = v;
                },
                _ => ()
            }
            match obj.get("t") {
                Some(&Json::String(ref t)) if t == "move" => {
                    let d = obj.get("d").unwrap().as_object().unwrap();
                    let fen = d.get("fen").unwrap().as_string().unwrap();
                    cli::render_fen(fen);
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

            let ping_packet = PingPacket {
                t: "p".to_string(),
                v: *version.lock().unwrap()
            };

            let message = Message::Text(json::encode(&ping_packet).unwrap());

            match tx_2.send(message) {
                Ok(()) => (),
                Err(e) => {
                    //println!("Main Loop: {:?}", e);
                }
            }
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

    let _ = send_loop.join();
    //let _ = receive_loop.join();
}

mod cli {
    extern crate term;

    pub fn render_fen(fen: &str) {
        let mut t = term::stdout().unwrap();

        let border_color = term::color::CYAN;
        let piece_dark = term::color::BRIGHT_BLUE;
        let piece_light = term::color::YELLOW;
        let space_dark = term::color::BRIGHT_BLUE;
        let space_light = term::color::YELLOW;

        (write!(t, "\n")).unwrap();
        t.fg(border_color).unwrap();
        (write!(t, "  ╔═════════════════╗\n")).unwrap();
        for (y, row) in fen.split('/').enumerate() {
            let row = row
                .replace("8", "········")
                .replace("7", "·······")
                .replace("6", "······")
                .replace("5", "·····")
                .replace("4", "····")
                .replace("3", "···")
                .replace("2", "··")
                .replace("1", "·");
            t.fg(border_color).unwrap();
            (write!(t, "  ║")).unwrap();
            for (x, char) in row.chars().enumerate() {
                if char == '·' {
                    if (y + x) % 2 == 0 {
                        t.fg(space_light).unwrap();
                    } else {
                        t.fg(space_dark).unwrap();
                    }
                } else {
                    if char.is_uppercase() {
                        t.fg(piece_light).unwrap();
                    } else {
                        t.fg(piece_dark).unwrap();
                    }
                }
                (write!(t, " {}", char.to_uppercase().collect::<String>())).unwrap();
            }
            t.fg(border_color).unwrap();
            (write!(t, " ║\n")).unwrap();
        }
        (write!(t, "  ╚═════════════════╝\n")).unwrap();

        assert!(t.reset().unwrap());
    }
}

fn get_pov(game_id: String) -> Option<game::Pov> {
    let url = format!("http://en.l.org/{}", game_id);
    let client = Client::new();
    let mut res = client.get(&*url)
        .header(Connection::close())
        .header(Accept(vec![qitem(Mime(TopLevel::Application, SubLevel::Ext("vnd.lichess.v1+json".to_owned()), vec![]))]))
        .send()
        .unwrap();
    let mut body = String::new();
    res.read_to_string(&mut body).unwrap();
    println!("{}", body);
    json::decode(&body).unwrap()
}

pub mod game {

    #[derive(RustcDecodable)]
    pub struct Pov {
        pub game: Game,
        pub url: GameUrl,
        pub player: Option<Player>,
    }

    #[derive(RustcDecodable)]
    pub struct GameUrl {
        pub socket: String,
        pub round: String,
    }

    #[derive(RustcDecodable)]
    pub struct Game {
        pub id: String,
        pub speed: String,
        pub perf: String,
        pub rated: bool,
        pub initialFen: String,
        pub fen: String,
        pub moves: String,
        pub player: String,
        pub turns: i64,
        pub startedAtTurn: i64,
        pub lastMove: Option<String>,
        pub threefold: bool,
        pub source: String,
    }

    #[derive(RustcDecodable)]
    pub struct Player {
        pub color: String,
        pub version: i64,
    }
}
