use time;

use rustc_serialize::Decoder;
use rustc_serialize::Decodable;

use super::Color;

// TODO: ehm, maybe not like this
// wrap to implement trait on foreign type in this version of rust
pub struct Time(time::Tm);
impl Decodable for Time {
    fn decode<D: Decoder>(_d: &mut D) -> Result<Time, D::Error> {
        Ok(Time(time::now_utc()))
    }
}


#[derive(RustcDecodable)]
pub struct Clock {
    pub white: f64,
    pub black: f64,
    last_update: Time,
}

impl Clock {
    pub fn from(&self, color: Color) -> f64 {
        match color {
            Color::white => self.white,
            Color::black => self.black,
        }
    }
    
    //pub fn update(&mut self, white: f64, black: f64) {
    //    self.last_update = Time(time::now_utc());
    //    self.white = white;
    //    self.black = black;
    //}
    
    pub fn tick(&mut self, color: Color) {
        let now = time::now_utc();
        let Time(updated) = self.last_update;
        let passed = ((now - updated).num_milliseconds() as f64) / 1000.0;
        self.last_update = Time(now);
        match color {
            // TODO: only tick when gameIsRunning instead of using max
            Color::white => self.white = (self.white - passed).max(0.0),
            Color::black => self.black = (self.black - passed).max(0.0),
        };
    }
}
