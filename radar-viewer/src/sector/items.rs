use macroquad::{prelude::{Color, Vec2}, shapes::{draw_poly_lines, draw_line, draw_triangle}};
use sct_reader::waypoint::Waypoint;

use crate::radar::position_calc::PositionCalculator;

use super::draw::{Draw, DrawableObjectType};

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Position {
    pub lat: f32,
    pub lon: f32,

    pub cached_x: f32,
    pub cached_y: f32,
}

impl From<sct_reader::position::Position<sct_reader::position::Valid>> for Position {
    fn from(value: sct_reader::position::Position<sct_reader::position::Valid>) -> Self {
        Position {
            lat: value.lat as f32,
            lon: value.lon as f32,
            cached_x: 0.0,
            cached_y: 0.0,
        }
    }
}
impl Position {
    pub fn cache_screen_coords(&mut self, position_calculator: &PositionCalculator) {
        self.cached_x = position_calculator.lon_to_window_x(self.lon);
        self.cached_y = position_calculator.lat_to_window_y(self.lat);
    }
}

#[derive(Debug)]
pub struct NamedPoint {
    pub identifier: String,
    pub position: Position,
    pub show_symbol: bool,
    pub show_identifier: bool,
}
impl Draw for NamedPoint {
    fn draw(&mut self, position_calculator: &crate::radar::position_calc::PositionCalculator, drawable_object_type: super::draw::DrawableObjectType) {
        if position_calculator.invalidated {
            self.position.cache_screen_coords(position_calculator);
        }
        let colour = drawable_object_type.default_colour();
        if !self.visible() {
            return;
        }
        match drawable_object_type {
            DrawableObjectType::Fix => {
                draw_poly_lines(
                    self.position.cached_x,
                    self.position.cached_y,
                    3,
                    5.0,
                    30.0,
                    1.0,
                    colour,
                );
            }
            _ => {
                draw_poly_lines(
                    self.position.cached_x,
                    self.position.cached_y,
                    4,
                    5.0,
                    45.0,
                    1.0,
                    colour,
                );
            }
        }
    }
}

impl SetVisibility for NamedPoint {
    fn set_visibility(&mut self, visible: bool) {
        self.show_symbol = visible;
    }
    fn visible(&self) -> bool {
        self.show_symbol
    }
}

impl<W> From<W> for NamedPoint where W: Waypoint {
    fn from(value: W) -> Self {
        NamedPoint {
            identifier: value.identifier().to_owned(),
            position: value.position().into(),
            show_symbol: false,
            show_identifier: false,
        }
    }
}

#[derive(Debug)]
pub struct LineGroup {
    pub identifier: String,
    pub lines: Vec<ColouredLine>,
    pub show: bool,
}

impl Draw for LineGroup {
    fn draw(&mut self, position_calculator: &PositionCalculator, drawable_object_type: DrawableObjectType) {
        if !self.visible() {
            return;
        }
        for line in &mut self.lines {
            if position_calculator.invalidated {
                line.start.cache_screen_coords(position_calculator);
                line.end.cache_screen_coords(position_calculator);
            }
            draw_line(
                line.start.cached_x,
                line.start.cached_y,
                line.end.cached_x,
                line.end.cached_y,
                1.0,
                line.colour.unwrap_or(drawable_object_type.default_colour()),
            );
        }

    }
}

impl SetVisibility for LineGroup {
    fn set_visibility(&mut self, visible: bool) {
        self.show = visible;
    }
    fn visible(&self) -> bool {
        self.show
    }
}
impl From<sct_reader::line::LineGroup<sct_reader::line::ColouredLine>> for LineGroup {
    fn from(value: sct_reader::line::LineGroup<sct_reader::line::ColouredLine>) -> Self {
        LineGroup {
            identifier: value.name,
            lines: value.lines.into_iter().map(|line| line.into()).collect(),
            show: false,
        }
    }
}

#[derive(Debug)]
pub struct ColouredLine {
    pub start: Position,
    pub end: Position,
    pub colour: Option<Color>,
}
impl From<sct_reader::line::ColouredLine> for ColouredLine {
    fn from(value: sct_reader::line::ColouredLine) -> Self {
        ColouredLine {
            start: value.line.start.into(),
            end: value.line.end.into(),
            colour: value.colour.map(|c| mq_colour_from_sf_colour(c)),
        }
    }
}

#[derive(Debug)]
pub struct PolyGroup {
    pub identifier: String,
    pub polys: Vec<ColouredPoly>,
    pub show: bool,
}
impl SetVisibility for PolyGroup {
    fn set_visibility(&mut self, visible: bool) {
        self.show = visible;
    }
    fn visible(&self) -> bool {
        self.show
    }
}
impl From<sct_reader::sector::RegionGroup> for PolyGroup {
    fn from(value: sct_reader::sector::RegionGroup) -> Self {
        PolyGroup {
            identifier: value.name,
            polys: value.regions.into_iter().map(ColouredPoly::from).collect(),
            show: false,
        }
    }
}
impl Draw for PolyGroup {
    fn draw(&mut self, position_calculator: &PositionCalculator, drawable_object_type: DrawableObjectType) {
        if !self.visible() {
            return;
        }

        for poly in &mut self.polys {
            if position_calculator.invalidated {
                for point in &mut poly.points {
                    point.cache_screen_coords(position_calculator);
                }
            }

            for triangle in poly.indices.chunks_exact(3) {
                let index_a = triangle[0] * 2;
                let index_b = triangle[1] * 2;
                let index_c = triangle[2] * 2;

                let vertex_a = Vec2::new(poly.points[index_a].cached_x, poly.points[index_a].cached_y);
                let vertex_b = Vec2::new(poly.points[index_b].cached_x, poly.points[index_b].cached_y);
                let vertex_c = Vec2::new(poly.points[index_c].cached_x, poly.points[index_c].cached_y);

    
                draw_triangle(vertex_a, vertex_b, vertex_c, poly.colour);


        }

        
        }


    }
}

#[derive(Debug)]
pub struct ColouredPoly {
    pub colour: Color,
    pub points: Vec<Position>,
    pub indices: Vec<usize>,
}
impl From<sct_reader::sector::Region> for ColouredPoly {
    fn from(value: sct_reader::sector::Region) -> Self {
        let colour = mq_colour_from_sf_colour(value.colour);
        let points: Vec<Position> = value.vertices.into_iter().map(|v| v.into()).collect();

        // Ear cutting
        let vertices = points
            .iter()
            .map(|position| [position.lon as f32, position.lat as f32])
            .flatten()
            .collect::<Vec<_>>();
        let indices = earcutr::earcut(&vertices, &vec![], 2).unwrap();

        ColouredPoly { colour, points, indices }
    }
}

#[derive(Debug)]
pub struct Label {
    pub text: String,
    pub position: Position,
    pub colour: Color,
    pub show: bool,
}
impl SetVisibility for Label {
    fn set_visibility(&mut self, visible: bool) {
        self.show = visible;
    }
    fn visible(&self) -> bool {
        self.show
    }
}
impl From<sct_reader::sector::Label> for Label {
    fn from(value: sct_reader::sector::Label) -> Self {
        Label {
            text: value.name,
            position: value.position.into(),
            colour: mq_colour_from_sf_colour(value.colour),
            show: false,
        }
    }
}

pub fn mq_colour_from_sf_colour(value: sct_reader::colour::Colour) -> Color {
    Color {
        r: value.r as f32 / 255.0,
        g: value.g as f32 / 255.0,
        b: value.b as f32 / 255.0,
        a: 1.0,
    }
}



pub trait SetVisibility {
    fn set_visibility(&mut self, visible: bool);
    fn visible(&self) -> bool;
}