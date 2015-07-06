extern crate uuid;
extern crate hyper;
extern crate rustc_serialize;
extern crate websocket;

use std::thread;
use std::sync::{Arc, Mutex};
use std::io::stdin;
use uuid::Uuid;

use std::process::Command;

use std::io::Read;
use hyper::Client;
use hyper::header::Connection;

use hyper::header::{Headers, Accept, qitem};
use hyper::mime::{Mime, TopLevel, SubLevel};

use rustc_serialize::json;
use rustc_serialize::json::Json;

mod game;

fn main() {
    let sri = Uuid::new_v4().to_simple_string();
    let base_url = "";
    let lobby_url = "";
    let game_url = "";

    get_pov("".to_string());
    //return ();
    // tv
    //loop {
        //let output = Command::new("tv").output().unwrap();
        //let game_url = String::from_utf8_lossy(&output.stdout).to_string();

        //println!("disconnected");
        //std::thread::sleep_ms(3000);
        //println!("get next tv");
    //}
    let pov = get_pov(base_url, "mtYp91YZ".to_string());
    match pov {
        Some(pov) => {
            let pov1 = Arc::new(Mutex::new(pov));
            game::socket::connect(base_socket_url, sri, pov1.clone());
            loop {
                std::thread::sleep_ms(4000);
                tui::render_fen(pov1.clone());
            }
        },
        None => ()
    }

}

mod tui {
    extern crate term;

    use std::sync::{Arc, Mutex};

    use game;

    pub fn render_fen(pov: Arc<Mutex<game::Pov>>) {
        let pov = pov.lock().unwrap();
        let fen = pov.game.fen.clone();
        let mut t = term::stdout().unwrap();

        let border_color = term::color::CYAN;
        let piece_dark = term::color::BRIGHT_BLUE;
        let piece_light = term::color::YELLOW;
        let space_dark = term::color::BRIGHT_BLUE;
        let space_light = term::color::YELLOW;

        (write!(t, "\n")).unwrap();
        t.fg(border_color).unwrap();
        (write!(t, "  ╔═════════════════╗\n")).unwrap();
        // TODO: fen parser
        // r1bqkb1r/ppp1pppp/2n2n2/3p4/4P3/3P1P2/PPP3PP/RNBQKBNR w KQkq - 1 4
        for (y, row) in fen.split(' ').next().unwrap().split('/').enumerate() {
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
            (write!(t, "{} ║", 9-(y+1))).unwrap();
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

fn get_pov(base_url: String, game_id: String) -> Option<game::Pov> {
    let url = format!("http://{}/{}", base_url, game_id);
    println!("connecting to {}", url);
    let client = Client::new();
    let mut res = client.get(&*url)
        .header(Connection::close())
        .header(Accept(vec![qitem(Mime(TopLevel::Application, SubLevel::Ext("vnd.lichess.v1+json".to_owned()), vec![]))]))
        .send()
        .unwrap();
    let mut body = String::new();
    res.read_to_string(&mut body).unwrap();
    println!("{}", body);
    let decoded = json::decode(&body).unwrap();
    println!("decoded successfully");
    decoded
}

