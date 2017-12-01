extern crate rustbox;

mod view;
mod game_view;
mod renderer;
mod menu_result;
mod rb_style;

use std::default::Default;

use rustbox::{Color, RustBox};
use rustbox::Key;
use rustbox::{RB_BOLD, RB_NORMAL};

use std::time::Duration;

use game;
use lila;

pub use ui::game_view::GameView;
pub use ui::menu_result::MenuResult;
pub use ui::renderer::Renderer;
pub use ui::view::View;
pub use ui::rb_style::RBStyle;

struct MenuView {
    menu_options: Vec<MenuOption>,
    current: usize,
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

impl UI {
    pub fn new(session: lila::Session) -> UI {

        let rb = match RustBox::init(Default::default()) {
            Result::Ok(v) => v,
            Result::Err(e) => panic!("{}", e),
        };

        let mut views = Vec::new();
        let mut menu_options = Vec::new();

        for game in &session.user.nowPlaying {
            menu_options.push(MenuOption::WatchTv { name: game.opponent.username.clone(), url: game.fullId.clone() });
        }

        menu_options.push(MenuOption::WatchTv { name: "Best".into(), url: "tv/best".into() });
        menu_options.push(MenuOption::WatchTv { name: "Bullet".into(), url: "tv/bullet".into() });
        menu_options.push(MenuOption::WatchTv { name: "Blitz".into(), url: "tv/blitz".into() });
        menu_options.push(MenuOption::WatchTv { name: "Classical".into(), url: "tv/classical".into() });
        menu_options.push(MenuOption::WatchTv { name: "Crazyhouse".into(), url: "tv/crazyhouse".into() });
        menu_options.push(MenuOption::WatchTv { name: "Chess 960".into(), url: "tv/chess960".into() });
        menu_options.push(MenuOption::WatchTv { name: "King of the Hill".into(), url: "tv/kingOfTheHill".into() });
        menu_options.push(MenuOption::WatchTv { name: "Three-Check".into(), url: "tv/threeCheck".into() });
        menu_options.push(MenuOption::WatchTv { name: "Antichess".into(), url: "tv/antichess".into() });
        menu_options.push(MenuOption::WatchTv { name: "Atomic".into(), url: "tv/atomic".into() });
        menu_options.push(MenuOption::WatchTv { name: "Horde".into(), url: "tv/horde".into() });
        menu_options.push(MenuOption::WatchTv { name: "Racing Kings".into(), url: "tv/racingKings".into() });
        menu_options.push(MenuOption::WatchTv { name: "Computer".into(), url: "tv/computer".into() });

        views.push(Box::new(MenuView {
            menu_options: menu_options,
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

    fn add_view(&mut self, view: Box<View>) {
        self.views.push(view);
    }

    fn add_game(&mut self, name: String, url: String) {
        let game = GameView::new(&self.session, name, url);
        self.add_view(Box::new(game) as Box<View>);
    }

    pub fn start(&mut self) {
        while self.running {
            self.tick();
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
        match self.renderer.rb.peek_event(Duration::from_millis(100), false) {
            Ok(rustbox::Event::KeyEvent(key)) => {
                match key {
                    Key::Char('q') => self.running = false,
                    Key::Left => self.prev_view(),
                    Key::Right => self.next_view(),
                    key => {
                        match self.current_view().key_event(key) {
                            MenuResult::AddGameView { name, url } => {
                                self.add_game(name, url);
                            }
                            MenuResult::None => {},
                        }
                    }
                }
            },
            Err(e) => panic!("{}", e),
            _ => { }
        }
    }

    pub fn tick(&mut self) {
        self.views.get_mut(self.current_view).unwrap().tick();
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

impl View for MenuView {
    fn tick(&mut self) {
    }

    fn render(&self, r: &mut Renderer) {
        let dark  = RBStyle { style: RB_BOLD, fg: Color::Blue, bg:     Color::Black };
        let light = RBStyle { style: RB_NORMAL, fg: Color::Yellow, bg: Color::Black };

        for (i, option) in self.menu_options.iter().enumerate() {
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
                if self.current < self.menu_options.len() -1 {
                    self.current += 1;
                }
                MenuResult::None
            },
            Key::Enter => {
                let option = self.menu_options.get(self.current).unwrap(); // TODO
                option.execute()
            }
            _ => { MenuResult::None }
        }
    }
}


impl MenuView {
}

impl GameView {
    pub fn new(session: &lila::Session, name: String, url: String) -> GameView {
        let connected_pov = game::ConnectedPov::new(session, &url);

        return GameView {
            name: name,
            url: url,
            pov: connected_pov,
            input: vec!(),
        };
    }

    fn handle_input(&mut self) {
        if self.input.len() == 4 { // assume move for now
            let from: String = self.input[0..2].iter().cloned().collect();
            let to: String = self.input[2..4].iter().cloned().collect();
            self.pov.send_move(from, to);
            self.input.clear();
        }
    }

    pub fn render_latency(&self, r: &mut Renderer, x: usize, y: usize, latency: &game::LatencyRecorder) {
        let style = RBStyle { style: RB_NORMAL, fg: Color::Cyan, bg: Color::Black };
        r.print(x, y+13, style, &format!("{:3}ms", latency.average()));
    }

    pub fn render_pov(&self, r: &mut Renderer, x: usize, y: usize, pov: &game::Pov) {
        self.render_player(r, x + 1, y + 2, &pov.opponent);
        self.render_player(r, x + 1, y + 14, &pov.player);
        let fen = pov.game.fen.clone();
        self.render_fen(r, x, y, fen, pov.orientation() == game::Color::white);
        match pov.clock {
            Some(ref clock) => {
                self.render_clock(r, x + 19, y + 3, clock.from(!pov.orientation()));
                self.render_clock(r, x + 19, y + 12, clock.from(pov.orientation()));
            },
            None => ()
        };
        if pov.movable() {
            let style = RBStyle { style: RB_BOLD, fg: Color::White, bg: Color::Black };
            r.print(5, 16, style, &format!("Move {}▍          ", self.input.iter().cloned().collect::<String>()));
        }
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

    pub fn render_fen(&self, r: &mut Renderer, x: usize, y: usize, fen: String, orientation: bool) {
        let _text_style  = RBStyle { style: RB_BOLD, fg: Color::White, bg:    Color::Black };
        let border      = RBStyle { style: RB_NORMAL, fg: Color::Cyan, bg:   Color::Black };
        let piece_dark  = RBStyle { style: RB_BOLD, fg: Color::Blue, bg:     Color::Black };
        let piece_light = RBStyle { style: RB_NORMAL, fg: Color::Yellow, bg: Color::Black };
        let space_dark  = RBStyle { style: RB_BOLD, fg: Color::Blue, bg:     Color::Black };
        let space_light = RBStyle { style: RB_NORMAL, fg: Color::Yellow, bg: Color::Black };

        //r.print(x + 3, y +  1, text_style, &fen);
        r.print(x + 5, y +  3, border, "╔═════════════════╗");
        r.print(x + 5, y + 12, border, "╚═════════════════╝");
        if orientation {
            r.print(x + 7, y + 13, border,   "a b c d e f g h");
        } else {
            r.print(x + 7, y + 13, border,   "h g f e d c b a");
        }

        // TODO: fen parser
        // r1bqkb1r/ppp1pppp/2n2n2/3p4/4P3/3P1P2/PPP3PP/RNBQKBNR w KQkq - 1 4
        let mut fen = fen.split(' ').next().unwrap().to_string();
        if !orientation {
            fen = fen.chars().rev().collect();
        }
        for (y2, row) in fen.split('/').enumerate() {
            if y2 >= 8 {
                break; // temp workaround for crazyhouse
            }
            let row = row
                .replace("8", "········")
                .replace("7", "·······")
                .replace("6", "······")
                .replace("5", "·····")
                .replace("4", "····")
                .replace("3", "···")
                .replace("2", "··")
                .replace("1", "·");
            if orientation {
                r.print(x + 3, 4 + y + y2, border, &format!("{} ║", 9-(y2+1)));
            } else {
                r.print(x + 3, 4 + y + y2, border, &format!("{} ║", y2+1));
            }
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
            &MenuOption::WatchTv {ref name, ..} => { // TODO remove url
               name
            },
        }
    }

    fn execute(&self) -> MenuResult {
        match self {
            &MenuOption::WatchTv {ref name, ref url} => {
                MenuResult::AddGameView { name: name.clone(), url: url.clone() }
            },
            //_ => { MenuResult::None }
        }
    }
}
