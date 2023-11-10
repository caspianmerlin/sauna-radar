use std::{io::BufReader, fs::File, thread, sync::mpsc};

use args::Args;
use clap::Parser;
use macroquad::{window::{clear_background, next_frame}, prelude::{Color, WHITE, BLACK, GREEN}, text::draw_text};
use sct_reader::{reader::SctReader, sector::Sector};

mod args;



#[macroquad::main("Sauna Radar")]
async fn main() {
    // Get command line args
    let args = Args::parse();
    let mut sector: Option<Sector> = None;
    
    // Attempt to load sector file
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let sector = SctReader::new(BufReader::new(File::open(args.sector_file).unwrap())).try_read().unwrap();
        tx.send(sector).unwrap();
    });

    loop {
        if sector.is_none() {
            if let Ok(new_sector) = rx.try_recv() {
                sector = Some(new_sector);
            }
        }
        let state = if sector.is_some() {
            State::Loaded
        } else {
            State::Loading
        };

        match state {
            State::Loading => {
                clear_background(BLACK);
                draw_text("LOADING...", 5.0, 20.0, 20.0, WHITE);
            },
            State:: Loaded => {
                clear_background(GREEN);
                let text = format!("Loaded {}", sector.as_ref().unwrap().sector_info.name);
                draw_text(&text, 5.0, 20.0, 20.0, BLACK);
            }
        }

        
        
        next_frame().await
    }
}



enum State {
    Loading,
    Loaded,
}