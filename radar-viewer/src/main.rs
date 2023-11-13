#![allow(unused)]

use std::{fs::File, io::BufReader, sync::{mpsc, Mutex, Arc}, thread, net::TcpStream};

use args::Args;
use asr::Asr;
use clap::Parser;
use macroquad::{
    prelude::{Color, Vec2, BLACK, BLUE, WHITE},
    shapes::{draw_poly_lines, draw_triangle},
    text::draw_text,
    window::{self, clear_background, next_frame}, ui::{root_ui, hash, widgets::{Window, Group, Button, InputText, ComboBox}, Layout},
};
use radar::{line::LineType, position_calc::PositionCalculator, WINDOW_HT_N_MI, display::RadarDisplay};

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
                let arc = start_ipc_worker();
                println!("This should only show once");
                let new_radar_display = RadarDisplay::new(new_sector, args.screen_height_n_mi, start_ipc_worker());
                radar_display = Some(new_radar_display);
            }
        }

        



        macroquad_profiler::profiler(Default::default());
        
        next_frame().await
    }
}







fn start_ipc_worker() -> Arc<Mutex<Vec<AircraftRecord>>> {
    let arc = Arc::new(Mutex::new(vec![]));
    let arc_cloned = Arc::clone(&arc);


    thread::spawn(move || {
        let mut arc = arc_cloned;
        let tcp_stream = TcpStream::connect("127.0.0.1:14416").unwrap();

        loop {
            let aircraft_data: ipc::MessageType = bincode::deserialize_from(&tcp_stream).unwrap();
            match aircraft_data {
                ipc::MessageType::AircraftData(aircraft_data) => {
                    //println!("{:?}", aircraft_data);
                    let aircraft_data = aircraft_data.into_iter().map(AircraftRecord::from).collect::<Vec<_>>();
                    
                    let mut mutex_guard = arc.lock().unwrap();
                    *mutex_guard = aircraft_data;
                }
                _ => (),
            }
            
        }
        
    });



    return arc;
}



#[derive(Debug)]
pub struct AircraftRecord {
    pub callsign: String,
    pub position: Position,
    pub alt: i32,
}
impl From<ipc::SimAircraftRecord> for AircraftRecord {
    fn from(value: ipc::SimAircraftRecord) -> Self {
        AircraftRecord { callsign: value.callsign, position: Position { lat: value.lat, lon: value.lon, cached_x: 0.0, cached_y: 0.0 }, alt: value.alt }
    }
}

enum InitialisationStage {
    Uninitialised,
    Initialised,
    Running,
}