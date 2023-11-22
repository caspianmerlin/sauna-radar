use std::path::PathBuf;

use macroquad::{color::{Color, BLACK}, input::is_key_pressed, miniquad::KeyCode};

use crate::aircraft::AircraftManager;

use super::{display::RadarDisplay, loader::RadarDisplayLoader};



pub struct RadarManager {
    loader: RadarDisplayLoader,
    radar_displays: Vec<RadarDisplay>,
    active_display: usize,
}
impl RadarManager {
    pub fn new(paths: Vec<PathBuf>) -> RadarManager {
        let mut loader = RadarDisplayLoader::new();
        loader.start_load(paths);
        Self { loader, radar_displays: vec![], active_display: 0 }
    }
    fn store(&mut self, radar_displays: Vec<RadarDisplay>) {
        self.radar_displays = radar_displays;
        self.active_display = 0;
    }
    pub fn cycle(&mut self) {
        if self.radar_displays.is_empty() { return; }
        self.active_display += 1;
        if self.active_display > self.radar_displays.len() - 1 {
            self.active_display = 0;
        }
    }
    pub fn cycle_back(&mut self) {
        if self.radar_displays.is_empty() { return; }
        if self.active_display == 0 {
            self.active_display = self.radar_displays.len() - 1;
            return;
        }
        self.active_display -= 1;
    }
    pub fn is_initialised(&self) -> bool {
        !self.radar_displays.is_empty()
    }
    pub fn update(&mut self, aircraft_manager: &mut AircraftManager) {
        if let Some(radar_displays) = self.loader.poll() {
            println!("Loaded {} radar displays", radar_displays.len());
            self.radar_displays = radar_displays;
        }
        if is_key_pressed(KeyCode::F6) {
            self.cycle_back();
        } else if is_key_pressed(KeyCode::F7) {
            self.cycle();
        }
        self.radar_displays.get_mut(self.active_display).map(|active_display| active_display.update(aircraft_manager));
    }
    pub fn draw(&mut self, aircraft_manager: &mut AircraftManager) {
        self.radar_displays.get_mut(self.active_display).map(|active_display| active_display.draw(aircraft_manager));
    }
    pub fn background_colour(&self) -> Color {
        self.radar_displays.get(self.active_display).map(|active_display| active_display.background_colour()).unwrap_or(BLACK)
    }
    pub fn active_display(&self) -> Option<&RadarDisplay> {
        self.radar_displays.get(self.active_display)
    }
}