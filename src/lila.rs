use std::io::Read;
use std::collections::HashMap;

use hyper::Client;
use hyper::header::{
    Accept,
    Connection,
    ContentType,
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
    pub engine: bool,
    pub booster: bool,
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
    //opponent: {
    //    id:test,
    //    username: test,
    //    rating: 1500
    //},
    //isMyTurn: true,
    //secondsLeft: 120613
}

pub fn anonymous() -> Session {
    let mut cjar = CookieJar::new(b"a234lj5sdfla234sjdfasldkfjlasdf");
    Session {
        user: LilaUser {
            id: "anonymous".to_owned(),
            username: "Anonymous".to_owned(),
            online: true,
            engine: false,
            booster: false,
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

pub fn sign_in(username: String, password: String) -> Result<Session, &'static str> {
    let base_url = "en.lichess.org".to_string();
    let url = format!("https://{}/login", base_url);
    let client = Client::new();
    let mut data = String::new();
    let mut body = String::new();
    form_urlencoded::Serializer::new(&mut data)
        .append_pair("username", &username)
        .append_pair("password", &password);
    let mut res = client.post(&*url)
        .body(&data)
        .header(Connection::close())
        .header(Accept(vec![qitem(Mime(
                        TopLevel::Application,
                        SubLevel::Ext("vnd.lichess.v1+json".to_owned()),
                        vec![]))]))
        .header(UserAgent("liru/0.1.1".to_owned()))
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
        res.read_to_string(&mut body);
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

