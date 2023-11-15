use std::{sync::{Arc, Mutex, mpsc::{Receiver, TryRecvError}}, net::TcpStream};

use ipc::{SimAircraftRecord, profile::colours::RadarColours};
use macroquad::{prelude::{Color, is_key_down, KeyCode, is_key_pressed, is_mouse_button_pressed, MouseButton, mouse_position, is_mouse_button_down, mouse_delta_position, Vec2, WHITE, mouse_wheel}, window, text::{draw_text, Font, load_ttf_font_from_bytes}, ui::{Ui, root_ui}};
use once_cell::sync::{Lazy, OnceCell};
use sct_reader::line::{ColouredLine, Line as SectorLine};

use crate::{sector::{Sector, draw::{Draw, DrawableObjectType}, ui::SectorUi}, IpcMessage, ReceiverDropGuard, radar_colour_to_mq_colour};

use super::{
    line::{Line, LineType},
    position_calc::PositionCalculator, WINDOW_HT_N_MI, aircraft::AircraftRecord,
};

pub static TAG_FONT: OnceCell<Font> = OnceCell::new(); 

#[derive(Debug)]
pub struct RadarDisplay {
    sector: Sector,
    position_calculator: PositionCalculator,
    mouse_pos_last_frame: Vec2,
    sector_ui: SectorUi,
    show_fms_lines: bool,
    colours: RadarColours,
}

impl RadarDisplay {
    pub fn new(sector: Sector, screen_ht_n_mi: f32, colours: RadarColours) -> RadarDisplay {
        let position_calculator = PositionCalculator::new(
            sector.default_centre_pt.lat,
            sector.default_centre_pt.lon,
            screen_ht_n_mi,
            sector.n_mi_per_deg_lat,
            sector.n_mi_per_deg_lon,
        );
        RadarDisplay { sector, position_calculator, mouse_pos_last_frame: Vec2::default(), sector_ui: SectorUi::new(), show_fms_lines: false, colours }
    }
    pub fn background_colour(&self) -> Color {
        radar_colour_to_mq_colour(&self.colours.background)
    }
    pub fn update(&mut self, aircraft: &mut Vec<AircraftRecord>) {
        
        let ui_has_mouse = root_ui().is_mouse_over(Vec2::new(mouse_position().0, mouse_position().1));

        let current_position = {
            let mouse_pos = mouse_position();
            Vec2::new(mouse_pos.0, mouse_pos.1)
        };


        

        if is_key_pressed(KeyCode::F2) {
            self.show_fms_lines = !self.show_fms_lines;
        }
        if is_key_pressed(KeyCode::F3) {
            self.sector_ui.toggle_visibility();
        }
        
        if is_mouse_button_down(MouseButton::Right) {
            let diff = self.mouse_pos_last_frame - current_position;
            self.position_calculator.update_position_by_mouse_offset(diff);
        }
        if is_mouse_button_pressed(MouseButton::Left) {
            println!("Mouse pos: {:?}", mouse_position());
        }

        if !ui_has_mouse {
            let mouse_wheel_delta = mouse_wheel().1;
            if mouse_wheel_delta < 0.0 {
            self.position_calculator.zoom_out_mouse(mouse_position());
        } else if mouse_wheel_delta > 0.0 {
            self.position_calculator.zoom_in_mouse(mouse_position());
        }
        }
        
        self.mouse_pos_last_frame = current_position;
    }





    pub fn draw(&mut self, aircraft: &mut Vec<AircraftRecord>) {
        self.position_calculator.invalidated = true;
        self.sector.draw(&mut self.position_calculator, &self.colours);

        for aircraft in aircraft.iter_mut() {
            aircraft.draw(&mut self.position_calculator, self.show_fms_lines);
        }

        
        
        
        self.sector_ui.show_ui(&mut self.sector);
    }
}

