#![allow(unused)]

use std::{fs::File, io::BufReader, sync::{mpsc::{self, Receiver}, Mutex, Arc}, thread, net::TcpStream};

use args::Args;
use asr::Asr;
use clap::Parser;
use macroquad::{
    prelude::{Color, Vec2, BLACK, BLUE, WHITE},
    shapes::{draw_poly_lines, draw_triangle, draw_line},
    text::{draw_text, load_ttf_font_from_bytes, TextParams, draw_text_ex},
    window::{self, clear_background, next_frame}, ui::{root_ui, hash, widgets::{Window, Group, Button, InputText, ComboBox}, Layout},
};
use radar::{line::LineType, position_calc::{PositionCalculator, self}, WINDOW_HT_N_MI, display::{RadarDisplay, TAG_FONT}};

use sct_reader::reader::SctReader;
use sector::{Sector, items::Position};

mod args;
mod asr;
mod radar;
mod sector;


#[macroquad::main("Sauna Radar")]
async fn main() {
    // Get command line args
    let args = Args::parse();
    
    let mut radar_display: Option<RadarDisplay> = None;
    // Attempt to load sector file
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let asr = args.asr_file.map(|path| Asr::from_file(path).unwrap());
        let mut sector: Sector = SctReader::new(BufReader::new(File::open(args.sector_file).unwrap()))
            .try_read()
            .unwrap().into();
        // Set centre airport if there is one
        if let Some(centre_airport) = args.centre_airport {
            if let Some(airport) = sector
                .airports
                .get_by_name(&centre_airport)
            {
                sector.default_centre_pt = airport.position;
            }
        }
        tx.send((sector, asr)).unwrap();
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

        if let Some(radar_display) = &mut radar_display {

            radar_display.update();
            radar_display.draw();



        } else {
            if let Ok((mut new_sector, new_asr)) = rx.try_recv() {
                if let Some(new_asr) = new_asr {
                    new_sector.load_filters_from_asr(&new_asr);
                }

                
                let new_radar_display = RadarDisplay::new(new_sector, args.screen_height_n_mi, start_ipc_worker());
                radar_display = Some(new_radar_display);
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


fn start_ipc_worker() -> ReceiverDropGuard {
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



    return ReceiverDropGuard(rx);
}

pub enum IpcMessage {
    TcpStream(TcpStream),
    AircraftData(Vec<AircraftRecord>),
}

#[derive(Debug)]
pub struct AircraftRecord {
    pub callsign: String,
    pub position: Position,
    pub alt: i32,
    pub fms_lines: Vec<ipc::SimAircraftFmsLine>,
}
impl From<ipc::SimAircraftRecord> for AircraftRecord {
    fn from(value: ipc::SimAircraftRecord) -> Self {
        AircraftRecord { callsign: value.callsign, position: Position { lat: value.lat, lon: value.lon, cached_x: 0.0, cached_y: 0.0 }, alt: value.alt, fms_lines: value.fms_lines }
    }
}

impl AircraftRecord {
    pub fn draw(&mut self, position_calculator: &position_calc::PositionCalculator, show_fms_lines: bool) {
        self.position.cache_screen_coords(position_calculator);

        draw_poly_lines(
            self.position.cached_x,
            self.position.cached_y,
            4,
            5.0,
            45.0,
            1.0,
            WHITE,
        );

        if show_fms_lines {
            for line in &self.fms_lines {
                let mut start_pos = Position {
                    lat: line.start_lat,
                    lon: line.start_lon,
                    cached_x: 0.0,
                    cached_y: 0.0,
                };
                start_pos.cache_screen_coords(position_calculator);
                let mut end_pos = Position {
                    lat: line.end_lat,
                    lon: line.end_lon,
                    cached_x: 0.0,
                    cached_y: 0.0,
                };
                end_pos.cache_screen_coords(position_calculator);

                draw_line(start_pos.cached_x, start_pos.cached_y, end_pos.cached_x, end_pos.cached_y, 1.0, Color::from_rgba(162, 50, 168, 255));
            }
        }


        let font = TAG_FONT.get_or_init(|| {
            load_ttf_font_from_bytes(include_bytes!("../fonts/RobotoMono-Regular.ttf")).unwrap()
        });
        let text_params = TextParams {
            font: Some(font),
            font_size: 16,
            font_scale: 1.0,
            color: WHITE,
            ..Default::default()
        };

        draw_text_ex(&self.callsign, self.position.cached_x, self.position.cached_y + 20.0, text_params.clone());
        draw_text_ex(&self.alt.to_string(), self.position.cached_x, self.position.cached_y + 35.0, text_params);
    }
}

enum InitialisationStage {
    Uninitialised,
    Initialised,
    Running,
}