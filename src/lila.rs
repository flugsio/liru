use serde_derive::Deserialize;

use std::str;
use std::collections::HashMap;

use hyper::{Body, Request};
use hyper::client::Client;
use hyper::header::{
    CONTENT_LENGTH,
    ACCEPT,
    CONNECTION,
    CONTENT_TYPE,
    SET_COOKIE,
    USER_AGENT,
};
use hyper_tls::HttpsConnector;

use serde_json;
use url::form_urlencoded;

use cookie::{Cookie, CookieJar};

pub struct Session {
    pub user: LilaUser,
    pub cookie: Box<CookieJar>,
}

#[allow(non_snake_case)]
#[derive(Deserialize, Debug)]
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

#[derive(Deserialize, Debug)]
pub struct Perf {
    pub games: i64,
    pub rating: i64,
    pub rd: i64,
    pub prov: Option<bool>,
    pub prog: i64,
}

#[derive(Deserialize, Debug)]
pub struct PlayTime {
    pub total: i64,
    pub tv: i64,
}

#[allow(non_snake_case)]
#[derive(Deserialize, Debug)]
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
#[derive(Deserialize, Debug)]
pub struct PlayingOpponent {
    pub id: Option<String>,
    pub username: String,
    pub rating: Option<i64>,
}

impl Session {
    pub fn anonymous() -> Session {
        let cookie = Box::new(CookieJar::new());
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
        let base_url = "https://lichess.org";
        format!("{}/{}", base_url, path)
    }

    pub fn socket_url(path: &str) -> String {
        let base_url = "wss://socket3.lichess.org";
        format!("{}{}", base_url, path)
    }

    pub fn sign_in(username: String, password: String) -> Result<Session, &'static str> {
        let https = HttpsConnector::new();
        let client = Client::builder()
            .build::<_, Body>(https);
        let mut data = String::new();
        form_urlencoded::Serializer::new(&mut data)
            .append_pair("username", &username)
            .append_pair("password", &password);
        let req = Request::builder()
            .method("POST")
            .uri(&Session::url("login"))
            .header(CONTENT_LENGTH, data.len())
            .header(CONNECTION, "close")
            .header(USER_AGENT, format!("liru/{}", crate::VERSION).as_str())
            .header(ACCEPT, "application/vnd.lichess.v1+json")
            .header(CONTENT_TYPE, "application/x-www-form-urlencoded")
            .body(data.into()).unwrap();

        let res = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(async {
                client.request(req).await.unwrap()
            });
        if res.status().is_success() {
            let mut cookie_jar = CookieJar::new();
            for cookie in res.headers().get_all(SET_COOKIE).iter() {
                let cookie = cookie.to_str().unwrap().to_string();
                cookie_jar.add_original(Cookie::parse(cookie).unwrap());
            }
            let b = tokio::runtime::Runtime::new()
                .unwrap()
                .block_on(async {
                    hyper::body::to_bytes(res).await.unwrap()
                });
            let body = str::from_utf8(&b).unwrap();
            log::trace!("{}", &body);
            Ok(Session {
                user: serde_json::from_str(&body).unwrap(),
                cookie: Box::new(cookie_jar),
            })
            //} else if res.status.is_client_error() {
        } else {
            log::error!("Could not login: {}", res.status());
            Err("Could not login")
        }
    }

    pub fn get(&self, path: &str) -> String {
        // TODO: catch error and print
        let https = HttpsConnector::new();
        let client = Client::builder()
            .build::<_, Body>(https);

        let mut builder = Request::builder();
        builder = builder.method("GET")
            .uri(&Session::url(path))
            .header(CONNECTION, "close")
            .header(USER_AGENT, format!("liru/{}", crate::VERSION).as_str())
            .header(ACCEPT, "application/vnd.lichess.v1+json");
        for cookie in self.cookie.iter() {
            builder = builder.header(cookie.name(), cookie.value());
        }
        let req = builder.body(Body::empty()).unwrap();
        let res = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(async {
                hyper::body::to_bytes(client.request(req).await.unwrap()).await.unwrap()
            });
        str::from_utf8(&res).unwrap().to_string()
    }
}
