use std::str;
use std::collections::HashMap;

use futures::{Stream};
use hyper::{Method, Request, Client};
use hyper::header::{
    Accept,
    Connection,
    ContentLength,
    ContentType,
    Cookie,
    SetCookie,
    UserAgent,
    qitem,
};
use tokio_core::reactor::Core;
use hyper_tls::HttpsConnector;

use rustc_serialize::json;
use url::form_urlencoded;

pub struct Session {
    pub user: LilaUser,
    pub cookie: Cookie,
}

#[allow(non_snake_case)]
#[derive(RustcDecodable)]
pub struct LilaUser {
    pub id: String,
    pub username: String,
    pub online: bool,
    pub engine: Option<bool>,
    pub booster: Option<bool>,
    // pub profile :{"country":"SE"}
    pub perfs: HashMap<String, Perf>,
    pub createdAt: i64,
    pub seenAt: i64,
    pub playTime: PlayTime,
    pub nowPlaying: Vec<PlayingGame>,
}

#[derive(RustcDecodable)]
pub struct Perf {
    pub games: i64,
    pub rating: i64,
    pub rd: i64,
    pub prov: Option<bool>,
    pub prog: i64,
}

#[derive(RustcDecodable)]
pub struct PlayTime {
    pub total: i64,
    pub tv: i64,
}

#[allow(non_snake_case)]
#[derive(RustcDecodable)]
pub struct PlayingGame {
    pub fullId: String,
    pub gameId: String,
    pub fen: String,
    //color: white,
    //lastMove: "",
    //variant: {
    //    key: crazyhouse,
    //    name: Crazyhouse
    //},
    //speed: correspondence,
    //perf: crazyhouse,
    //rated: false,
    pub opponent: PlayingOpponent,
    //pub isMyTurn: bool,
    //pub secondsLeft: i64,
}

#[allow(non_snake_case)]
#[derive(RustcDecodable)]
pub struct PlayingOpponent {
    pub id: Option<String>,
    pub username: String,
    pub rating: Option<i64>,
}

impl Session {
    pub fn anonymous() -> Session {
        let cookie = Cookie::new();
        Session {
            user: LilaUser {
                id: "anonymous".to_owned(),
                username: "Anonymous".to_owned(),
                online: true,
                engine: Some(false),
                booster: Some(false),
                perfs: HashMap::new(),
                createdAt: 0,
                seenAt: 0,
                playTime: PlayTime {
                    total: 0,
                    tv: 0,
                },
                nowPlaying: vec!(),
            },
            cookie: cookie,
        }
    }

    pub fn url(path: &str) -> String {
        let base_url = "https://en.lichess.org";
        format!("{}/{}", base_url, path)
    }

    pub fn socket_url(path: &str) -> String {
        let base_url = "wss://socket.lichess.org";
        format!("{}{}", base_url, path)
    }

    pub fn sign_in(username: String, password: String) -> Result<Session, &'static str> {
        let mut core = Core::new().unwrap();
        let handle = core.handle();
        let client = Client::configure()
            .connector(HttpsConnector::new(4, &handle).unwrap())
            .build(&handle);
        let mut data = String::new();
        form_urlencoded::Serializer::new(&mut data)
            .append_pair("username", &username)
            .append_pair("password", &password);
        let mut req = Request::new(Method::Post, (&Session::url("login")).parse().unwrap());
        req.headers_mut().set(ContentLength(data.len() as u64));
        req.headers_mut().set(Connection::close());
        req.headers_mut().set(UserAgent::new(format!("liru/{}", ::VERSION)));
        req.headers_mut().set(Accept(vec![qitem("application/vnd.lichess.v1+json".parse().unwrap())]));
        req.headers_mut().set(ContentType::form_url_encoded());
        req.set_body(data);
        let res = core.run(client.request(req)).unwrap();
        if res.status().is_success() {
            let mut cookie = Cookie::new();
            {
                let cookies = res.headers().get::<SetCookie>().expect("Cookies: session cookie expected!");
                for c in cookies.iter() {
                    let mut split = c.split("=").collect::<Vec<_>>();
                    let key = split.remove(0);
                    let value = split.join("");
                    cookie.set(key.to_owned(), value.to_owned());
                }
            }
            let b = core.run(res.body().concat2()).unwrap();
            let body = str::from_utf8(&b).unwrap();
            trace!("{}", &body);
            Ok(Session {
                user: json::decode(&body).unwrap(),
                cookie: cookie,
            })
            //} else if res.status.is_client_error() {
        } else {
            error!("Could not login: {}", res.status());
            Err("Could not login")
        }
    }

    pub fn get(&self, path: &str) -> String {
        // TODO: catch error and print
        let mut core = Core::new().unwrap();
        let handle = core.handle();
        let client = Client::configure()
            .connector(HttpsConnector::new(4, &handle).unwrap())
            .build(&handle);

        let mut req = Request::new(Method::Get, (&Session::url(path)).parse().unwrap());
        req.headers_mut().set(Connection::close());
        req.headers_mut().set(UserAgent::new(format!("liru/{}", ::VERSION)));
        req.headers_mut().set(Accept(vec![qitem("application/vnd.lichess.v1+json".parse().unwrap())]));
        req.headers_mut().set(self.cookie.clone());
        let res = core.run(client.request(req)).unwrap();
        str::from_utf8((&core.run(res.body().concat2()).unwrap())).unwrap().to_string()
    }

    pub fn cookie(&self) -> Cookie {
        self.cookie.clone()
    }
}
