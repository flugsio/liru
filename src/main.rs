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

mod game;
mod ui;
mod lila;

fn main() {
    //let session = lila::sign_in("username".to_owned(), "passwordlol".to_owned()).unwrap();
    setup_logger(true);
    debug!("Init");
    let session = lila::anonymous();
    let mut ui = ui::UI::new(session);
    ui.start();
    debug!("Exit");
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
