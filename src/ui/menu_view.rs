use rustbox;
use rustbox::Color;
use rustbox::Key;
use rustbox::{RB_BOLD, RB_NORMAL};

use super::MenuOption;
use super::MenuResult;
use super::View;
use super::RBStyle;
use super::Renderer;

pub struct MenuView {
    pub menu_options: Vec<MenuOption>,
    pub current: usize,
}

impl MenuView {
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

