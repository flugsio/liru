extern crate uuid;
extern crate hyper;
extern crate url;
extern crate rustc_serialize;
extern crate websocket;
extern crate rustbox;
extern crate time;

mod game;
mod ui;
mod lila;

fn main() {
    //let session = lila::sign_in("username".to_owned(), "passwordlol".to_owned()).unwrap();
    let session = lila::anonymous();
    let mut ui = ui::UI::new(session);
    ui.start();
}

