use std::{thread::{JoinHandle, self}, sync::{Arc, atomic::{AtomicBool, Ordering}}, path::PathBuf, io::BufReader, fs::File};

use common::{radar_profile::{colours::RadarColours, RadarProfile, LatLon}, position::Position};
use sct_reader::reader::SctReader;

use crate::sector::Sector;

use super::display::RadarDisplay;





pub struct RadarDisplayLoader {
    thread: Option<JoinHandle<Option<Vec<PartiallyLoadedSector>>>>,
    ready: Arc<AtomicBool>,
}
impl RadarDisplayLoader {
    pub fn new() -> Self {
        Self {
            thread: None,
            ready: Arc::new(AtomicBool::new(true)),
        }
    }

    pub fn start_load(&mut self, sct: PathBuf, sym: PathBuf, asr: PathBuf, centre_lat: f32, centre_lon: f32, zoom: f32) {
        if !self.ready.load(Ordering::Relaxed) || self.thread.is_some() {
            println!("Not starting to load");
            return;
        }
        self.ready.store(false, Ordering::Relaxed);
        let ready = Arc::clone(&self.ready);
        self.thread = Some(
            thread::spawn(move || {
                let res = load_sectors(vec![(sct, sym, asr, centre_lat, centre_lon, zoom)]);
                ready.store(true, Ordering::Relaxed);
                res
            })
        );
    }

    pub fn poll(&mut self) -> Option<Vec<RadarDisplay>> {
        if self.ready.load(Ordering::Relaxed) {
            if let Some(thread) = self.thread.take() {
                let result = thread.join().unwrap();
                let result = result.map(|vec| {
                    vec.into_iter().map(|pls| {
                        RadarDisplay::new(pls.sector, pls.window_ht_n_mi, pls.colours)
                    }).collect::<Vec<RadarDisplay>>()
                });
                return result;
            }
        }
        return None;
    }
}

struct PartiallyLoadedSector {
    sector: Sector,
    colours: RadarColours,
    window_ht_n_mi: f32,
}
impl PartiallyLoadedSector {
    fn new(sector: Sector, colours: RadarColours, window_ht_n_mi: f32) -> PartiallyLoadedSector {
        PartiallyLoadedSector { sector, colours, window_ht_n_mi }
    }
}

//(Sector, RadarColours, f32)

fn load_sectors(paths: Vec<(PathBuf, PathBuf, PathBuf, f32, f32, f32)>) -> Option<Vec<PartiallyLoadedSector>> {
    let mut partially_loaded_sectors: Vec<PartiallyLoadedSector> = Vec::with_capacity(paths.len());
        for (sct, sym, asr, centre_lat, centre_lon, zoom) in &paths {
            // Parse the toml file
            // let file = std::fs::read_to_string(path).ok()?;
            // let profile = toml::from_str::<RadarProfile>(&file).ok()?;
            
            // Attempt to load the sector file
            let mut sector: Sector = SctReader::new(BufReader::new(File::open(sct).ok()?))
            .try_read()
            .ok()?.into();

            sector.default_centre_pt = Position::new(*centre_lat, *centre_lon);
            
            // Read and apply the filters
            let filters = common::radar_profile::filters::RadarFilters::read_from_asr_file(asr).ok()?;
            sector.load_filters_from_profile(&filters);
            
            // Read the colours from symbology file
            let colours = common::radar_profile::colours::RadarColours::read_from_symbology_file(sym).ok()?;
            partially_loaded_sectors.push(PartiallyLoadedSector::new(sector, colours, *zoom));
        }
    
    return Some(partially_loaded_sectors);
}