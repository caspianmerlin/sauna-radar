use ipc::filters::FilterSettings;
use macroquad::{prelude::{Color, is_key_down, KeyCode}, window};
use sct_reader::line::{ColouredLine, Line as SectorLine};

use crate::sector::{Sector, draw::{Draw, DrawableObjectType}};

use super::{
    line::{Line, LineType},
    position_calc::PositionCalculator, WINDOW_HT_N_MI,
};

#[derive(Debug)]
pub struct RadarDisplay {
    sector: Sector,
    position_calculator: PositionCalculator,
    
}

impl RadarDisplay {
    pub fn new(sector: Sector, screen_ht_n_mi: Option<f32>) -> RadarDisplay {
        let position_calculator = PositionCalculator::new(
            sector.default_centre_pt.lat,
            sector.default_centre_pt.lon,
            screen_ht_n_mi
                .map(|x| x as f32)
                .unwrap_or(WINDOW_HT_N_MI),
            sector.n_mi_per_deg_lat,
            sector.n_mi_per_deg_lon,
        );
        RadarDisplay { sector, position_calculator }
    }
    pub fn update(&mut self) {
        if is_key_down(KeyCode::Down) {
            self.position_calculator.decrease_window_ht_by_n_mi(5.0);
        }
    }




    pub fn draw(&mut self) {
        self.sector.draw(&mut self.position_calculator, DrawableObjectType::Default);




        self.position_calculator.invalidated = false;
    }
}
