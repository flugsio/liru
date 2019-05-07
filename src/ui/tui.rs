use rustbox;

use std::default::Default;

use rustbox::Key;
use rustbox::RB_NORMAL;
use rustbox::{Color, RustBox};

use std::time::Duration;

use crate::lila;

use super::GameView;
use super::MenuResult;
use super::MenuView;
use super::RBStyle;
use super::Renderer;
use super::View;

pub struct TUI {
    running: bool,
    renderer: Renderer,
    views: Vec<Box<dyn View>>,
    current_view: usize,
    session: lila::Session,
}

impl TUI {
    pub fn new(session: lila::Session) -> TUI {

        let rb = match RustBox::init(Default::default()) {
            Result::Ok(v) => v,
            Result::Err(e) => panic!("{}", e),
        };

        let mut views = Vec::<Box<dyn View>>::new();

        views.push(Box::new(MenuView::new_tv()));
        views.push(Box::new(MenuView::new_playing(&session.user.nowPlaying)));

        return TUI {
            running: true,
            renderer: Renderer { rb: rb },
            views: views,
            current_view: 0,
            session: session,
        };
    }

    fn add_view(&mut self, view: Box<dyn View>) {
        self.views.push(view);
    }

    fn add_game(&mut self, name: String, url: String) {
        let game = GameView::new(&self.session, name, url);
        self.add_view(Box::new(game) as Box<dyn View>);
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

    fn current_view(&mut self) -> &mut Box<dyn View> {
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
                self.renderer.print(x, 0, light, &view.name());
            } else {
                self.renderer.print(x, 0, dark, &view.name());
            }
            x += view.name().len() + 3;
            self.renderer.print(x- 2, 0, dark, "|");
        }

        self.views.get(self.current_view).unwrap().render(&mut self.renderer);
        self.renderer.present();
    }
}
