use serde::{Serialize, Deserialize};




#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default, Serialize, Deserialize)]
pub struct Position {
    pub lat: f32,
    pub lon: f32,

    /// Indicated altitude. Display as FL if the aircraft has standard set.
    pub alt: f32,
}

impl From<sct_reader::position::Position<sct_reader::position::Valid>> for Position {
    fn from(value: sct_reader::position::Position<sct_reader::position::Valid>) -> Self {
        Position {
            lat: value.lat as f32,
            lon: value.lon as f32,
            alt: 0.0,
        }
    }
}
impl Position {
    pub fn new(lat: f32, lon: f32) -> Position {
        Position {
            lat,
            lon,
            alt: 0.0,
        }
    }
    pub fn new_with_alt(lat: f32, lon: f32, alt: f32) -> Position {
        Position { lat, lon, alt }
    }
}