use macroquad::prelude::Color;

pub mod draw;


#[derive(Debug)]
pub struct Sector {
    pub name: String,
    pub default_centre_pt: Position,
    pub n_mi_per_deg_lat: f32,
    pub n_mi_per_deg_lon: f32,
    pub magnetic_variation: f32,

    pub airports: Vec<NamedPoint>,
    pub vors: Vec<NamedPoint>,
    pub ndbs: Vec<NamedPoint>,
    pub fixes: Vec<NamedPoint>,
    pub artcc_entries: Vec<LineGroup>,
    pub artcc_low_entries: Vec<LineGroup>,
    pub artcc_high_entries: Vec<LineGroup>,
    pub low_airways: Vec<LineGroup>,
    pub high_airways: Vec<LineGroup>,
    pub sid_entries: Vec<LineGroup>,
    pub star_entries: Vec<LineGroup>,
    pub geo_entries: Vec<LineGroup>,
    pub regions: Vec<PolyGroup>,
    pub labels: Vec<Label>,
}

#[derive(Debug)]
pub struct Position {
    pub lat: f32,
    pub lon: f32,

    pub cached_x: f32,
    pub cached_y: f32,
}

#[derive(Debug)]
pub struct NamedPoint {
    pub identifier: String,
    pub position: Position,
    pub show_symbol: bool,
    pub show_identifier: bool,
}


#[derive(Debug)]
pub struct LineGroup {
    pub identifier: String,
    pub lines: Vec<ColouredLine>,
    pub show: bool,
}

#[derive(Debug)]
pub struct ColouredLine {
    pub start: Position,
    pub end: Position,
    pub colour: Option<Color>,
}

#[derive(Debug)]
pub struct PolyGroup {
    pub identifier: String,
    pub polys: Vec<ColouredPoly>,
    pub show: bool,
}

#[derive(Debug)]
pub struct ColouredPoly {
    pub points: Vec<Position>,
    pub indices: Vec<usize>,
}

#[derive(Debug)]
pub struct Label {
    pub text: String,
    pub position: Position,
    pub colour: Color,
    pub show: bool,
}