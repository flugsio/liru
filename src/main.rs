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
    let base_url = "";
    let lobby_url = "";
    let game_url = "";

    let pov = get_pov(base_url, "tv".to_string());
    match pov {
        Some(pov) => {
            let pov1 = Arc::new(Mutex::new(pov));
            game::socket::connect(base_socket_url, sri, pov1.clone());
            let mut ui = ui::UI::new(pov1.clone());
            ui.start();
        },
        None => ()
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

