use rustbox;

#[derive(Clone, Copy)]
pub struct RBStyle {
    pub style: rustbox::Style,
    pub fg: rustbox::Color,
    pub bg: rustbox::Color,
}

