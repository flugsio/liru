extern crate uuid;
extern crate hyper;
extern crate rustc_serialize;
extern crate websocket;
extern crate rustbox;
extern crate time;

use std::thread;
use std::sync::{Arc, Mutex};
use std::io::stdin;
use uuid::Uuid;
use time::Duration;

use std::process::Command;

use std::io::Read;
use hyper::Client;
use hyper::header::Connection;

use hyper::header::{Headers, Accept, qitem};
use hyper::mime::{Mime, TopLevel, SubLevel};

use rustc_serialize::json;
use rustc_serialize::json::Json;

use rustbox::Key;

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
    let pov = get_pov(base_url, "tv".to_string());
    match pov {
        Some(pov) => {
            let pov1 = Arc::new(Mutex::new(pov));
            // TODO: clean this
            let rustbox = tui::init();
            game::socket::connect(base_socket_url, sri, pov1.clone());
            loop {
                tui::render_fen(&rustbox, pov1.clone());
                match rustbox.peek_event(Duration::milliseconds(100), false) {
                    Ok(rustbox::Event::KeyEvent(key)) => {
                        match key {
                            Some(Key::Char('q')) => { break; }
                            _ => { }
                        }
                    },
                    Err(e) => panic!("{}", e),
                    _ => { }
                }
            }
        },
        None => ()
    }

}

mod tui {
    extern crate rustbox;

    use std::sync::{Arc, Mutex};

    use std::default::Default;

    use rustbox::{Color, RustBox};
    use rustbox::Key;
    use rustbox::{RB_BOLD, RB_NORMAL};

    use game;

    pub fn init() -> RustBox {
        match RustBox::init(Default::default()) {
            Result::Ok(v) => v,
            Result::Err(e) => panic!("{}", e),
        }
    }

    #[derive(Clone, Copy)]
    struct RBStyle {
        style: rustbox::Style,
        fg: Color,
        bg: Color,
    }

    pub fn render_fen(rb: &RustBox, pov: Arc<Mutex<game::Pov>>) {
        let pov = pov.lock().unwrap();
        let fen = pov.game.fen.clone();

        let border = RBStyle { style: RB_NORMAL, fg: Color::Cyan, bg: Color::Black };
        let piece_dark = RBStyle { style: RB_BOLD, fg: Color::Blue, bg: Color::Black };
        let piece_light = RBStyle { style: RB_NORMAL, fg: Color::Yellow, bg: Color::Black };
        let space_dark = RBStyle { style: RB_BOLD, fg: Color::Blue, bg: Color::Black };
        let space_light = RBStyle { style: RB_NORMAL, fg: Color::Yellow, bg: Color::Black };

        rb.print(3,  1, RB_BOLD, Color::White, Color::Black, &fen);
        rb.print(5,  3, border.style, border.fg, border.bg, "╔═════════════════╗");
        rb.print(5, 12, border.style, border.fg, border.bg, "╚═════════════════╝");

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
            rb.print(3, 4 + y, border.style, border.fg, border.bg, &format!("{} ║", 9-(y+1)));
            rb.print(23, 4 + y, border.style, border.fg, border.bg, "║");
            for (x, char) in row.chars().enumerate() {
                let color = if char == '·' {
                    if (y + x) % 2 == 0 {
                        space_light
                    } else {
                        space_dark
                    }
                } else {
                    if char.is_uppercase() {
                        piece_light
                    } else {
                        piece_dark
                    }
                };
                rb.print(7 + x*2, 4 + y, color.style, color.fg, color.bg, &char.to_uppercase().collect::<String>());
            }
        }
        rb.present();
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

