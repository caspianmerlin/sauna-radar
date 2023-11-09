use crate::{colour::Colour, position::{Position, Valid}, waypoint::Airport};

#[derive(Debug)]
pub struct Sector {
    pub sector_info: SectorInfo,
}

#[derive(Debug)]
pub struct SectorInfo {
    pub name: String,
    pub default_callsign: String,
    pub default_airport: Airport,
    pub default_centre_pt: Position<Valid>,
    pub n_mi_per_deg_lat: f32,
    pub n_mi_per_deg_lon: f32,
    pub magnetic_variation: f32,
    pub sector_scale: f32,
}



// Lines

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Line {
    pub start: Position<Valid>,
    pub end: Position<Valid>,
}

#[derive(Debug, Clone)]
pub struct MultiLineMaybeColoured {
    pub name: String,
    pub lines: Vec<MaybeColouredLine>,
}

#[derive(Debug, Clone)]
pub struct MaybeColouredLine {
    pub line: Line,
    pub colour: Option<Colour>,
}



#[derive(Debug, Clone)]
pub struct MultiLine {
    pub name: String,
    pub lines: Vec<Line>,
}