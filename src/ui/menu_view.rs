/// A view that displays multiple options
/// that the user can choose between

use rustbox::Color;
use rustbox::Key;
use rustbox::{RB_BOLD, RB_NORMAL};
use rustbox;

use super::MenuOption;
use super::MenuResult;
use super::RBStyle;
use super::Renderer;
use super::View;

use lila;

pub struct MenuView {
    pub menu_options: Vec<MenuOption>,
    pub current: usize,
    name: String,
}

impl MenuView {
    pub fn new_playing(games: &Vec<lila::PlayingGame>) -> MenuView {
        let mut menu_options = Vec::new();
        for game in games {
            menu_options.push(MenuOption::WatchTv {
                name: game.opponent.username.clone(),
                url: game.fullId.clone()
            });
        }

        MenuView {
            name: "Playing".to_string(),
            menu_options: menu_options,
            current: 0,
        }
    }

    pub fn new_tv() -> MenuView {
        let mut menu_options = Vec::new();
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

        MenuView {
            name: "Playing".to_string(),
            menu_options: menu_options,
            current: 0,
        }
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

    fn name(&self) -> String {
        self.name.to_owned()
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

