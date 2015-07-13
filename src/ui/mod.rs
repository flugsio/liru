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

pub struct UI {
    running: bool,
    rb: RustBox,
    povs: Vec<Arc<Mutex<game::Pov>>>
}

impl UI {
    pub fn new() -> UI {

        let rb = match RustBox::init(Default::default()) {
            Result::Ok(v) => v,
            Result::Err(e) => panic!("{}", e),
        };

        let mut povs = Vec::new();

        return UI {
            running: true,
            rb: rb,
            povs: povs,
        };
    }

    pub fn add_game(&mut self, pov: Arc<Mutex<game::Pov>>) {
        self.povs.push(pov);
    }

    pub fn start(&mut self) {
        while self.running {
            self.render();
            self.poll_keys();
        }
    }

    pub fn poll_keys(&mut self) {
        match self.rb.peek_event(Duration::milliseconds(100), false) {
            Ok(rustbox::Event::KeyEvent(key)) => {
                match key {
                    Some(Key::Char('q')) => self.running = false,
                    _ => { }
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
        for (i, pov) in self.povs.iter().enumerate() {
            pov.lock().map(|p| {
                self.render_pov(i * 30, 0, &p);
            });
        }
        self.rb.present();
    }

    pub fn render_pov(&self, x: usize, y: usize, pov: &game::Pov) {
        self.render_player(x + 1, y + 2, &pov.opponent);
        self.render_player(x + 1, y + 14, &pov.player);
        let fen = pov.game.fen.clone();
        self.render_fen(x, y, fen);
        match pov.clock {
            Some(ref clock) => {
                self.render_clock(x + 19, y + 3, clock.black);
                self.render_clock(x + 19, y + 12, clock.white);
            },
            None => ()
        };
    }

    pub fn render_player(&self, x: usize, y: usize, player: &game::Player) {
        let style = RBStyle { style: RB_BOLD, fg: Color::White, bg: Color::Black };
        self.print(x, y, style, &format!("{:4}", player.rating.unwrap_or(1500)));
        match player.user {
            Some(ref user) => {
                self.print(x + 5, y, style, &format!("{}", user.username));
            },
            None => ()
        };
    }

    pub fn render_clock(&self, x: usize, y: usize, time: f64) {
        self.print(x, y, RBStyle { style: RB_BOLD, fg: if time < 10f64 { Color::Red } else { Color::White }, bg: Color::Black }, &format!("{:04.1}", time));
    }

    pub fn render_fen(&self, x: usize, y: usize, fen: String) {
        let text_style  = RBStyle { style: RB_BOLD, fg: Color::White, bg:    Color::Black };
        let border      = RBStyle { style: RB_NORMAL, fg: Color::Cyan, bg:   Color::Black };
        let piece_dark  = RBStyle { style: RB_BOLD, fg: Color::Blue, bg:     Color::Black };
        let piece_light = RBStyle { style: RB_NORMAL, fg: Color::Yellow, bg: Color::Black };
        let space_dark  = RBStyle { style: RB_BOLD, fg: Color::Blue, bg:     Color::Black };
        let space_light = RBStyle { style: RB_NORMAL, fg: Color::Yellow, bg: Color::Black };

        self.print(x + 3, y +  1, text_style, &fen);
        self.print(x + 5, y +  3, border, "╔═════════════════╗");
        self.print(x + 5, y + 12, border, "╚═════════════════╝");
        self.print(x + 7, y + 13, border,   "A B C D E F G H");

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
            self.print(x + 3, 4 + y + y2, border, &format!("{} ║", 9-(y2+1)));
            self.print(x + 23, 4 + y + y2, border, "║");
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
                self.print(7 + x + x2*2, 4 + y + y2, color, &char.to_uppercase().collect::<String>());
            }
        }
    }
}
