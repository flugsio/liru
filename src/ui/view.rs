use rustbox;

use super::Renderer;
use super::MenuResult;

pub trait View {
    fn tick(&mut self);
    fn name(&self) -> &str;
    fn render(&self, ui: &mut Renderer);
    fn key_event(&mut self, key: rustbox::keyboard::Key) -> MenuResult;
}
