extern crate ws;

use std::rc::Rc;
use std::cell::Cell;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};

use std::thread;

use cookie::CookieJar;

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
            t: "p".into(),
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
    game_tx: mpsc::Sender<json::Object>,
    last_ping: time::Tm,
    cookie: CookieJar,
}

impl Handler for Client {

    fn on_open(&mut self, _: Handshake) -> Result<()> {
        self.out.timeout(10, PING)
    }

    fn on_message(&mut self, msg: Message) -> Result<()> {
        trace!("Received str: {}", msg);
        let json = Json::from_str(msg.as_text().unwrap()).unwrap();
        debug!("Received obj: {:?}", json);
        if json.is_object() {
            let obj = json.as_object().unwrap();
            match obj.get("t").and_then(|t| t.as_string()) {
                Some("n") => { // pong, inject travel time
                    let latency = time::now_utc() - self.last_ping;
                    // this mess changes {"t":"n"} to {"t":"n","d":{"latency":30}}
                    let mut data = json::Object::new();
                    data.insert("latency".into(), Json::I64(latency.num_milliseconds()));
                    let mut pong = obj.clone();
                    pong.insert("d".into(), Json::Object(data));
                    self.on_handle(&pong);
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
                let ping = PingPacket::new(self.version.get()).to_message();
                self.last_ping = time::now_utc();
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
        r.headers_mut().push(("User-Agent".into(), format!("liru/{}", ::VERSION).into()));
        r.headers_mut().push(("Cookie".into(), format!("{:?}", self.cookie).into()));
        debug!("Built request: {:?}", r);
        Ok(r)
    }

}

impl Client {
    pub fn connect(c: &CookieJar, url: String, version: u64,
                   game_tx: mpsc::Sender<json::Object>,
                   send_rx: Arc<Mutex<mpsc::Receiver<String>>>) {
        debug!("connecting to: {}", url.clone());
        let v = Rc::new(Cell::new(version));

        ws::connect(url.to_owned(), |out| {
            {
                let out2 = out.clone();
                let send_rx = send_rx.clone();
                thread::spawn(move || {
                    let send_rx = send_rx.lock().unwrap();
                    loop {
                        let msg = send_rx.recv().unwrap();
                        debug!("Sending: {}", msg);
                        out2.send(msg).unwrap();
                    };
                });
            }
            Client {
                out: out.clone(),
                version: v.clone(),
                game_tx: game_tx.clone(),
                last_ping: time::now_utc(),
                cookie: c.clone(),
            }
        }).unwrap();
    }

    fn on_handle(&mut self, obj: &json::Object) {
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

