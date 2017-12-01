extern crate rustbox;

mod view;
mod game_view;
mod menu_view;
mod renderer;
mod menu_result;
mod rb_style;

use std::default::Default;

use rustbox::{Color, RustBox};
use rustbox::Key;
use rustbox::RB_NORMAL;

use std::time::Duration;

use lila;

pub use ui::game_view::GameView;
pub use ui::menu_view::MenuView;
pub use ui::menu_result::MenuResult;
pub use ui::renderer::Renderer;
pub use ui::view::View;
pub use ui::rb_style::RBStyle;

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
