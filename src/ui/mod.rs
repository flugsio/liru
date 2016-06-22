extern crate rustbox;

use std::sync::{Arc, Mutex};

use std::default::Default;

use rustbox::{Color, RustBox};
use rustbox::Key;
use rustbox::{RB_BOLD, RB_NORMAL};

use time::Duration;

use game;
use lila;

#[derive(Clone, Copy)]
struct RBStyle {
    style: rustbox::Style,
    fg: Color,
    bg: Color,
}

trait View {
    fn name(&self) -> &str;
    fn render(&self, ui: &mut Renderer);
    fn key_event(&mut self, key: rustbox::keyboard::Key) -> MenuResult;
}

enum MenuResult {
    AddGameView { name: String, url: String },
    None
}

struct MenuView {
    menuOptions: Vec<MenuOption>,
    current: usize,
}

struct GameView {
    name: String,
    url: String,
    povs: Vec<Arc<Mutex<game::Pov>>>,
}

pub enum MenuOption {
    WatchTv {
        name: String,
        url: String,
    },
}

pub struct UI {
    running: bool,
    renderer: Renderer,
    views: Vec<Box<View>>,
    current_view: usize,
    session: lila::Session,
}

struct Renderer {
    rb: RustBox,
}

impl UI {
    pub fn new(session: lila::Session) -> UI {

        let rb = match RustBox::init(Default::default()) {
            Result::Ok(v) => v,
            Result::Err(e) => panic!("{}", e),
        };

        let mut views = Vec::new();
        let mut menuOptions = Vec::new();
        menuOptions.push(MenuOption::WatchTv { name: "Best".to_string(), url: "tv/best".to_string() });
        menuOptions.push(MenuOption::WatchTv { name: "Bullet".to_string(), url: "tv/bullet".to_string() });
        menuOptions.push(MenuOption::WatchTv { name: "Blitz".to_string(), url: "tv/blitz".to_string() });
        menuOptions.push(MenuOption::WatchTv { name: "Classical".to_string(), url: "tv/classical".to_string() });
        menuOptions.push(MenuOption::WatchTv { name: "Crazyhouse".to_string(), url: "tv/crazyhouse".to_string() });
        menuOptions.push(MenuOption::WatchTv { name: "Chess 960".to_string(), url: "tv/chess960".to_string() });
        menuOptions.push(MenuOption::WatchTv { name: "King of the Hill".to_string(), url: "tv/kingOfTheHill".to_string() });
        menuOptions.push(MenuOption::WatchTv { name: "Three-Check".to_string(), url: "tv/threeCheck".to_string() });
        menuOptions.push(MenuOption::WatchTv { name: "Antichess".to_string(), url: "tv/antichess".to_string() });
        menuOptions.push(MenuOption::WatchTv { name: "Atomic".to_string(), url: "tv/atomic".to_string() });
        menuOptions.push(MenuOption::WatchTv { name: "Horde".to_string(), url: "tv/horde".to_string() });
        menuOptions.push(MenuOption::WatchTv { name: "Racing Kings".to_string(), url: "tv/racingKings".to_string() });
        menuOptions.push(MenuOption::WatchTv { name: "Computer".to_string(), url: "tv/computer".to_string() });

        for game in &session.user.nowPlaying {
            menuOptions.push(MenuOption::WatchTv { name: game.gameId.clone(), url: game.fullId.clone() });
        }

        views.push(Box::new(MenuView {
            menuOptions: menuOptions,
            current: 0,
        }) as Box<View>);

        return UI {
            running: true,
            renderer: Renderer { rb: rb },
            views: views,
            current_view: 0,
            session: session,
        };
    }

    pub fn add_view(&mut self, view: Box<View>) {
        self.views.push(view);
    }

    pub fn add_game(&mut self, name: String, url: String) {
        let game = GameView::new(&self.session.cjar, name, url);
        self.add_view(Box::new(game) as Box<View>);
    }

    pub fn start(&mut self) {
        while self.running {
            self.render();
            self.poll_keys();
        }
    }

    fn prev_view(&mut self) {
        if 0 < self.current_view {
            self.current_view -= 1;
            self.renderer.clear();
        }
    }

    fn next_view(&mut self) {
        if self.current_view < self.views.len() - 1 {
            self.current_view += 1;
            self.renderer.clear();
        }
    }

    fn current_view(&mut self) -> &mut Box<View> {
        self.views.get_mut(self.current_view).unwrap()
    }

    pub fn poll_keys(&mut self) {
        match self.renderer.rb.peek_event(Duration::milliseconds(100), false) {
            Ok(rustbox::Event::KeyEvent(key)) => {
                match key {
                    Some(Key::Char('q')) => self.running = false,
                    Some(Key::Left) => self.prev_view(),
                    Some(Key::Right) => self.next_view(),
                    Some(key) => {
                        match self.current_view().key_event(key) {
                            MenuResult::AddGameView { name, url } => {
                                self.add_game(name, url);
                            }
                            MenuResult::None => {},
                        }
                    }
                    _ => {}
                }
            },
            Err(e) => panic!("{}", e),
            _ => { }
        }
    }

    pub fn render(&mut self) {
        let dark  = RBStyle { style: RB_NORMAL, fg: Color::Blue, bg: Color::Black };
        let light = RBStyle { style: RB_NORMAL, fg: Color::Yellow, bg: Color::Black };

        self.renderer.print(0, 0, dark, "|");
        let mut x = 2;
        for (i, view) in self.views.iter().enumerate() {
            if i == self.current_view {
                self.renderer.print(x, 0, light, view.name());
            } else {
                self.renderer.print(x, 0, dark, view.name());
            }
            x += view.name().len() + 3;
            self.renderer.print(x- 2, 0, dark, "|");
        }

        self.views.get(self.current_view).unwrap().render(&mut self.renderer);
        self.renderer.present();
    }

}

impl Renderer {
    pub fn print(&self, x: usize, y: usize, rbstyle: RBStyle, chars: &str) {
        self.rb.print(x, y, rbstyle.style, rbstyle.fg, rbstyle.bg, chars);
    }

    pub fn clear(&self) {
        self.rb.clear();
    }

    pub fn present(&self) {
        self.rb.present();
    }
}

impl View for MenuView {
    fn render(&self, r: &mut Renderer) {
        let dark  = RBStyle { style: RB_BOLD, fg: Color::Blue, bg:     Color::Black };
        let light = RBStyle { style: RB_NORMAL, fg: Color::Yellow, bg: Color::Black };

        for (i, option) in self.menuOptions.iter().enumerate() {
            if i == self.current {
                r.print(0, 2 + i, light, option.name());
            } else {
                r.print(0, 2 + i, dark, option.name());
            }
        }
    }

    fn name(&self) -> &str {
        "Menu"
    }

    fn key_event(&mut self, key: rustbox::keyboard::Key) -> MenuResult {
        match key {
            Key::Up => {
                if 0 < self.current {
                    self.current -= 1;
                }
                MenuResult::None
            },
            Key::Down => {
                if self.current < self.menuOptions.len() -1 {
                    self.current += 1;
                }
                MenuResult::None
            },
            Key::Enter => {
                let option = self.menuOptions.get(self.current).unwrap(); // TODO
                option.execute()
            }
            _ => { MenuResult::None }
        }
    }
}

impl View for GameView {
    fn render(&self, r: &mut Renderer) {
        for (i, pov) in self.povs.iter().enumerate() {
            pov.lock().map(|p| {
                self.render_pov(r, i * 30, 0, &p);
            });
        }
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn key_event(&mut self, key: rustbox::keyboard::Key) -> MenuResult {
        MenuResult::None
    }
}

impl MenuView {
}


// TODO: move all this
//use std::sync::{Arc, Mutex};
use uuid::Uuid;

use std::io::Read;
use hyper::Client;
use hyper::header::Connection;

use hyper::header::{Headers, Cookie, CookieJar, Accept, qitem};
use hyper::mime::{Mime, TopLevel, SubLevel};

use rustc_serialize::json;

impl GameView {
    pub fn new(jar: &CookieJar<'static>, name: String, url: String) -> GameView {
        // TODO: move this
        fn get_pov(jar: &CookieJar<'static>, base_url: String, game_id: String) -> Option<game::Pov> {
            let url = format!("https://{}/{}", base_url, game_id);
            let client = Client::new();
            let mut body = String::new();
            client.get(&*url)
                .header(Connection::close())
                .header(Accept(vec![qitem(Mime(TopLevel::Application, SubLevel::Ext("vnd.lichess.v1+json".to_owned()), vec![]))]))
                .header(Cookie::from_cookie_jar(&jar))
                .send()
                .map(|mut res| {
                    res.read_to_string(&mut body);
                });
            // TODO: catch error and print
            json::decode(&body).unwrap()
        }

        // TODO: should this be reused or new for each socket?
        let sri = Uuid::new_v4().to_simple_string();
        let base_url = "en.lichess.org".to_string();
        let base_socket_url = "socket.lichess.org".to_string();

        let mut povs = Vec::new();

        let pov = get_pov(jar, base_url, url.clone());
        match pov {
            Some(pov) => {
                let pov1 = Arc::new(Mutex::new(pov));
                game::socket::connect(jar, base_socket_url, sri, pov1.clone());
                povs.push(pov1);
            },
            None => warn!("no pov")
        }

        return GameView {
            name: name,
            url: url,
            povs: povs,
        };
    }

    pub fn render_pov(&self, r: &mut Renderer, x: usize, y: usize, pov: &game::Pov) {
        self.render_player(r, x + 1, y + 2, &pov.opponent);
        self.render_player(r, x + 1, y + 14, &pov.player);
        let fen = pov.game.fen.clone();
        self.render_fen(r, x, y, fen);
        match pov.clock {
            Some(ref clock) => {
                self.render_clock(r, x + 19, y + 3, clock.black);
                self.render_clock(r, x + 19, y + 12, clock.white);
            },
            None => ()
        };
    }

    pub fn render_player(&self, r: &mut Renderer, x: usize, y: usize, player: &game::Player) {
        let style = RBStyle { style: RB_BOLD, fg: Color::White, bg: Color::Black };
        r.print(x, y, style, &format!("{:4}", player.rating.unwrap_or(1500)));
        match player.user {
            Some(ref user) => {
                r.print(x + 5, y, style, &format!("{}", user.username));
            },
            None => ()
        };
    }

    pub fn render_clock(&self, r: &mut Renderer, x: usize, y: usize, time: f64) {
        r.print(x, y, RBStyle { style: RB_BOLD, fg: if time < 10f64 { Color::Red } else { Color::White }, bg: Color::Black }, &format!("{:04.1}", time));
    }

    pub fn render_fen(&self, r: &mut Renderer, x: usize, y: usize, fen: String) {
        let text_style  = RBStyle { style: RB_BOLD, fg: Color::White, bg:    Color::Black };
        let border      = RBStyle { style: RB_NORMAL, fg: Color::Cyan, bg:   Color::Black };
        let piece_dark  = RBStyle { style: RB_BOLD, fg: Color::Blue, bg:     Color::Black };
        let piece_light = RBStyle { style: RB_NORMAL, fg: Color::Yellow, bg: Color::Black };
        let space_dark  = RBStyle { style: RB_BOLD, fg: Color::Blue, bg:     Color::Black };
        let space_light = RBStyle { style: RB_NORMAL, fg: Color::Yellow, bg: Color::Black };

        //r.print(x + 3, y +  1, text_style, &fen);
        r.print(x + 5, y +  3, border, "╔═════════════════╗");
        r.print(x + 5, y + 12, border, "╚═════════════════╝");
        r.print(x + 7, y + 13, border,   "A B C D E F G H");

        // TODO: fen parser
        // r1bqkb1r/ppp1pppp/2n2n2/3p4/4P3/3P1P2/PPP3PP/RNBQKBNR w KQkq - 1 4
        for (y2, row) in fen.split(' ').next().unwrap().split('/').enumerate() {
            let row = row
                .replace("8", "········")
                .replace("7", "·······")
                .replace("6", "······")
                .replace("5", "·····")
                .replace("4", "····")
                .replace("3", "···")
                .replace("2", "··")
                .replace("1", "·");
            r.print(x + 3, 4 + y + y2, border, &format!("{} ║", 9-(y2+1)));
            r.print(x + 23, 4 + y + y2, border, "║");
            for (x2, char) in row.chars().enumerate() {
                let color = if char == '·' {
                    if (y2 + x2) % 2 == 0 {
                        space_light
                    } else {
                        space_dark
                    }
                } else {
                    if char.is_uppercase() {
                        piece_light
                    } else {
                        piece_dark
                    }
                };
                r.print(7 + x + x2*2, 4 + y + y2, color, &char.to_uppercase().collect::<String>());
            }
        }
    }
}

impl MenuOption {
    pub fn name(&self) -> &str {
        match self {
            &MenuOption::WatchTv {ref name, ref url} => { // TODO remove url
               name
            },
        }
    }

    pub fn execute(&self) -> MenuResult {
        match self {
            &MenuOption::WatchTv {ref name, ref url} => {
                MenuResult::AddGameView { name: name.clone(), url: url.clone() }
            },
            //_ => { MenuResult::None }
        }
    }
}
