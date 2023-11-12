use std::sync::{Arc, Mutex};

use ipc::SimAircraftRecord;
use macroquad::{prelude::{Color, is_key_down, KeyCode, is_key_pressed, is_mouse_button_pressed, MouseButton, mouse_position, is_mouse_button_down, mouse_delta_position, Vec2, WHITE, mouse_wheel}, window, text::draw_text};
use sct_reader::line::{ColouredLine, Line as SectorLine};

use crate::{sector::{Sector, draw::{Draw, DrawableObjectType}}, AircraftRecord};

use super::{
    line::{Line, LineType},
    position_calc::PositionCalculator, WINDOW_HT_N_MI,
};

#[derive(Debug)]
pub struct RadarDisplay {
    sector: Sector,
    position_calculator: PositionCalculator,
    mouse_pos_last_frame: Vec2,
    aircraft_data: Arc<Mutex<Vec<AircraftRecord>>>,
}

impl RadarDisplay {
    pub fn new(sector: Sector, screen_ht_n_mi: Option<f32>, aircraft_data: Arc<Mutex<Vec<AircraftRecord>>>) -> RadarDisplay {
        let position_calculator = PositionCalculator::new(
            sector.default_centre_pt.lat,
            sector.default_centre_pt.lon,
            screen_ht_n_mi
                .map(|x| x as f32)
                .unwrap_or(WINDOW_HT_N_MI),
            sector.n_mi_per_deg_lat,
            sector.n_mi_per_deg_lon,
        );
        RadarDisplay { sector, position_calculator, mouse_pos_last_frame: Vec2::default(), aircraft_data }
    }
    pub fn update(&mut self) {

        let current_position = {
            let mouse_pos = mouse_position();
            Vec2::new(mouse_pos.0, mouse_pos.1)
        };

        if is_key_pressed(KeyCode::Up) {
            self.position_calculator.decrease_window_ht_by_n_mi(10.0);
        }
        if is_key_pressed(KeyCode::Down) {
            self.position_calculator.increase_window_ht_by_n_mi(10.0);
        }
        if is_mouse_button_down(MouseButton::Right) {
            
            let diff = self.mouse_pos_last_frame - current_position;
            self.position_calculator.update_position_by_mouse_offset(diff);
        }

        let mouse_wheel_delta = mouse_wheel().1;
        if mouse_wheel_delta != 0.0 {
            self.position_calculator.decrease_window_ht_by_n_mi(mouse_wheel_delta / 2.0);
        }
        self.mouse_pos_last_frame = current_position;
    }




    pub fn draw(&mut self) {
        self.sector.draw(&mut self.position_calculator, DrawableObjectType::Default);
        let mut mutex_guard = self.aircraft_data.lock().unwrap();
        for aircraft in mutex_guard.iter_mut() {
            aircraft.draw(&mut self.position_calculator, DrawableObjectType::Default);
        }
        self.position_calculator.invalidated = false;
    }
}
