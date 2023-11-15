use ipc::SimAircraftFmsLine;
use macroquad::{shapes::{draw_poly_lines, draw_line}, color::{WHITE, Color}, text::{load_ttf_font_from_bytes, TextParams, draw_text_ex}};

use crate::sector::{items::Position, draw::{Draw, DrawableObjectType}};

use super::{position_calc, display::TAG_FONT};

#[derive(Debug)]
pub struct AircraftRecord {
    pub callsign: String,
    pub position: Position,
    pub alt: i32,
    pub fms_lines: Vec<FmsLine>,
}
impl From<ipc::SimAircraftRecord> for AircraftRecord {
    fn from(value: ipc::SimAircraftRecord) -> Self {
        AircraftRecord { callsign: value.callsign, position: Position { lat: value.lat, lon: value.lon, cached_x: 0.0, cached_y: 0.0 }, alt: value.alt, fms_lines: value.fms_lines.into_iter().map(FmsLine::from).collect(), }
    }
}

impl AircraftRecord {
    pub fn draw(&mut self, position_calculator: &position_calc::PositionCalculator, show_fms_lines: bool) {
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

        if show_fms_lines {
            self.fms_lines.iter_mut().for_each(|fms_line| fms_line.draw(position_calculator, Color::new(0.6352941176470588, 0.196078431372549, 0.6588235294117647, 1.0)));
        }


        let font = TAG_FONT.get_or_init(|| {
            load_ttf_font_from_bytes(include_bytes!("../../fonts/RobotoMono-Regular.ttf")).unwrap()
        });
        let text_params = TextParams {
            font: Some(font),
            font_size: 16,
            font_scale: 1.0,
            color: WHITE,
            ..Default::default()
        };

        draw_text_ex(&self.callsign, self.position.cached_x, self.position.cached_y + 20.0, text_params.clone());
        draw_text_ex(&self.alt.to_string(), self.position.cached_x, self.position.cached_y + 35.0, text_params);
    }
}


#[derive(Debug, PartialEq)]
pub struct FmsLine {
    pub start: Position,
    pub end: Position,
}

impl From<SimAircraftFmsLine> for FmsLine {
    fn from(value: SimAircraftFmsLine) -> Self {
        let start = Position::new(value.start_lat, value.start_lon);
        let end = Position::new(value.end_lat, value.end_lon);
        FmsLine {
            start,
            end,
        }
    }
}

impl Draw for FmsLine {
    fn draw(&mut self, position_calculator: &position_calc::PositionCalculator, default_colour: Color) {
        if position_calculator.invalidated {
            self.start.cache_screen_coords(position_calculator);
            self.end.cache_screen_coords(position_calculator);
        }
        draw_line(self.start.cached_x, self.start.cached_y, self.end.cached_x, self.end.cached_y, 1.0, default_colour);
    }
}