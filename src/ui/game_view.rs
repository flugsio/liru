use rustbox::Key;

use game;
use super::MenuResult;
use super::Renderer;
use super::View;

pub struct GameView {
    pub name: String,
    #[allow(dead_code)]
    pub url: String,
    pub pov: game::ConnectedPov,
    pub input: Vec<char>,
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

    fn name(&self) -> &str {
        &self.name
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
