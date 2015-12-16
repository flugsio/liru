extern crate uuid;
extern crate hyper;
extern crate rustc_serialize;
extern crate websocket;
extern crate rustbox;
extern crate time;

use std::sync::{Arc, Mutex};
use uuid::Uuid;

use std::io::Read;
use hyper::Client;
use hyper::header::Connection;

use hyper::header::{Headers, Accept, qitem};
use hyper::mime::{Mime, TopLevel, SubLevel};

use rustc_serialize::json;

mod game;
mod ui;

fn main() {
    let sri = Uuid::new_v4().to_simple_string();
    let base_url = "en.lichess.org".to_string();
    let base_socket_url = "socket.en.lichess.org".to_string();

    let mut ui = ui::UI::new();
    let pov = get_pov(base_url, "tv/bullet".to_string());
    match pov {
        Some(pov) => {
            let pov1 = Arc::new(Mutex::new(pov));
            game::socket::connect(base_socket_url, sri, pov1.clone());
            ui.add_game(pov1.clone());
        },
        None => println!("no pov")
    }
    ui.start();

}

fn get_pov(base_url: String, game_id: String) -> Option<game::Pov> {
    let url = format!("http://{}/{}", base_url, game_id);
    let client = Client::new();
    let mut body = String::new();
    client.get(&*url)
        .header(Connection::close())
        .header(Accept(vec![qitem(Mime(TopLevel::Application, SubLevel::Ext("vnd.lichess.v1+json".to_owned()), vec![]))]))
        .send()
        .map(|mut res| {
            res.read_to_string(&mut body);
        });
    // TODO: catch error and print
    json::decode(&body).unwrap()
}

