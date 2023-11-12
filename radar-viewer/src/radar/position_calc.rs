use macroquad::{prelude::Color, window};
use sct_reader::line::{ColouredLine, Line as SectorLine};

use super::line::{Line, LineType};

#[derive(Debug)]
pub struct PositionCalculator {
    // Position calc stuff
    window_ht_n_mi: f32,
    n_mi_per_deg_lat: f32,
    n_mi_per_deg_lon: f32,
    origin_lat: f32,
    origin_lon: f32,

    pub invalidated: bool,
}

impl PositionCalculator {
    pub fn new(
        window_centre_lat: f32,
        window_centre_lon: f32,
        window_ht_n_mi: f32,
        n_mi_per_deg_lat: f32,
        n_mi_per_deg_lon: f32,
    ) -> PositionCalculator {
        let mut position_calculator = PositionCalculator {
            window_ht_n_mi,
            n_mi_per_deg_lat,
            n_mi_per_deg_lon,
            origin_lat: 0.0,
            origin_lon: 0.0,
            invalidated: true,
        };
        position_calculator.update_centre_lat_lon(window_centre_lat, window_centre_lon);
        position_calculator
    }
    fn update_zoom(&mut self, window_ht_n_mi: f32) {
        self.window_ht_n_mi = window_ht_n_mi;
        self.invalidated = true;
    }
    pub fn increase_window_ht_by_n_mi(&mut self, n_mi: f32) {
        self.update_zoom(self.window_ht_n_mi + n_mi);
    }
    pub fn decrease_window_ht_by_n_mi(&mut self, n_mi: f32) {
        self.update_zoom(self.window_ht_n_mi - n_mi);
    }
    pub fn update_centre_lat_lon(&mut self, window_centre_lat: f32, window_centre_lon: f32) {
        let half_window_ht_px = window::screen_height() / 2.0;
        let lat_offset = half_window_ht_px / self.pixels_per_deg_lat();
        let origin_lat = window_centre_lat + lat_offset;

        let half_window_wi_px = window::screen_width() / 2.0;
        let lon_offset = half_window_wi_px / self.pixels_per_deg_lon();
        let origin_lon = window_centre_lon - lon_offset;

        self.origin_lat = origin_lat;
        self.origin_lon = origin_lon;
    }
    pub fn pixels_per_n_mi(&self) -> f32 {
        window::screen_height() / self.window_ht_n_mi
    }
    pub fn pixels_per_deg_lat(&self) -> f32 {
        self.pixels_per_n_mi() * self.n_mi_per_deg_lat
    }
    pub fn pixels_per_deg_lon(&self) -> f32 {
        self.pixels_per_n_mi() * self.n_mi_per_deg_lon
    }
    pub fn lat_to_window_y(&self, lat: f32) -> f32 {
        let deg_offset_from_origin = self.origin_lat - lat;
        let px_offset_from_origin = deg_offset_from_origin * self.pixels_per_deg_lat();
        px_offset_from_origin
    }
    pub fn lon_to_window_x(&self, lon: f32) -> f32 {
        let deg_offset_from_origin = lon - self.origin_lon;
        let px_offset_from_origin = deg_offset_from_origin * self.pixels_per_deg_lon();
        px_offset_from_origin
    }

    pub fn window_ht_deg(&self) -> f32 {
        self.window_ht_n_mi / self.n_mi_per_deg_lat
    }
    pub fn window_wi_deg(&self) -> f32 {
        let window_wi_n_mi = window::screen_width() / self.pixels_per_n_mi();
        window_wi_n_mi / self.n_mi_per_deg_lon
    }
    pub fn is_within_screen_bounds(&self, lat: f32, lon: f32) -> bool {
        let top_lat = self.origin_lat;
        let bottom_lat = self.origin_lat - self.window_ht_deg();
        let left_lon = self.origin_lon;
        let right_lon = self.origin_lon + self.window_wi_deg();

        let lat_a = f32::min(top_lat, bottom_lat);
        let lat_b = f32::max(top_lat, bottom_lat);

        let lon_a = f32::min(left_lon, right_lon);
        let lon_b = f32::max(left_lon, right_lon);
        (lat_a..=lat_b).contains(&lat) && (lon_a..=lon_b).contains(&lon)
    }
}
