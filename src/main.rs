#[macro_use]
extern crate log;
extern crate fern;
extern crate uuid;
extern crate hyper;
extern crate url;
extern crate rustc_serialize;
extern crate websocket;
extern crate rustbox;
extern crate time;

use std::io;
use std::io::Write;

mod game;
mod ui;
mod lila;

fn main() {
    let session = setup_session("Press Enter for anonymous");
    //setup_logger(matches.is_present("debug"));
    setup_logger(true);
    debug!("Init");
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
    let output = vec![fern::OutputConfig::file("./liru.log")];
    let level = match debug {
        true  => log::LogLevelFilter::Debug,
        false => log::LogLevelFilter::Warn,
    };

    let config = fern::DispatchConfig {
        format: Box::new(|msg, level, location| {
            let now = time::now_utc();
            format!("{}.{}Z [{}] [{}] {}",
                    now.strftime("%Y-%m-%dT%H:%M:%S").unwrap(), now.tm_nsec,
                    level, location.module_path(), msg)
        }),
        output: output,
        level: level,
    };

    if let Err(e) = fern::init_global_logger(config, log::LogLevelFilter::Trace) {
        println!("Failed to initialize global logger: {}", e);
    };
}
