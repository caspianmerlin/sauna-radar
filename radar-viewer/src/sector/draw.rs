use macroquad::prelude::Color;

use crate::radar::position_calc::PositionCalculator;


pub const DEFAULT_FMS_LINE_COLOUR: Color = Color::new(0.6352941176470588, 0.196078431372549, 0.6588235294117647, 1.0);



pub trait Draw {
    fn draw(&mut self, position_calculator: &PositionCalculator, default_colour: Color);
}

pub enum DrawableObjectType {
    Default,
    Airport,
    Fix,
    Vor,
    Ndb,
    Artcc,
    ArtccLow,
    ArtccHigh,
    LowAirway,
    HighAirway,
    Sid,
    Star,
    Geo,
    FmsLine,
}
