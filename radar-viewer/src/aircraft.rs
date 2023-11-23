use std::{ops::Deref, fmt::Display};

use common::{aircraft_data::{AircraftData, AircraftUpdate}, position::Position};
use indexmap::IndexMap;

use crate::radar::{position_calc::PositionCalculator, draw::DrawableAircraft};



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
    pub fn handle_aircraft_updates(&mut self, aircraft_updates: Vec<AircraftUpdate>) {
        for AircraftUpdate { callsign, data } in aircraft_updates {
            if let Some(aircraft) = self.aircraft_map.get_mut(&callsign) {
                aircraft.updates.push(data);
            } else {
                let aircraft = Aircraft { callsign: callsign.clone(), updates: vec![data] };
                self.aircraft_map.insert(callsign, aircraft);
            }
        }
    }
    pub fn draw(&mut self, position_calculator: &PositionCalculator, show_fms_lines: bool) {
        self.aircraft_map.values_mut().for_each(|aircraft| aircraft.draw(position_calculator, show_fms_lines));
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
    pub fn updates(&self) -> &Vec<AircraftData> {
        &self.updates
    }
}