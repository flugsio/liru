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
    pub fn new(pov: Arc<Mutex<game::Pov>>) -> UI {

        let rb = match RustBox::init(Default::default()) {
            Result::Ok(v) => v,
            Result::Err(e) => panic!("{}", e),
        };

        let mut povs = Vec::new();
        povs.push(pov);

        return UI {
            running: true,
            rb: rb,
            povs: povs,
        };
    }

    pub fn start(&mut self) {
        while self.running {
            self.render_fen();
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

    pub fn render_fen(&mut self) {
        let pov = self.povs[0].lock().unwrap();
        let fen = pov.game.fen.clone();

        let border = RBStyle { style: RB_NORMAL, fg: Color::Cyan, bg: Color::Black };
        let piece_dark = RBStyle { style: RB_BOLD, fg: Color::Blue, bg: Color::Black };
        let piece_light = RBStyle { style: RB_NORMAL, fg: Color::Yellow, bg: Color::Black };
        let space_dark = RBStyle { style: RB_BOLD, fg: Color::Blue, bg: Color::Black };
        let space_light = RBStyle { style: RB_NORMAL, fg: Color::Yellow, bg: Color::Black };

        self.rb.print(3,  1, RB_BOLD, Color::White, Color::Black, &fen);
        self.rb.print(5,  3, border.style, border.fg, border.bg, "╔═════════════════╗");
        self.rb.print(5, 12, border.style, border.fg, border.bg, "╚═════════════════╝");

        // draw clocks
        match pov.clock.as_ref() {
            Some(clock) => {
                self.rb.print(18,  2, RB_BOLD, if clock.black < 10f64 { Color::Red } else { Color::White }, Color::Black, &format!(" {:04.1}  ", clock.black));
                self.rb.print(18,  13, RB_BOLD, if clock.white < 10f64 { Color::Red } else { Color::White }, Color::Black, &format!(" {:04.1}  ", clock.white));
            },
            None => ()
        }
        // TODO: fen parser
        // r1bqkb1r/ppp1pppp/2n2n2/3p4/4P3/3P1P2/PPP3PP/RNBQKBNR w KQkq - 1 4
        for (y, row) in fen.split(' ').next().unwrap().split('/').enumerate() {
            let row = row
                .replace("8", "········")
                .replace("7", "·······")
                .replace("6", "······")
                .replace("5", "·····")
                .replace("4", "····")
                .replace("3", "···")
                .replace("2", "··")
                .replace("1", "·");
            self.rb.print(3, 4 + y, border.style, border.fg, border.bg, &format!("{} ║", 9-(y+1)));
            self.rb.print(23, 4 + y, border.style, border.fg, border.bg, "║");
            for (x, char) in row.chars().enumerate() {
                let color = if char == '·' {
                    if (y + x) % 2 == 0 {
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
                self.rb.print(7 + x*2, 4 + y, color.style, color.fg, color.bg, &char.to_uppercase().collect::<String>());
            }
        }
        self.rb.present();
    }
}
