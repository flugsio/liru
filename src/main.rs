extern crate uuid;
extern crate hyper;
extern crate rustc_serialize;
extern crate websocket;
extern crate rustbox;
extern crate time;

mod game;
mod ui;

fn main() {
    let mut ui = ui::UI::new();
    ui.start();
}

