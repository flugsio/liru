use std::io::Read;
use std::collections::HashMap;

use hyper::Client;
use hyper::header::{
    Accept,
    Connection,
    ContentType,
    Cookie,
    CookieJar,
    SetCookie,
    UserAgent,
    qitem,
};
use hyper::mime::{Mime, TopLevel, SubLevel};

use rustc_serialize::json;
use url::form_urlencoded;

pub struct Session {
    pub user: LilaUser,
    pub cjar: CookieJar<'static>,
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
    pub prov: bool,
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
        let cjar = CookieJar::new(b"a234lj5sdfla234sjdfasldkfjlasdf");
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
            cjar: cjar,
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
        let client = Client::new();
        let mut data = String::new();
        let mut body = String::new();
        form_urlencoded::Serializer::new(&mut data)
            .append_pair("username", &username)
            .append_pair("password", &password);
        let mut res = client.post(&Session::url("login"))
            .body(&data)
            .header(Connection::close())
            .header(Accept(vec![qitem(Mime(
                            TopLevel::Application,
                            SubLevel::Ext("vnd.lichess.v1+json".to_owned()),
                            vec![]))]))
            .header(UserAgent(format!("liru/{}", ::VERSION)))
            .header(ContentType::form_url_encoded())
            .send()
            .unwrap();
        if res.status.is_success() {
            let cookie = match res.headers.get::<SetCookie>() {
                Some(cookie) => {
                    cookie.to_owned()
                },
                None => {
                    panic!("Cookies: session cookie expected!");
                }
            };
            res.read_to_string(&mut body).ok();
            trace!("{}", &body);
            let mut cjar = CookieJar::new(b"a234lj5sdfla234sjdfasldkfjlasdf");
            cookie.apply_to_cookie_jar(&mut cjar);
            Ok(Session {
                user: json::decode(&body).unwrap(),
                cjar: cjar,
            })
            //} else if res.status.is_client_error() {
        } else {
            error!("Could not login: {}", res.status);
            Err("Could not login")
        }
    }

    pub fn get(&self, path: &str, mut body: &mut String) {
        // TODO: catch error and print
        let client = Client::new();
        client.get(&Session::url(path))
            .header(Connection::close())
            .header(Accept(vec![qitem(Mime(TopLevel::Application, SubLevel::Ext("vnd.lichess.v1+json".to_owned()), vec![]))]))
            .header(Cookie::from_cookie_jar(&self.cjar))
            .header(UserAgent(format!("liru/{}", ::VERSION)))
            .send()
            .map(|mut res| res.read_to_string(&mut body).ok()).unwrap();
    }

    pub fn cookie(&self) -> Cookie {
        Cookie::from_cookie_jar(&self.cjar)
    }
}
