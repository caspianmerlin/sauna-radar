use std::f32::consts::PI;

use ipc::{SimAircraftFmsLine, SimAircraftFmsGraphic};
use macroquad::{shapes::{draw_poly_lines, draw_line}, color::{WHITE, Color}, text::{load_ttf_font_from_bytes, TextParams, draw_text_ex}};

use crate::sector::{items::Position, draw::{Draw, DrawableObjectType}};

use super::{position_calc::{self, PositionCalculator}, display::TAG_FONT};

#[derive(Debug)]
pub struct AircraftRecord {
    pub callsign: String,
    pub position: Position,
    pub alt: i32,
    pub fms_graphics: Vec<FmsGraphic>,
}
impl From<ipc::SimAircraftRecord> for AircraftRecord {
    fn from(value: ipc::SimAircraftRecord) -> Self {
        AircraftRecord { callsign: value.callsign, position: Position { lat: value.lat, lon: value.lon, cached_x: 0.0, cached_y: 0.0 }, alt: value.alt, fms_graphics: value.fms_graphics.into_iter().map(FmsGraphic::from).collect(), }
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
            self.fms_graphics.iter_mut().for_each(|fms_line| fms_line.draw(position_calculator, Color::new(0.6352941176470588, 0.196078431372549, 0.6588235294117647, 1.0)));
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
pub enum FmsGraphic {
    Line(FmsLine),
    Arc(FmsArc),
}

#[derive(Debug, PartialEq)]
pub struct FmsArc {
    pub state: FmsArcState,
}
impl FmsArc {
    pub fn calculate_arc_points(&mut self, position_calc: &PositionCalculator) {
        match self.state {
            FmsArcState::Initialised { .. } => return,
            FmsArcState::Uninitialised { centre, radius_m, start_bearing_true, end_bearing_true, clockwise } => {

                println!("Start bearing: {}", start_bearing_true);
                println!("End bearing: {}", end_bearing_true);
                println!("Clockwise: {}", clockwise);
                let (mut start_bearing, mut end_bearing) = if clockwise { (start_bearing_true, end_bearing_true) } else { (end_bearing_true, start_bearing_true) };
                if end_bearing < start_bearing {
                    end_bearing += 360.0;
                }
                println!("Start bearing adj: {}", start_bearing);
                println!("End bearing adj: {}", end_bearing);





                let x_rad = position_calc.n_mi_to_deg_lon(m_to_n_mi(radius_m));
                let y_rad = position_calc.n_mi_to_deg_lat(m_to_n_mi(radius_m));

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

#[derive(Debug, PartialEq)]
pub enum FmsArcState {
    Initialised{ lines: Vec<FmsLine> },
    Uninitialised { centre: Position, radius_m: f32, start_bearing_true: f32, end_bearing_true: f32, clockwise: bool }, 
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

#[derive(Debug, PartialEq)]
pub struct FmsLine {
    pub start: Position,
    pub end: Position,
}

impl From<SimAircraftFmsGraphic> for FmsGraphic {
    fn from(value: SimAircraftFmsGraphic) -> Self {
        match value {
            SimAircraftFmsGraphic::Line(sim_aircraft_fms_line) => {
                FmsGraphic::Line(FmsLine { start: Position { lat: sim_aircraft_fms_line.start_lat, lon: sim_aircraft_fms_line.start_lon, cached_x: 0.0, cached_y: 0.0 }, end: Position { lat: sim_aircraft_fms_line.end_lat, lon: sim_aircraft_fms_line.end_lon, cached_x: 0.0, cached_y: 0.0 } })
            },
            SimAircraftFmsGraphic::Arc(sim_aircraft_fms_arc) => {
                FmsGraphic::Arc(FmsArc { state: FmsArcState::Uninitialised { centre: Position::new(sim_aircraft_fms_arc.centre_lat, sim_aircraft_fms_arc.centre_lon), radius_m: sim_aircraft_fms_arc.radius_m, start_bearing_true: sim_aircraft_fms_arc.start_true_bearing, end_bearing_true: sim_aircraft_fms_arc.end_true_bearing, clockwise: sim_aircraft_fms_arc.clockwise } })
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

            self.start.cache_screen_coords(position_calculator);
            self.end.cache_screen_coords(position_calculator);

        draw_line(self.start.cached_x, self.start.cached_y, self.end.cached_x, self.end.cached_y, 1.0, default_colour);
    }
}

pub fn m_to_n_mi(m: f32) -> f32 {
    m / 1852.0
}