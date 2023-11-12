use serde::{Serialize, Deserialize};



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
}