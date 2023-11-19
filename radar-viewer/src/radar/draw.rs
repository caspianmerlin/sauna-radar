use std::f32::consts::PI;


use common::{aircraft_data::fms_graphics::{FmsArc, FmsGraphic, FmsLine, FmsArcState}, position::Position};
use macroquad::{shapes::{draw_poly_lines, draw_line}, color::{WHITE, Color}, text::{load_ttf_font_from_bytes, TextParams, draw_text_ex}};


use crate::{sector::draw::{Draw, DrawableObjectType}, aircraft::Aircraft};

use super::{position_calc::{self, PositionCalculator}, display::TAG_FONT};

pub trait DrawableAircraft {
    fn draw(&mut self, position_calculator: &position_calc::PositionCalculator, show_fms_lines: bool);
}
impl DrawableAircraft for Aircraft {
    fn draw(&mut self, position_calculator: &position_calc::PositionCalculator, show_fms_lines: bool) {
        let (x, y) = position_calculator.get_screen_coords_from_position(self.position());

        draw_poly_lines(
            x,
            y,
            4,
            5.0,
            45.0,
            1.0,
            WHITE,
        );

        if show_fms_lines {
            self.data().fms_graphics.iter_mut().for_each(|fms_line| fms_line.draw(position_calculator, Color::new(0.6352941176470588, 0.196078431372549, 0.6588235294117647, 1.0)));
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

        draw_text_ex(self.callsign(), x, y + 20.0, text_params.clone());
        draw_text_ex(&(self.position().alt.floor() as i32).to_string(), x, y + 35.0, text_params);
    }
}



pub trait DrawableFmsArc {
    fn calculate_arc_points(&mut self, position_calc: &PositionCalculator);

}

impl DrawableFmsArc for FmsArc {
    fn calculate_arc_points(&mut self, position_calc: &PositionCalculator) {
        match self.state {
            FmsArcState::Initialised { .. } => return,
            FmsArcState::Uninitialised { centre, radius_m, start_bearing_true, end_bearing_true, clockwise } => {
                let (mut start_bearing, mut end_bearing) = if clockwise { (start_bearing_true, end_bearing_true) } else { (end_bearing_true, start_bearing_true) };
                if end_bearing < start_bearing {
                    end_bearing += 360.0;
                }

                let x_rad = position_calc.n_mi_to_deg_lon(common::util::m_to_n_mi(radius_m));
                let y_rad = position_calc.n_mi_to_deg_lat(common::util::m_to_n_mi(radius_m));

                let mut points = Vec::new();

                while start_bearing < end_bearing {
                    let angle = start_bearing.to_radians();
                    let x = centre.lon + (x_rad * f32::sin(angle));
                    let y = centre.lat + (y_rad * f32::cos(angle));
                    points.push(Position::new(y, x));
                    start_bearing += 5.0;
                }
                let angle = end_bearing.to_radians();
                let x = centre.lon + (x_rad * f32::sin(angle));
                let y = centre.lat + (y_rad * f32::cos(angle));
                points.push(Position::new(y, x));

                let mut lines = Vec::with_capacity(points.len() + 1);
                for i in 1..points.len() {
                    let line = FmsLine { start: points[i - 1], end: points[i] };
                    lines.push(line);
                }
                self.state = FmsArcState::Initialised { lines };
            }
        }
    }
}
impl Draw for FmsArc {
    fn draw(&mut self, position_calculator: &position_calc::PositionCalculator, default_colour: Color) {
        if let FmsArcState::Uninitialised { .. } = self.state {
            self.calculate_arc_points(position_calculator);
        }
        if let FmsArcState::Initialised { lines } = &mut self.state {
            for line in lines.iter_mut() {
                line.draw(position_calculator, default_colour);
            }
        }
    }
}




impl Draw for FmsGraphic {
    fn draw(&mut self, position_calculator: &position_calc::PositionCalculator, default_colour: Color) {
        match self {
            FmsGraphic::Line(line) => line.draw(position_calculator, default_colour),
            FmsGraphic::Arc(arc) => arc.draw(position_calculator, default_colour),
        }
    }
}

impl Draw for FmsLine {
    fn draw(&mut self, position_calculator: &position_calc::PositionCalculator, default_colour: Color) {

        let (start_x, start_y) = position_calculator.get_screen_coords_from_position(&self.start);
        let (end_x, end_y) = position_calculator.get_screen_coords_from_position(&self.end);

        draw_line(start_x, start_y, end_x, end_y, 1.0, default_colour);
    }
}