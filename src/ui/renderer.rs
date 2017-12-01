use rustbox::RustBox;

use super::RBStyle;

pub struct Renderer {
    pub rb: RustBox,
}

impl Renderer {
    pub fn print(&self, x: usize, y: usize, rbstyle: RBStyle, chars: &str) {
        self.rb.print(x, y, rbstyle.style, rbstyle.fg, rbstyle.bg, chars);
    }

    pub fn clear(&self) {
        self.rb.clear();
    }

    pub fn present(&self) {
        self.rb.present();
    }
}

