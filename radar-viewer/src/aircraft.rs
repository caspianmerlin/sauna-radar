use std::{ops::Deref, fmt::Display};

use indexmap::IndexMap;



/// Stores the aircraft
#[derive(Debug)]
pub struct AircraftManager {
    /// Stores all the aircraft records
    aircraft_map:           IndexMap<String, Aircraft>,
    current_selected_idx:   Option<usize>,
}



/// Represents a record of an aircraft
#[derive(Debug)]
pub struct Aircraft {
    callsign: String,
    updates: Vec<DataUpdate>,
}

// NOTE: This is fine because the constructor ensures that at least one DataUpdate exists from creation.
impl Deref for Aircraft {
    type Target = DataUpdate;
    fn deref(&self) -> &Self::Target {
        self.updates.last().unwrap()
    }
}

