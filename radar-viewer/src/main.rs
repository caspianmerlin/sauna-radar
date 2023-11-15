#![allow(unused)]

use std::{fs::File, io::BufReader, sync::{mpsc::{self, Receiver, TryRecvError}, Mutex, Arc}, thread, net::TcpStream};

use args::Args;
use asr::Asr;
use clap::Parser;
use ipc::profile::{RadarProfile, colours::{RadarColour, RadarColours}, LatLon};
use macroquad::{
    prelude::{Color, Vec2, BLACK, BLUE, WHITE},
    shapes::{draw_poly_lines, draw_triangle, draw_line},
    text::{draw_text, load_ttf_font_from_bytes, TextParams, draw_text_ex},
    window::{self, clear_background, next_frame}, ui::{root_ui, hash, widgets::{Window, Group, Button, InputText, ComboBox}, Layout}, input::is_key_pressed, miniquad::KeyCode,
};
use radar::{line::LineType, position_calc::{PositionCalculator, self}, WINDOW_HT_N_MI, display::{RadarDisplay, TAG_FONT}, aircraft::AircraftRecord};

use sct_reader::reader::SctReader;
use sector::{Sector, items::Position};

mod args;
mod asr;
mod radar;
mod sector;

#[derive(Default)]
struct TcpStreamKiller(Option<TcpStream>);
impl Drop for TcpStreamKiller {
    fn drop(&mut self) {
        if let Some(tcp_stream) = &self.0 {
            tcp_stream.shutdown(std::net::Shutdown::Both).ok();
        }
    }
}
impl TcpStreamKiller {
    pub fn store(&mut self, tcp_stream: TcpStream) {
        self.0 = Some(tcp_stream);
    }
}

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
async fn main() {
    // Get command line args
    let args = Args::parse();
    let mut tcp_stream_killer = TcpStreamKiller::default();
    let mut aircraft = Vec::new();
    let mut radar_display_manager = RadarDisplayManager::default();
    let mut show_help = true;
    let mut full_screen = false;
    let aircraft_receiver = start_ipc_worker();
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
        match initialisation_stage {
            InitialisationStage::Uninitialised => initialisation_stage = InitialisationStage::Initialised,
            InitialisationStage::Initialised => {
                window::set_fullscreen(false);
                initialisation_stage = InitialisationStage::Running;
            },
            InitialisationStage::Running => {},
        }

        // Update aircraft
        match aircraft_receiver.try_recv()  {
            Ok(IpcMessage::TcpStream(tcp_stream)) => {
                println!("Successfully received");
                tcp_stream_killer.store(tcp_stream);
            }
            Ok(IpcMessage::AircraftData(aircraft_data)) => aircraft = aircraft_data,
            Err(e) => {
                if let TryRecvError::Disconnected = e { println!("Try recv error") };
            }
        }



        if radar_display_manager.is_initialised() {
            if is_key_pressed(KeyCode::F1) {
                show_help = !show_help;
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

        



        macroquad_profiler::profiler(Default::default());
        
        next_frame().await
    }
}



#[derive(Debug)]
pub struct ReceiverDropGuard(Receiver<IpcMessage>);
impl Drop for ReceiverDropGuard {
    fn drop(&mut self) {
        println!("Rx dropped");
    }
}


fn start_ipc_worker() -> Receiver<IpcMessage> {
    let (tx, rx) = std::sync::mpsc::channel();


    thread::spawn(move || {
        let tx = tx;
        let tcp_stream = TcpStream::connect("127.0.0.1:14416").unwrap();
        tx.send(IpcMessage::TcpStream(tcp_stream.try_clone().unwrap()));
        loop {
            let aircraft_data = match bincode::deserialize_from(&tcp_stream) {
                Ok(ipc::MessageType::AircraftData(aircraft_data)) => aircraft_data,
                Err(_) => break,
            };
            let aircraft_data = aircraft_data.into_iter().map(AircraftRecord::from).collect::<Vec<_>>();
            if let Err(e) = tx.send(IpcMessage::AircraftData(aircraft_data)) {
                println!("{}", e);
            }
        }
        println!("Thread exiting");
    });

    return rx;
}

pub enum IpcMessage {
    TcpStream(TcpStream),
    AircraftData(Vec<AircraftRecord>),
}



enum InitialisationStage {
    Uninitialised,
    Initialised,
    Running,
}


pub fn radar_colour_to_mq_colour(radar_colour: &RadarColour) -> Color {
    Color::from_rgba(radar_colour.r, radar_colour.g, radar_colour.b, 255)
}