use std::{ops::Deref, fmt::Display};

use common::{aircraft_data::AircraftData, position::Position};
use indexmap::IndexMap;



/// Stores the aircraft
#[derive(Debug)]
pub struct AircraftManager {
    /// Stores all the aircraft records
    aircraft_map:           IndexMap<String, Aircraft>,
    current_selected_idx:   Option<usize>,
}
impl AircraftManager {
    pub fn new() -> Self {
        Self {
            aircraft_map: IndexMap::new(),
            current_selected_idx: None,
        }
    }
    pub fn aircraft(&mut self) -> indexmap::map::ValuesMut<'_, String, Aircraft> {
        self.aircraft_map.values_mut()
    }
}



/// Represents a record of an aircraft
#[derive(Debug)]
pub struct Aircraft {
    callsign: String,
    updates: Vec<AircraftData>,
}
impl Aircraft {
    pub fn data(&mut self) -> &mut AircraftData {
        self.updates.last_mut().unwrap()
    }
    pub fn position(&mut self) -> &Position {
        &self.data().position
    }
    pub fn callsign(&self) -> &str {
        &self.callsign
    }
}