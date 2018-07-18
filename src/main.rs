#[macro_use]
extern crate log;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serde_json;

extern crate serde;
extern crate fern;
extern crate uuid;
extern crate futures;
extern crate hyper;
extern crate hyper_tls;
extern crate tokio_core;
extern crate url;
extern crate ws;
extern crate rustbox;
extern crate time;
extern crate cookie;

use std::io;
use std::io::Write;

mod game;
mod ui;
mod lila;

const VERSION: &'static str = env!("CARGO_PKG_VERSION");

fn main() {
    //setup_logger(matches.is_present("debug"));
    setup_logger(true);
    debug!("Init");
    let session = setup_session("Press Enter for anonymous");
    let mut ui = ui::UI::new(session);
    ui.start();
    debug!("Exit");
}

/// Recursively asks for valid credentials
/// or using anonymous with blank username
fn setup_session(message: &str) -> lila::Session {
    println!("{}", message);
    match acquire("Username") {
        ref u if u.is_empty() => lila::Session::anonymous(),
        username =>
            lila::Session::sign_in(username, acquire("Password"))
            .unwrap_or_else(|e| setup_session(e))
    }
}

/// Prints first argument to stdout and
/// expects the user to input one line,
/// which is returned excluding newline
fn acquire(what: &str) -> String {
    print!("{}: ", what);
    io::stdout().flush().unwrap();
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    input.pop();
    input
}

// levels: trace, debug, info, warn, error
fn setup_logger(debug: bool) {
    let level = match debug {
        true  => log::LevelFilter::Debug,
        false => log::LevelFilter::Warn,
    };

    fern::Dispatch::new()
        .format(|out, message, record| {
            let now = time::now_utc();
            out.finish(format_args!(
                    "{}.{}Z [{}] [{}] {}",
                    now.strftime("%Y-%m-%dT%H:%M:%S").unwrap(),
                    now.tm_nsec,
                    record.level(),
                    record.target(),
                    message))
        })
        .level(level)
        .chain(fern::log_file("./liru.log").unwrap())
        .apply().unwrap();
}
