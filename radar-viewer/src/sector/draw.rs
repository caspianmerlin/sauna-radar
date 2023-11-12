use macroquad::prelude::Color;

use crate::radar::position_calc::PositionCalculator;

pub const DEFAULT_ARTCC_COLOUR: Color =
    Color::new(0.4705882352941176, 0.4196078431372549, 0.2, 1.0);
pub const DEFAULT_ARTCC_LOW_COLOUR: Color = Color::new(
    0.1490196078431373,
    0.3686274509803922,
    0.3803921568627451,
    1.0,
);
pub const DEFAULT_ARTCC_HIGH_COLOUR: Color = Color::new(
    0.1490196078431373,
    0.3686274509803922,
    0.3803921568627451,
    1.0,
);
pub const DEFAULT_AIRWAY_LOW_COLOUR: Color = Color::new(0.3490196078431373, 0., 0., 1.0);
pub const DEFAULT_AIRWAY_HIGH_COLOUR: Color = Color::new(0.3490196078431373, 0., 0., 1.0);
pub const DEFAULT_SID_COLOUR: Color = Color::new(
    0.2705882352941176,
    0.3058823529411765,
    0.3450980392156863,
    1.0,
);
pub const DEFAULT_STAR_COLOUR: Color = Color::new(0.4705882352941176, 0.4196078431372549, 0.2, 1.0);
pub const DEFAULT_GEO_COLOUR: Color = Color::new(0., 0.5019607843137255, 0.2509803921568627, 1.0);
pub const DEFAULT_FIX_COLOUR: Color = Color::new(
    0.1490196078431373,
    0.3686274509803922,
    0.3803921568627451,
    1.0,
);



pub trait Draw {
    fn draw(&mut self, position_calculator: &PositionCalculator, drawable_object_type: DrawableObjectType);
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
}
impl DrawableObjectType {
    pub fn default_colour(&self) -> Color {
        match self {
            Self::Default => Color::new(1.0, 1.0, 1.0, 1.0),
            Self::Airport => DEFAULT_FIX_COLOUR,
            Self::Fix => DEFAULT_FIX_COLOUR,
            Self::Vor => DEFAULT_FIX_COLOUR,
            Self::Ndb => DEFAULT_FIX_COLOUR,
            Self::Artcc => DEFAULT_ARTCC_COLOUR,
            Self::ArtccLow => DEFAULT_AIRWAY_LOW_COLOUR,
            Self::ArtccHigh => DEFAULT_ARTCC_HIGH_COLOUR,
            Self::LowAirway => DEFAULT_AIRWAY_LOW_COLOUR,
            Self::HighAirway => DEFAULT_AIRWAY_HIGH_COLOUR,
            Self::Sid => DEFAULT_SID_COLOUR,
            Self::Star => DEFAULT_STAR_COLOUR,
            Self::Geo => DEFAULT_GEO_COLOUR,
        }
    }
}