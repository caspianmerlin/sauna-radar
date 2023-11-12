#[derive(Debug, Clone)]
pub struct FilterSettings {
    pub airports: Vec<WaypointEntry>,
    pub vors: Vec<BeaconEntry>,
    pub ndbs: Vec<BeaconEntry>,
    pub fixes: Vec<WaypointEntry>,
    pub artcc: Vec<LineGroupEntry>,
    pub artcc_high: Vec<LineGroupEntry>,
    pub artcc_low: Vec<LineGroupEntry>,
    pub sids: Vec<LineGroupEntry>,
    pub stars: Vec<LineGroupEntry>,
    pub low_airways: Vec<AirwayEntry>,
    pub high_airways: Vec<AirwayEntry>,
    pub geo: Vec<LineGroupEntry>,
    pub regions: Vec<LineGroupEntry>,
}

#[derive(Debug, Clone)]
pub struct WaypointEntry {
    pub identifier: String,
    pub show_symbol: bool,
    pub show_identifier: bool,
}

#[derive(Debug, Clone)]
pub struct AirwayEntry {
    pub identifier: String,
    pub show_line: bool,
    pub show_identifier: bool,
}

#[derive(Debug, Clone)]
pub struct BeaconEntry {
    pub identifier: String,
    pub show_symbol: bool,
    pub show_identifier: bool,
    pub show_frequency: bool,
}

#[derive(Debug, Clone)]
pub struct LineGroupEntry {
    pub identifier: String,
    pub show: bool,
}
