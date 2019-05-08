use ws;

use std;
use std::rc::Rc;
use std::cell::Cell;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};

use std::thread;

use log::{error, warn, debug, trace};

use cookie::CookieJar;

use time;

use serde_json;

use url;
use ws::{Request, Handler, Sender, Result, Message, Handshake, CloseCode, Error};
use ws::util::{Token};

const PING: Token = Token(1);

#[derive(Serialize, Debug)]
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
        serde_json::to_string(self).unwrap()
    }
}


pub struct Client {
    out: Sender,
    version: Rc<Cell<u64>>,
    game_tx: mpsc::Sender<serde_json::Value>,
    last_ping: time::Tm,
    cookie: CookieJar,
}

impl Handler for Client {

    fn on_open(&mut self, _: Handshake) -> Result<()> {
        self.out.timeout(10, PING)
    }

    /// Takes care of low level messages. For example
    /// ping/pong responses, and splitting batch messages.
    fn on_message(&mut self, msg: Message) -> Result<()> {
        trace!("Received str: {}", msg);
        let json: serde_json::Value = serde_json::from_str(msg.as_text().unwrap()).unwrap();
        debug!("Received obj: {:?}", json);
        if json.is_object() {
            let obj = json.as_object().unwrap();
            match obj.get("t").and_then(|t| t.as_str()) {
                Some("n") => { // pong, inject travel time
                    let pong = json!({
                        "t": "n",
                        "d": { "latency": self.milliseconds_since_ping() },
                    });
                    self.on_handle(&pong);
                    self.out.timeout(2000, PING).unwrap();
                }
                Some("b") => { // batch
                    let data = obj.get("d").unwrap();
                    for item in data.as_array().unwrap().iter() {
                        self.on_handle(item);
                    }
                },
                _ => self.on_handle(&json!(obj)),
            }
        } else {
            self.on_handle(&json);
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
        let mut r = Request::from_url(url)?;
        r.headers_mut().push(("User-Agent".into(), format!("liru/{}", crate::VERSION).into()));
        r.headers_mut().push(("Cookie".into(), format!("{:?}", self.cookie).into()));
        debug!("Built request: {:?}", r);
        Ok(r)
    }

}

impl Client {
    pub fn connect(c: &CookieJar, url: String, version: u64,
                   game_tx: mpsc::Sender<serde_json::Value>,
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

    fn on_handle(&mut self, obj: &serde_json::Value) {
        // If message is versioned, it must have the expected version
        match obj.get("v").map_or(Ok(()), |v| self.update_version(v.as_u64())) {
            Ok(()) => self.game_tx.send(obj.to_owned()).unwrap(),
            Err(e) => debug!("Dropping unexpected message. {}", e),
        }
    }

    fn update_version(&mut self, version: Option<u64>) -> std::result::Result<(), String> {
        let expected = self.version.get() + 1;
        match version {
            Some(v) if (v < expected) => Err(format!("Already has event {}", v)),
            Some(v) if (v > expected) =>
                Err(format!("Event gap detected, expected {} but got {}", expected, v)),
            Some(v) => Ok(self.version.set(v)),
            None => Err("Version value is not u64".into()),
        }
    }

    fn milliseconds_since_ping(&self) -> i64 {
        (time::now_utc() - self.last_ping).num_milliseconds()
    }
}

