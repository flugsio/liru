use rustbox::Color;
use rustbox::Key;
use rustbox::{RB_BOLD, RB_NORMAL};

use crate::game;
use crate::lila;

use super::MenuResult;
use super::RBStyle;
use super::Renderer;
use super::View;

pub struct GameView {
    name: String,
    pub pov: game::ConnectedPov,
    pub input: Vec<char>,
}

impl GameView {
    pub fn new(session: &lila::Session, name: String, url: String) -> GameView {
        let connected_pov = game::ConnectedPov::new(session, &url);

        return GameView {
            name: name,
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
        if let Some(crowd) = &pov.crowd {
            let opponent_present = crowd.opponent_from(pov.orientation());
            let player_present = crowd.player_from(pov.orientation());
            self.render_player(r, x + 1, y + 2, &pov.opponent, opponent_present);
            self.render_player(r, x + 1, y + 14, &pov.player, player_present);
            self.render_watchers(r, x + 1, y + 18, &crowd);
        } else {
            self.render_player(r, x + 1, y + 2, &pov.opponent, true);
            self.render_player(r, x + 1, y + 14, &pov.player, true);
        }
        let fen = pov.game.fen.clone();
        self.render_fen(r, x, y, fen, pov.orientation() == game::Color::white, &pov.game);
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
        self.render_last_move(r, x + 26, y + 12, &pov.game);
    }

    pub fn render_player(&self, r: &mut Renderer, x: usize, y: usize, player: &game::Player, present: bool) {
        let style = RBStyle { style: RB_BOLD, fg: Color::White, bg: Color::Black };
        let style_absent = RBStyle { style: RB_BOLD, fg: Color::Red, bg: Color::Black };
        r.print(x, y, style, &format!("{:4}", player.rating.unwrap_or(1500)));
        match player.user {
            Some(ref user) => {
                r.print(x + 5, y,  if present { style } else { style_absent }, &format!("{}", user.username));
            },
            None => ()
        };
    }

    pub fn render_watchers(&self, r: &mut Renderer, x: usize, y: usize, crowd: &game::Crowd) {
        let style = RBStyle { style: RB_BOLD, fg: Color::White, bg: Color::Black };
        r.print(x, y, style, &format!("Watchers: {:4}", crowd.watchers.nb));
        if let Some(anons) = &crowd.watchers.anons {
            r.print(x, y + 1, style, &format!("Anonymous: {:3}", anons));
        }
        if let Some(users) = &crowd.watchers.users {
            for (i, user) in users.iter().enumerate() {
                r.print(x, y + 2 + i, style, &format!("{}", user));
            }
        }

    }

    pub fn render_last_move(&self, r: &mut Renderer, x: usize, y: usize, game: &game::Game) {
        let color = if game.player == game::Color::black {
            RBStyle { style: RB_NORMAL, fg: Color::Yellow, bg: Color::Black }
        } else {
            RBStyle { style: RB_BOLD, fg: Color::Blue, bg: Color::Black }
        };
        r.print(x, y, color, &format!("{}           ", game.lastMoveSan.as_ref().or(game.lastMove.as_ref()).unwrap_or(&"".to_string())));
    }

    pub fn render_clock(&self, r: &mut Renderer, x: usize, y: usize, time: f64) {
        r.print(x, y, RBStyle { style: RB_BOLD, fg: if time < 10f64 { Color::Red } else { Color::White }, bg: Color::Black }, &format!("{:04.1}", time));
    }

    fn highlighed(uci: &Option<String>, x: usize, y: usize) -> bool {
        if let Some(uci) = uci {
            let uci = uci
                .replace("a", "1")
                .replace("b", "2")
                .replace("c", "3")
                .replace("d", "4")
                .replace("e", "5")
                .replace("f", "6")
                .replace("g", "7")
                .replace("h", "8");
            if uci[0..2] == format!("{}{}", x, y) || uci[2..4] == format!("{}{}", x, y) {
                return true;
            }
        }
        false
    }

    pub fn render_fen(&self, r: &mut Renderer, x: usize, y: usize, fen: String, orientation: bool, game: &game::Game) {
        let _text_style  = RBStyle { style: RB_BOLD, fg: Color::White, bg:    Color::Black };
        let border      = RBStyle { style: RB_NORMAL, fg: Color::Cyan, bg:   Color::Black };

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
                let (uci_x, uci_y) = if orientation {
                    (x2 + 1, 9 - (y2 + 1))
                } else {
                    (9 - (x2 + 1), y2 + 1)
                };
                let bg = if Self::highlighed(&game.lastMove, uci_x, uci_y) {
                    Color::Magenta
                } else {
                    Color::Black
                };
                let color = if char == '·' {
                    if (y2 + x2) % 2 == 0 {
                        RBStyle { style: RB_NORMAL, fg: Color::Yellow, bg: bg }
                    } else {
                        RBStyle { style: RB_BOLD, fg: Color::Blue, bg: bg }
                    }
                } else {
                    if char.is_uppercase() {
                        RBStyle { style: RB_NORMAL, fg: Color::Yellow, bg: bg }
                    } else {
                        RBStyle { style: RB_BOLD, fg: Color::Blue, bg: bg }
                    }
                };
                r.print(7 + x + x2*2, 4 + y + y2, color, &char.to_uppercase().collect::<String>());
            }
        }
    }
}

impl View for GameView {
    fn tick(&mut self) {
        self.pov.pov.lock().ok().map(|mut p| {
            p.tick();
        });
    }

    fn render(&self, r: &mut Renderer) {
        self.pov.latency.lock().ok().map(|l| self.render_latency(r, 0, 0, &l));
        self.pov.pov.lock().ok().map(|p| self.render_pov(r, 0, 0, &p));
    }

    fn name(&self) -> String {
        self.name.to_owned()
    }

    fn key_event(&mut self, key: Key) -> MenuResult {
        match key {
            Key::Enter => {
                self.handle_input();
            }
            Key::Char(x) => {
                self.input.push(x);
            }
            Key::Backspace => {
                self.input.pop();
            }
            _ => ()
        }
        MenuResult::None
    }
}
