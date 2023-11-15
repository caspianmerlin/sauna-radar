use std::{path::Path, io::{BufReader, BufRead}, fs::File};

use serde::{Serialize, Deserialize};

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct RadarFilters {
    pub airports: Vec<WaypointFilter>,
    pub fixes: Vec<WaypointFilter>,
    pub vors: Vec<WaypointFilter>,
    pub ndbs: Vec<WaypointFilter>,

    pub artcc: Vec<String>,
    pub artcc_low: Vec<String>,
    pub artcc_high: Vec<String>,

    pub sids: Vec<String>,
    pub stars: Vec<String>,

    pub low_airways: Vec<String>,
    pub high_airways: Vec<String>,

    pub geography: Vec<String>,

    pub regions: Vec<String>,
    
    pub free_text: Vec<FreeTextFilter>,

    //REGIONS!
}
impl RadarFilters {
    pub fn read_from_asr_file(file: impl AsRef<Path>) -> Result<RadarFilters, std::io::Error> {

        let file = BufReader::new(File::open(file)?);
        let mut radar_filters = RadarFilters::default();
        for line in file.lines() {
            let line = match line {
                Ok(line) => line,
                Err(_) => continue,
            };

            let sections = line.split(':').collect::<Vec<_>>();
            if sections.len() < 3 {
                continue;
            }

            let line = line.trim();
                if line.is_empty() {
                    continue;
                }
                let sections = line.split(':').collect::<Vec<_>>();
                match sections[0] {
                    "Airports" => {
                        match sections[2] {
                            "name" => get_or_insert_waypoint_filter(&mut radar_filters.airports, sections[1]).show_text = true,
                            "symbol" => get_or_insert_waypoint_filter(&mut radar_filters.airports, sections[1]).show_symbol = true,
                            _ => continue,
                        }
                    },
                    "Fixes" => {
                        match sections[2] {
                            "name" => get_or_insert_waypoint_filter(&mut radar_filters.fixes, sections[1]).show_text = true,
                            "symbol" => get_or_insert_waypoint_filter(&mut radar_filters.fixes, sections[1]).show_symbol = true,
                            _ => continue,
                        }
                    },
                    "VORs" => {
                        match sections[2] {
                            "name" => get_or_insert_waypoint_filter(&mut radar_filters.vors, sections[1]).show_text = true,
                            "symbol" => get_or_insert_waypoint_filter(&mut radar_filters.vors, sections[1]).show_symbol = true,
                            _ => continue,
                        }
                    },
                    "NDBs" => {
                        match sections[2] {
                            "name" => get_or_insert_waypoint_filter(&mut radar_filters.ndbs, sections[1]).show_text = true,
                            "symbol" => get_or_insert_waypoint_filter(&mut radar_filters.ndbs, sections[1]).show_symbol = true,
                            _ => continue,
                        }
                    },
                    "ARTCC boundary" => {
                        insert_filter_if_not_present(&mut radar_filters.artcc, sections[1]);
                    },
                    "ARTCC low boundary" => {
                        insert_filter_if_not_present(&mut radar_filters.artcc_low, sections[1]);
                    },
                    "ARTCC high boundary" => {
                        insert_filter_if_not_present(&mut radar_filters.artcc_high, sections[1]);
                    },
                    "Sids" => {
                        insert_filter_if_not_present(&mut radar_filters.sids, sections[1]);
                    },
                    "Stars" => {
                        insert_filter_if_not_present(&mut radar_filters.stars, sections[1]);
                    },
                    "Low airways" => {
                        insert_filter_if_not_present(&mut radar_filters.low_airways, sections[1]);
                    },
                    "High airways" => {
                        insert_filter_if_not_present(&mut radar_filters.high_airways, sections[1]);
                    },
                    "Geo" => {
                        insert_filter_if_not_present(&mut radar_filters.geography, sections[1]);
                    },
                    "Regions" => {
                        insert_filter_if_not_present(&mut radar_filters.regions, sections[1]);
                    }
                    "Free Text" => {
                        let mut name_field = sections[1].split('\\');
                        let (Some(name), Some(entry)) = (name_field.next(), name_field.next()) else {
                            continue;
                        };
                        let filter = get_or_insert_free_text_filter(&mut radar_filters.free_text, name);
                        insert_filter_if_not_present(&mut filter.entries, entry);
                    }       
                    _ => continue,
                }
        }
        Ok(radar_filters)
    }
}


fn get_or_insert_waypoint_filter<'a>(vec: &'a mut Vec<WaypointFilter>, name: &str) -> &'a mut WaypointFilter {
    let index = vec.iter_mut().position(| entry| entry.name == name).unwrap_or_else(|| {
        let waypoint_filter = WaypointFilter::with_name(name);
        vec.push(waypoint_filter);
        vec.len() - 1
    });
    &mut vec[index]
}

fn get_or_insert_free_text_filter<'a>(vec: &'a mut Vec<FreeTextFilter>, name: &str) -> &'a mut FreeTextFilter {
    let index = vec.iter_mut().position(| entry| entry.name == name).unwrap_or_else(|| {
        let waypoint_filter = FreeTextFilter::with_name(name);
        vec.push(waypoint_filter);
        vec.len() - 1
    });
    &mut vec[index]
}

fn insert_filter_if_not_present(vec: &mut Vec<String>, name: &str) {
    if vec.iter().find(|entry| *entry == name).is_none() {
        vec.push(name.to_owned());
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WaypointFilter {
    pub name: String,
    pub show_symbol: bool,
    pub show_text: bool,
}
impl WaypointFilter {
    fn with_name(name: &str) -> WaypointFilter {
        WaypointFilter { name: name.to_owned(), show_symbol: false, show_text: false }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FreeTextFilter {
    pub name: String,
    pub entries: Vec<String>,
}
impl FreeTextFilter {
    fn with_name(name: &str) -> FreeTextFilter {
        FreeTextFilter { name: name.to_owned(), entries: vec![] }
    }
}