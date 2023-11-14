use std::{sync::{Arc, Mutex, mpsc::{Receiver, TryRecvError}}, net::TcpStream};

use ipc::SimAircraftRecord;
use macroquad::{prelude::{Color, is_key_down, KeyCode, is_key_pressed, is_mouse_button_pressed, MouseButton, mouse_position, is_mouse_button_down, mouse_delta_position, Vec2, WHITE, mouse_wheel}, window, text::{draw_text, Font, load_ttf_font_from_bytes}, ui::{Ui, root_ui}};
use once_cell::sync::{Lazy, OnceCell};
use sct_reader::line::{ColouredLine, Line as SectorLine};

use crate::{sector::{Sector, draw::{Draw, DrawableObjectType}, ui::SectorUi}, AircraftRecord, IpcMessage, ReceiverDropGuard};

use super::{
    line::{Line, LineType},
    position_calc::PositionCalculator, WINDOW_HT_N_MI,
};

pub static TAG_FONT: OnceCell<Font> = OnceCell::new(); 

#[derive(Debug)]
pub struct RadarDisplay {
    sector: Sector,
    position_calculator: PositionCalculator,
    mouse_pos_last_frame: Vec2,
    aircraft_records: Vec<AircraftRecord>,
    aircraft_data_receiver: ReceiverDropGuard,
    sector_ui: SectorUi,
    show_help: bool,
    show_fms_lines: bool,
    

    tcp_stream: Option<TcpStream>,
    fullscreen: bool,
}

impl RadarDisplay {
    pub fn new(sector: Sector, screen_ht_n_mi: Option<f32>, aircraft_data_receiver: ReceiverDropGuard) -> RadarDisplay {
        println!("D");
        let position_calculator = PositionCalculator::new(
            sector.default_centre_pt.lat,
            sector.default_centre_pt.lon,
            screen_ht_n_mi
                .map(|x| x as f32)
                .unwrap_or(WINDOW_HT_N_MI),
            sector.n_mi_per_deg_lat,
            sector.n_mi_per_deg_lon,
        );
        RadarDisplay { sector, position_calculator, mouse_pos_last_frame: Vec2::default(), aircraft_records: vec![], aircraft_data_receiver, sector_ui: SectorUi::new(), show_help: true, fullscreen: false, tcp_stream: None, show_fms_lines: false, }
    }
    pub fn update(&mut self) {
        match self.aircraft_data_receiver.0.try_recv()  {
            Ok(IpcMessage::TcpStream(tcp_stream)) => {
                println!("Successfully received");
                self.tcp_stream = Some(tcp_stream);
            }
            Ok(IpcMessage::AircraftData(aircraft_data)) => self.aircraft_records = aircraft_data,
            Err(e) => {
                if let TryRecvError::Disconnected = e { println!("Try recv error") };
            }
        }
            
        
        let ui_has_mouse = root_ui().is_mouse_over(Vec2::new(mouse_position().0, mouse_position().1));

        let current_position = {
            let mouse_pos = mouse_position();
            Vec2::new(mouse_pos.0, mouse_pos.1)
        };


        if is_key_pressed(KeyCode::F1) {
            self.show_help = !self.show_help;
        }

        if is_key_pressed(KeyCode::F2) {
            self.show_fms_lines = !self.show_fms_lines;
        }
        if is_key_pressed(KeyCode::F3) {
            self.sector_ui.toggle_visibility();
        }
        if is_key_pressed(KeyCode::F11) {
            self.fullscreen = !self.fullscreen;
            window::set_fullscreen(self.fullscreen);
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





    pub fn draw(&mut self) {
        self.position_calculator.invalidated = true;
        self.sector.draw(&mut self.position_calculator, DrawableObjectType::Default);

        for aircraft in self.aircraft_records.iter_mut() {
            aircraft.draw(&mut self.position_calculator, self.show_fms_lines);
        }

        
        if self.show_help {
            draw_text("F1 - Show / hide help    F2 - Toggle FMS lines    F3 - Filters    F11 - Toggle fullscreen", 10., 20.0, 20., WHITE);
        }
        
        self.sector_ui.show_ui(&mut self.sector);
    }
}

impl Drop for RadarDisplay {
    fn drop(&mut self) {
        if let Some(tcp_stream) = &self.tcp_stream {
            tcp_stream.shutdown(std::net::Shutdown::Both).ok();
        }
    }
}