#![allow(unused)]

use std::{fs::File, io::BufReader, sync::mpsc, thread};

use args::Args;
use asr::Asr;
use clap::Parser;
use macroquad::{
    prelude::{Color, Vec2, BLACK, BLUE, WHITE},
    shapes::{draw_poly_lines, draw_triangle},
    text::draw_text,
    window::{self, clear_background, next_frame},
};
use radar::{line::LineType, position_calc::PositionCalculator, WINDOW_HT_N_MI, display::RadarDisplay};

use sct_reader::reader::SctReader;
use sector::Sector;

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


    loop {

        if let Some(radar_display) = &mut radar_display {

            radar_display.update();
            radar_display.draw();



        } else {
            if let Ok((mut new_sector, new_asr)) = rx.try_recv() {
                if let Some(new_asr) = new_asr {
                    new_sector.load_filters_from_asr(&new_asr);
                }
                let new_radar_display = RadarDisplay::new(new_sector, args.screen_height_n_mi);
                radar_display = Some(new_radar_display);
            }
        }

        macroquad_profiler::profiler(Default::default());
        next_frame().await
    }
}

