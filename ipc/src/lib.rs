use std::path::PathBuf;

use serde::{Serialize, Deserialize};


pub mod profile;

#[derive(Debug, Serialize, Deserialize)]
pub enum MessageType {
    AircraftData(Vec<SimAircraftRecord>),
}


#[derive(Debug, Serialize, Deserialize)]
pub struct SimAircraftRecord {
    pub callsign: String,
    pub lat: f32,
    pub lon: f32,
    pub alt: i32,
    pub fms_lines: Vec<SimAircraftFmsLine>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SimAircraftFmsLine {
    pub start_lat: f32,
    pub start_lon: f32,
    pub end_lat: f32,
    pub end_lon: f32,
}












