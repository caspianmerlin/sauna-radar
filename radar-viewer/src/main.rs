#![allow(unused)]

use std::{fs::File, io::{BufReader, BufWriter, Write}, sync::{mpsc::{self, Receiver, TryRecvError, Sender}, Mutex, Arc, atomic::{AtomicBool, Ordering}}, thread, net::TcpStream, path::PathBuf, ops::DerefMut, error::Error};

use args::Args;
use asr::Asr;
use clap::Parser;
use fd_lock::{RwLockWriteGuard, RwLock};

use macroquad::{
    prelude::{Color, Vec2, BLACK, BLUE, WHITE},
    shapes::{draw_poly_lines, draw_triangle, draw_line},
    text::{draw_text, load_ttf_font_from_bytes, TextParams, draw_text_ex},
    window::{self, clear_background, next_frame}, ui::{root_ui, hash, widgets::{Window, Group, Button, InputText, ComboBox}, Layout, Skin}, input::is_key_pressed, miniquad::KeyCode, texture::Image, math::RectOffset, color::GREEN,
};

use sct_reader::reader::SctReader;
use sector::{Sector, items::Position};

mod args;
mod asr;
mod radar;
mod sector;
mod console;
mod aircraft;
mod util;
mod shutdown;
mod app;
mod ipc;



#[derive(Default)]
pub struct RadarDisplayManager {
    radar_displays: Vec<RadarDisplay>,
    active_display: usize,
}
impl RadarDisplayManager {
    pub fn store(&mut self, radar_displays: Vec<RadarDisplay>) {
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
    pub fn update(&mut self, aircraft: &mut Vec<AircraftRecord>) {
        self.radar_displays.get_mut(self.active_display).map(|active_display| active_display.update(aircraft));
    }
    pub fn draw(&mut self, aircraft: &mut Vec<AircraftRecord>) {
        self.radar_displays.get_mut(self.active_display).map(|active_display| active_display.draw(aircraft));
    }
    pub fn background_colour(&self) -> Color {
        self.radar_displays.get(self.active_display).map(|active_display| active_display.background_colour()).unwrap_or(BLACK)
    }
}

#[macroquad::main("Sauna Radar")]
async fn main() -> Result<(), Box<dyn Error>> {

    

    



    let (aircraft_receiver, ipc_message_sender, atomic_bool_drop_guard) = start_ipc_worker();
    // Attempt to load sector file
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {

        let mut radar_displays: Vec<(Sector, RadarColours, f32)> = Vec::with_capacity(args.radar_profile_paths.len());
        for path in args.radar_profile_paths.iter() {
            // Parse the toml file
            let file = std::fs::read_to_string(path).unwrap();
            let profile = toml::from_str::<RadarProfile>(&file).unwrap();
            
            // Attempt to load the sector file
            let mut sector: Sector = SctReader::new(BufReader::new(File::open(profile.sector_file).unwrap()))
            .try_read()
            .unwrap().into();
            if let Some(LatLon { lat, lon }) = profile.screen_centre {
                sector.default_centre_pt = Position::new(lat, lon);
            }
            // Apply the filters
            sector.load_filters_from_profile(&profile.filters);
            
            radar_displays.push((sector, profile.colours, profile.zoom_level));
        }

        
        tx.send(radar_displays).unwrap();
    });

    window::set_fullscreen(true);
    let mut initialisation_stage = InitialisationStage::Uninitialised;


    loop {


        // Update aircraft
        match aircraft_receiver.try_recv()  {
            Ok(IpcMessage::TcpStream(tcp_stream)) => {
                println!("Successfully received");
                tcp_stream_killer.store(tcp_stream);
            }
            Ok(IpcMessage::AircraftData(aircraft_data)) => aircraft = aircraft_data,
            Ok(IpcMessage::ConnectionLost) => {
                aircraft.clear();
            }
            Err(e) => {
                if let TryRecvError::Disconnected = e { println!("Try recv error") };
            }
            _ => {},
        }



        if radar_display_manager.is_initialised() {
            if is_key_pressed(KeyCode::F1) {
                show_help = !show_help;
            }
            if is_key_pressed(KeyCode::Enter) {
                if let Some(text_command) = try_parse_text_command(&text_input, &aircraft) {
                    ipc_message_sender.send(IpcMessage::TextCommand(text_command)).ok();
                    text_input.clear();
                }

                root_ui().set_input_focus(txt_input_id);
            }
            if is_key_pressed(KeyCode::Tab) {
                if !text_input.is_empty() {
                    let input = text_input.to_uppercase();
                    for aircraft in aircraft.iter() {
                        if aircraft.callsign.contains(&input) {
                            text_input = format!("{}, ", aircraft.callsign);
                            break;
                        }
                    }
                }
                
            }
            else if is_key_pressed(KeyCode::F11) {
                full_screen = !full_screen;
                window::set_fullscreen(full_screen);
            }
            else if is_key_pressed(KeyCode::F6) {
                radar_display_manager.cycle_back();
            }
            else if is_key_pressed(KeyCode::F7) {
                radar_display_manager.cycle();
            }
            radar_display_manager.update(&mut aircraft);

            clear_background(radar_display_manager.background_colour());
            radar_display_manager.draw(&mut aircraft);

            if show_help {
                draw_text("F1 - Show / hide help    F2 - Toggle FMS lines    F3 - Filters    F6 - Previous display    F7 - Next display    F11 - Toggle fullscreen", 10., 20.0, 20., WHITE);
            }
        }

        else {
            clear_background(radar_display_manager.background_colour());
            if let Ok(new_radar_displays) = rx.try_recv() {
                let radar_displays = new_radar_displays.into_iter().map(|(sector, colours, zoom_level)| {
                    RadarDisplay::new(sector, zoom_level, colours)
                }).collect();
                radar_display_manager.store(radar_displays);
            }
        }

        
        root_ui().push_skin(&editbox_skin);
        InputText::new(txt_input_id).position(Vec2::new(10.0, window::screen_height() - 30.0)).size(Vec2::new(window::screen_width() - 20., 20.0))
        .ui(root_ui().deref_mut(), &mut text_input);
        root_ui().pop_skin();
        macroquad_profiler::profiler(Default::default());
        
        next_frame().await
    }
}



#[derive(Debug)]
pub struct AtomicBoolDropGuard(Arc<AtomicBool>);
impl Drop for AtomicBoolDropGuard {
    fn drop(&mut self) {
        self.0.store(true, Ordering::Relaxed);
    }
}

pub enum IpcMessage {
    TcpStream(TcpStream),
    AircraftData(Vec<AircraftRecord>),
    ConnectionLost,
    TextCommand(TextCommandRequest),
}





pub fn radar_colour_to_mq_colour(radar_colour: &RadarColour) -> Color {
    Color::from_rgba(radar_colour.r, radar_colour.g, radar_colour.b, 255)
}







fn try_parse_text_command(txt: &str, aircraft_list: &Vec<AircraftRecord>) -> Option<TextCommandRequest> {
    let mut split = txt.split(&[',', ' ']).filter(|x| !x.is_empty());
    let callsign = split.next()?.to_string();
    if !aircraft_list.iter().any(|aircraft| aircraft.callsign == callsign) {
        return None;
    }
    let command = split.next()?.to_string();
    let args = split.map(|arg| arg.to_owned()).collect::<Vec<_>>();
    let request = TextCommandRequest {
        callsign,
        command,
        args
    };
    Some(request)
}