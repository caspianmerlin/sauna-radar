use serde::{Serialize, Deserialize};

use crate::aircraft_data::AircraftUpdate;


#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PacketType {
    AircraftDataUpdate(Vec<AircraftUpdate>),
    LogMessage(String),
}