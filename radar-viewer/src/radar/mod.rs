use macroquad::{prelude::{Color, WHITE}, shapes::draw_poly_lines, text::draw_text};

use crate::{sector::draw::Draw, AircraftRecord};

pub mod display;
pub mod line;
pub mod position_calc;

pub const WINDOW_HT_N_MI: f32 = 70.0;

//      Artcc,
//     ArtccLow,
//     ArtccHigh,
//     AirwayLow,
//     AirwayHigh,
//     Sid,
//     Star,
//     Geo,

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




impl Draw for AircraftRecord {
    fn draw(&mut self, position_calculator: &position_calc::PositionCalculator, drawable_object_type: crate::sector::draw::DrawableObjectType) {
        self.position.cache_screen_coords(position_calculator);
        draw_poly_lines(
            self.position.cached_x,
            self.position.cached_y,
            4,
            5.0,
            45.0,
            1.0,
            WHITE,
        );
        draw_text(&self.callsign, self.position.cached_x, self.position.cached_y + 20.0, 20.0, WHITE);
        draw_text(&self.alt.to_string(), self.position.cached_x, self.position.cached_y + 35.0, 20.0, WHITE);
    }
}