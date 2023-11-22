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

    pub fn start_load(&mut self, paths: Vec<PathBuf>) {
        if !self.ready.load(Ordering::Relaxed) || self.thread.is_some() {
            println!("Not starting to load");
            return;
        }
        self.ready.store(false, Ordering::Relaxed);
        let ready = Arc::clone(&self.ready);
        self.thread = Some(
            thread::spawn(move || {
                let res = load_sectors(paths);
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

fn load_sectors(paths: Vec<PathBuf>) -> Option<Vec<PartiallyLoadedSector>> {
    let mut partially_loaded_sectors: Vec<PartiallyLoadedSector> = Vec::with_capacity(paths.len());
        for path in &paths {
            // Parse the toml file
            let file = std::fs::read_to_string(path).ok()?;
            let profile = toml::from_str::<RadarProfile>(&file).ok()?;
            
            // Attempt to load the sector file
            let mut sector: Sector = SctReader::new(BufReader::new(File::open(profile.sector_file).ok()?))
            .try_read()
            .ok()?.into();
            if let Some(LatLon { lat, lon }) = profile.screen_centre {
                sector.default_centre_pt = Position::new(lat, lon);
            }
            // Apply the filters
            sector.load_filters_from_profile(&profile.filters);
            
            partially_loaded_sectors.push(PartiallyLoadedSector::new(sector, profile.colours, profile.zoom_level));
        }
    
    return Some(partially_loaded_sectors);
}