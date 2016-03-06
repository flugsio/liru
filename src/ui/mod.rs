extern crate rustbox;

use std::sync::{Arc, Mutex};

use std::default::Default;

use rustbox::{Color, RustBox};
use rustbox::Key;
use rustbox::{RB_BOLD, RB_NORMAL};

use time::Duration;

use game;

#[derive(Clone, Copy)]
struct RBStyle {
    style: rustbox::Style,
    fg: Color,
    bg: Color,
}

trait View {
    fn name(&self) -> &str;
    fn render(&self, ui: &mut rustbox::RustBox);
    fn key_event(&mut self, key: rustbox::keyboard::Key);
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
    rb: RustBox,
    views: Vec<Box<View>>,
    current_view: usize,
}

impl UI {
    pub fn new() -> UI {

        let rb = match RustBox::init(Default::default()) {
            Result::Ok(v) => v,
            Result::Err(e) => panic!("{}", e),
        };

        let mut views = Vec::new();
        let mut menuOptions = Vec::new();
        menuOptions.push(MenuOption::WatchTv {
            name: "Bullet".to_string(),
            url: "tv/bullet".to_string(),
        });
        menuOptions.push(MenuOption::WatchTv {
            name: "Blitz".to_string(),
            url: "tv/blitz".to_string(),
        });

        views.push(Box::new(MenuView {
            menuOptions: menuOptions,
            current: 0,
        }) as Box<View>);

        return UI {
            running: true,
            rb: rb,
            views: views,
            current_view: 0,
        };
    }

    //pub fn add_view(&mut self, view: View) {
        //self.views.push(view);
    //}

    pub fn add_game(&mut self, pov: Arc<Mutex<game::Pov>>) {
        let mut povs = Vec::new();
        povs.push(pov);
        self.views.push(Box::new(GameView {
            name: "Bullet".to_string(),
            url: "".to_string(),
            povs: povs,
        }) as Box<View>);
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
            self.rb.clear();
        }
    }

    fn next_view(&mut self) {
        if self.current_view < self.views.len() - 1 {
            self.current_view += 1;
            self.rb.clear();
        }
    }

    fn current_view(&mut self) -> &mut Box<View> {
        self.views.get_mut(self.current_view).unwrap()
    }

    pub fn poll_keys(&mut self) {
        match self.rb.peek_event(Duration::milliseconds(100), false) {
            Ok(rustbox::Event::KeyEvent(key)) => {
                match key {
                    Some(Key::Char('q')) => self.running = false,
                    Some(Key::Left) => self.prev_view(),
                    Some(Key::Right) => self.next_view(),
                    Some(key) => { self.current_view().key_event(key); }
                    _ => {}
                }
            },
            Err(e) => panic!("{}", e),
            _ => { }
        }
    }

    pub fn print(&self, x: usize, y: usize, rbstyle: RBStyle, chars: &str) {
        self.rb.print(x, y, rbstyle.style, rbstyle.fg, rbstyle.bg, chars);
    }

    pub fn render(&mut self) {
        let dark  = RBStyle { style: RB_BOLD, fg: Color::Blue, bg:     Color::Black };
        let light = RBStyle { style: RB_NORMAL, fg: Color::Yellow, bg: Color::Black };
        for (i, view) in self.views.iter().enumerate() {
            if i == self.current_view {
                self.print(i*10, 0, light, view.name());
            } else {
                self.print(i*10, 0, dark, view.name());
            }
        }

        self.views.get(self.current_view).unwrap().render(&mut self.rb);
        self.rb.present();
    }

    // #todo
    //fn render_view(&mut self, view: &View) {
    //}

}

impl View for MenuView {
    fn render(&self, rb: &mut rustbox::RustBox) {
        let dark  = RBStyle { style: RB_BOLD, fg: Color::Blue, bg:     Color::Black };
        let light = RBStyle { style: RB_NORMAL, fg: Color::Yellow, bg: Color::Black };

        for (i, option) in self.menuOptions.iter().enumerate() {
            if i == self.current {
                self.print(rb, 0, 2 + i, light, option.name());
            } else {
                self.print(rb, 0, 2 + i, dark, option.name());
            }
        }
    }


    fn name(&self) -> &str {
        "Menu"
    }

    fn key_event(&mut self, key: rustbox::keyboard::Key) {
        match key {
            Key::Up => {
                if 0 < self.current {
                    self.current -= 1;
                }
            },
            Key::Down => {
                if self.current < self.menuOptions.len() -1 {
                    self.current += 1;
                }
            },
            Key::Enter => {
            }
            _ => {}
        }
    }
}

impl View for GameView {
    fn render(&self, rb: &mut rustbox::RustBox) {
        for (i, pov) in self.povs.iter().enumerate() {
            pov.lock().map(|p| {
                self.render_pov(rb, i * 30, 0, &p);
            });
        }
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn key_event(&mut self, key: rustbox::keyboard::Key) {
        // nothing yet
    }
}

impl MenuView {
    // TODO: where to put this, custom wrapper maybe
    fn print(&self, rb: &mut rustbox::RustBox, x: usize, y: usize, rbstyle: RBStyle, chars: &str) {
        rb.print(x, y, rbstyle.style, rbstyle.fg, rbstyle.bg, chars);
    }
}

impl GameView {
    fn print(&self, rb: &mut rustbox::RustBox, x: usize, y: usize, rbstyle: RBStyle, chars: &str) {
        rb.print(x, y, rbstyle.style, rbstyle.fg, rbstyle.bg, chars);
    }

    pub fn render_pov(&self, rb: &mut rustbox::RustBox, x: usize, y: usize, pov: &game::Pov) {
        self.render_player(rb, x + 1, y + 2, &pov.opponent);
        self.render_player(rb, x + 1, y + 14, &pov.player);
        let fen = pov.game.fen.clone();
        self.render_fen(rb, x, y, fen);
        match pov.clock {
            Some(ref clock) => {
                self.render_clock(rb, x + 19, y + 3, clock.black);
                self.render_clock(rb, x + 19, y + 12, clock.white);
            },
            None => ()
        };
    }

    pub fn render_player(&self, rb: &mut rustbox::RustBox, x: usize, y: usize, player: &game::Player) {
        let style = RBStyle { style: RB_BOLD, fg: Color::White, bg: Color::Black };
        self.print(rb, x, y, style, &format!("{:4}", player.rating.unwrap_or(1500)));
        match player.user {
            Some(ref user) => {
                self.print(rb, x + 5, y, style, &format!("{}", user.username));
            },
            None => ()
        };
    }

    pub fn render_clock(&self, rb: &mut rustbox::RustBox, x: usize, y: usize, time: f64) {
        self.print(rb, x, y, RBStyle { style: RB_BOLD, fg: if time < 10f64 { Color::Red } else { Color::White }, bg: Color::Black }, &format!("{:04.1}", time));
    }

    pub fn render_fen(&self, rb: &mut rustbox::RustBox, x: usize, y: usize, fen: String) {
        let text_style  = RBStyle { style: RB_BOLD, fg: Color::White, bg:    Color::Black };
        let border      = RBStyle { style: RB_NORMAL, fg: Color::Cyan, bg:   Color::Black };
        let piece_dark  = RBStyle { style: RB_BOLD, fg: Color::Blue, bg:     Color::Black };
        let piece_light = RBStyle { style: RB_NORMAL, fg: Color::Yellow, bg: Color::Black };
        let space_dark  = RBStyle { style: RB_BOLD, fg: Color::Blue, bg:     Color::Black };
        let space_light = RBStyle { style: RB_NORMAL, fg: Color::Yellow, bg: Color::Black };

        self.print(rb, x + 3, y +  1, text_style, &fen);
        self.print(rb, x + 5, y +  3, border, "╔═════════════════╗");
        self.print(rb, x + 5, y + 12, border, "╚═════════════════╝");
        self.print(rb, x + 7, y + 13, border,   "A B C D E F G H");

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
            self.print(rb, x + 3, 4 + y + y2, border, &format!("{} ║", 9-(y2+1)));
            self.print(rb, x + 23, 4 + y + y2, border, "║");
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
                self.print(rb, 7 + x + x2*2, 4 + y + y2, color, &char.to_uppercase().collect::<String>());
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

    pub fn execute(&self) {
    }
}
