use std::{ops::Deref, fmt::Display};

use common::{aircraft_data::{AircraftData, AircraftUpdate}, position::Position};
use indexmap::IndexMap;
use macroquad::input::{is_mouse_button_pressed, mouse_position};

use crate::radar::{position_calc::PositionCalculator, draw::DrawableAircraft};



/// Stores the aircraft
#[derive(Debug)]
pub struct AircraftManager {
    /// Stores all the aircraft records
    aircraft_map:           IndexMap<String, Aircraft>,
    current_selected:   Option<String>,
}
impl AircraftManager {
    pub fn new() -> Self {
        Self {
            aircraft_map: IndexMap::new(),
            current_selected: None,
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
    pub fn draw(&mut self, position_calculator: &PositionCalculator, show_fms_lines: bool, num_speed_vectors: usize) {
        self.aircraft_map.values_mut().for_each(|aircraft| aircraft.draw(position_calculator, show_fms_lines, num_speed_vectors, &self.current_selected));
    }
    pub fn get_aircraft(&self, callsign: &str) -> Option<&Aircraft> {
        self.aircraft_map.get(callsign)
    }

    pub fn check_if_ac_clicked(&mut self, mouse_position: (f32, f32), position_calculator: &PositionCalculator) -> Option<&Aircraft> {
        for aircraft in self.aircraft_map.values_mut() {
            if aircraft.was_clicked(mouse_position, position_calculator) {
                let was_selected = match &self.current_selected {
                    Some(cs) => aircraft.callsign() == cs,
                    None => false,
                };

                if was_selected {
                    self.current_selected = None;
                    return None;
                } else {
                    self.current_selected = Some(aircraft.callsign.clone());
                    return Some(aircraft);
                }
            }
        }
        return None;
    }
    pub fn try_select_by_partial_callsign(&mut self, partial_callsign: &str) -> bool {
        let partial_callsign = partial_callsign.to_uppercase();
        for (callsign, aircraft) in self.aircraft_map.iter() {
            if callsign.to_uppercase().contains(&partial_callsign) {
                self.current_selected = Some(callsign.clone());
                return true;
            }
        }
        return false;
    }
    pub fn current_selected(&self) -> Option<&String> {
        self.current_selected.as_ref()
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
    pub fn position(&self) -> &Position {
        &self.updates.last().unwrap().position
    }
    pub fn callsign(&self) -> &str {
        &self.callsign
    }
    pub fn updates(&self) -> &Vec<AircraftData> {
        &self.updates
    }
    pub fn was_clicked(&mut self, mouse_position: (f32, f32), position_calculator: &PositionCalculator) -> bool {
        let (centre_x, centre_y) = position_calculator.get_screen_coords_from_position(self.position());
        let left = centre_x - 5.;
        let right = centre_x + 5.;
        let top = centre_y - 5.;
        let bottom = centre_y + 5.;

        if (left..=right).contains(&mouse_position.0) && (top..=bottom).contains(&mouse_position.1) {
            return true;
        }
        return false;
    }
}