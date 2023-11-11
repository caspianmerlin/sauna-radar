use macroquad::{prelude::Color, window, shapes::draw_line};

use super::{DEFAULT_ARTCC_COLOUR, DEFAULT_ARTCC_LOW_COLOUR, DEFAULT_ARTCC_HIGH_COLOUR, DEFAULT_AIRWAY_HIGH_COLOUR, DEFAULT_AIRWAY_LOW_COLOUR, DEFAULT_SID_COLOUR, DEFAULT_STAR_COLOUR, DEFAULT_GEO_COLOUR, DEFAULT_FIX_COLOUR};




pub struct Line {
    pub start_x: f32,
    pub start_y: f32,

    pub end_x: f32,
    pub end_y: f32,
    pub colour: Color,
}
impl Line {
    pub fn draw(&self) {
        if (self.start_x < window::screen_width() && self.start_y < window::screen_height())
            || (self.end_x < window::screen_width() && self.end_y < window::screen_height())
        {
            draw_line(
                self.start_x,
                self.start_y,
                self.end_x,
                self.end_y,
                1.0,
                self.colour,
            );
        }
    }
}

pub enum LineType {
    Artcc,
    ArtccLow,
    ArtccHigh,
    AirwayLow,
    AirwayHigh,
    Sid,
    Star,
    Geo,
    Fix,
}
impl LineType {
    pub fn default_colour(&self) -> Color {
        match self {
            LineType::Artcc => DEFAULT_ARTCC_COLOUR,
            LineType::ArtccLow => DEFAULT_ARTCC_LOW_COLOUR,
            LineType::ArtccHigh => DEFAULT_ARTCC_HIGH_COLOUR,
            LineType::AirwayLow => DEFAULT_AIRWAY_LOW_COLOUR,
            LineType::AirwayHigh => DEFAULT_AIRWAY_HIGH_COLOUR,
            LineType::Sid => DEFAULT_SID_COLOUR,
            LineType::Star => DEFAULT_STAR_COLOUR,
            LineType::Geo => DEFAULT_GEO_COLOUR,
            LineType::Fix => DEFAULT_FIX_COLOUR,
        }
    }
}