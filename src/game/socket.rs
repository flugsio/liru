extern crate ws;

use std::rc::Rc;
use std::cell::Cell;
use std::sync::mpsc;
use std::collections::BTreeMap;

use hyper::header::{HeaderFormatter, Cookie};

use time;

use rustc_serialize::json;
use rustc_serialize::json::Json;

use url;
use ws::{Request, Handler, Sender, Result, Message, Handshake, CloseCode, Error};
use ws::util::{Token};

const PING: Token = Token(1);

#[derive(RustcEncodable)]
struct PingPacket {
    t: String,
    v: u64
}

impl PingPacket {
    pub fn new(version: u64) -> PingPacket {
        PingPacket {
            t: "p".to_string(),
            v: version
        }
    }
    
    pub fn to_message(&self) -> String {
        json::encode(self).unwrap()
    }
}


pub struct Client {
    out: Sender,
    version: Rc<Cell<u64>>,
    game_tx: mpsc::Sender<BTreeMap<String, json::Json>>,
    last_ping: Rc<Cell<time::Tm>>,
    cookie: Cookie,
}

impl Handler for Client {

    fn on_open(&mut self, _: Handshake) -> Result<()> {
        self.out.timeout(2000, PING)
    }

    fn on_message(&mut self, msg: Message) -> Result<()> {
        trace!("Received str: {}", msg);
        let json = Json::from_str(msg.as_text().unwrap()).unwrap();
        debug!("Received obj: {:?}", json);
        if json.is_object() {
            let obj = json.as_object().unwrap();
            match obj.get("t").and_then(|t| t.as_string()) {
                Some("n") => { // pong
                    debug!("Ping time: {}", time::now_utc()-self.last_ping.get());
                    self.out.timeout(2000, PING).unwrap();
                }
                Some("b") => { // batch
                    let data = obj.get("d").unwrap();
                    for item in data.as_array().unwrap().iter() {
                        let obj = item.as_object().unwrap();
                        self.on_handle(obj);
                    }
                },
                _ => self.on_handle(obj),
            }
        }
        Ok(())
    }

    fn on_timeout(&mut self, event: Token) -> Result<()> {
        match event {
            PING => {
                //try!(self.out.ping(time::precise_time_ns().to_string().into()));
                let ping = PingPacket::new(self.version.get()).to_message();
                debug!("sending: {}", ping.clone());
                self.last_ping.set(time::now_utc());
                self.out.send(ping)
            }
            _ => Ok(error!("Invalid timeout token encountered!")),
        }
    }

    fn on_close(&mut self, code: CloseCode, reason: &str) {
        debug!("on_close");
        match code {
            CloseCode::Normal => debug!("The client is done with the connection."),
            CloseCode::Away   => debug!("The client is leaving the site."),
            CloseCode::Abnormal => warn!(
                "Closing handshake failed! Unable to obtain closing status from client."),
                _ => error!("The client encountered an error: {}", reason),
        }
    }

    fn on_error(&mut self, err: Error) {
        error!("The server encountered an error: {:?}", err);
    }

    fn build_request(&mut self, url: &url::Url) -> Result<Request> {
        debug!("Handler is building request from {}.", url);
        let mut r = try!(Request::from_url(url));
        debug!("Adding header name {}, value {}", "Cookie", format!("{:?}", HeaderFormatter(&self.cookie)));
        r.headers_mut().push(("Cookie".to_string(), format!("{:?}", HeaderFormatter(&self.cookie)).into()));
        Ok(r)
    }

}

impl Client {
    pub fn connect(c: &Cookie, url: String, version: u64, game_tx: mpsc::Sender<BTreeMap<String, json::Json>>) {
        debug!("connecting to: {}", url.clone());
        let v = Rc::new(Cell::new(version));
        let last_ping = Rc::new(Cell::new(time::now_utc()));
        ws::connect(url.to_owned(), |out| {
            Client {
                out: out,
                version: v.clone(),
                game_tx: game_tx.clone(),
                last_ping: last_ping.clone(),
                cookie: c.clone(),
            } }).unwrap()
    }

    fn on_handle(&mut self, obj: &BTreeMap<String, json::Json>) {
        if let Some(v) = obj.get("v").and_then(|v| v.as_u64()) {
            let current = self.version.get();
            if v <= current {
                debug!("Already has event {}", v);
                return;
            } else if current + 1 < v {
                debug!("Event gap detected from {} to {}", current, v);
                return;
            } else {
                self.version.set(v);
            }
        }
        self.game_tx.send(obj.to_owned()).unwrap();
    }
}

