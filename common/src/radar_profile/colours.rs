use std::{io::{BufRead, BufReader}, path::Path, str::FromStr, fs::File};

use serde::{Serialize, Deserialize};

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct RadarColours {
    pub background: RadarColour,

    pub airports_symbol: RadarColour,
    pub airports_name: RadarColour,

    pub fixes_symbol: RadarColour,
    pub fixes_name: RadarColour,

    pub vors_symbol: RadarColour,
    pub vors_name: RadarColour,

    pub ndbs_symbol: RadarColour,
    pub ndbs_name: RadarColour,

    pub artcc: RadarColour,
    pub artcc_low: RadarColour,
    pub artcc_high: RadarColour,

    pub sids: RadarColour,
    pub stars: RadarColour,

    pub low_airways: RadarColour,
    pub high_airways: RadarColour,

    pub geography: RadarColour,

    pub free_text: RadarColour,
}

impl RadarColours {
    pub fn read_from_symbology_file<P: AsRef<Path>>(symbology_file: P) -> Result<RadarColours, std::io::Error> {

        let mut radar_colours = RadarColours::default();

        let file = BufReader::new(File::open(symbology_file)?);
        for line in file.lines() {
            let line = match line {
                Ok(line) => line,
                Err(_) => continue,
            };

            let sections = line.split(':').collect::<Vec<_>>();
            if sections.len() < 3 {
                continue;
            }

            let category = sections[0].to_uppercase();
            match category.as_str() {
                "AIRPORTS" => {
                    let sub_category = sections[1].to_uppercase();
                    match sub_category.as_str() {
                        "NAME" => radar_colours.airports_name = sections[2].parse().unwrap_or_default(),
                        "SYMBOL" => radar_colours.airports_symbol = sections[2].parse().unwrap_or_default(),
                        _ => continue,
                    }
                },

                "FIXES" => {
                    let sub_category = sections[1].to_uppercase();
                    match sub_category.as_str() {
                        "NAME" => radar_colours.fixes_name = sections[2].parse().unwrap_or_default(),
                        "SYMBOL" => radar_colours.fixes_symbol = sections[2].parse().unwrap_or_default(),
                        _ => continue,
                    }
                },

                "VORS" => {
                    let sub_category = sections[1].to_uppercase();
                    match sub_category.as_str() {
                        "NAME" => radar_colours.vors_name = sections[2].parse().unwrap_or_default(),
                        "SYMBOL" => radar_colours.vors_symbol = sections[2].parse().unwrap_or_default(),
                        _ => continue,
                    }
                },

                "NDBS" => {
                    let sub_category = sections[1].to_uppercase();
                    match sub_category.as_str() {
                        "NAME" => radar_colours.ndbs_name = sections[2].parse().unwrap_or_default(),
                        "SYMBOL" => radar_colours.ndbs_symbol = sections[2].parse().unwrap_or_default(),
                        _ => continue,
                    }
                },

                "ARTCC BOUNDARY" => radar_colours.artcc = sections[2].parse().unwrap_or_default(),
                "ARTCC LOW BOUNDARY" => radar_colours.artcc_low = sections[2].parse().unwrap_or_default(),
                "ARTCC HIGH BOUNDARY" => radar_colours.artcc_high = sections[2].parse().unwrap_or_default(),

                "SIDS" => radar_colours.sids = sections[2].parse().unwrap_or_default(),
                "STARS" => radar_colours.stars = sections[2].parse().unwrap_or_default(),
                "GEO" => radar_colours.geography = sections[2].parse().unwrap_or_default(),

                "LOW AIRWAYS" => {
                    let sub_category = sections[1].to_uppercase();
                    if sub_category == "LINE" {
                        radar_colours.low_airways = sections[2].parse().unwrap_or_default();
                    }
                }
                "HIGH AIRWAYS" => {
                    let sub_category = sections[1].to_uppercase();
                    if sub_category == "LINE" {
                        radar_colours.high_airways = sections[2].parse().unwrap_or_default();
                    }
                }
                "OTHER" => {
                    let sub_category = sections[1].to_uppercase();
                    if sub_category == "FREETEXT" {
                        radar_colours.free_text = sections[2].parse().unwrap_or_default();
                    }
                },
                "SECTOR" => {
                    let sub_category = sections[1].to_uppercase();
                    if sub_category == "ACTIVE SECTOR BACKGROUND" {
                        radar_colours.background = sections[2].parse().unwrap_or_default();
                    }
                },

                _ => continue,
            }
        }
        Ok(radar_colours)
    }
}



#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RadarColour {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}
impl RadarColour {
    pub const fn new(r: u8, g: u8, b: u8) -> RadarColour {
        RadarColour { r, g, b }
    }
}

impl Default for RadarColour {
    fn default() -> Self {
        Self::new(38, 94, 97)
    }
}

impl From<u32> for RadarColour {
    fn from(value: u32) -> Self {
        let r = (value & 0xFF) as u8;
        let g = ((value >> 8) & 0xFF) as u8;
        let b = ((value >> 16) & 0xFF) as u8;
        Self { r, g, b }
    }
}
impl FromStr for RadarColour {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.parse::<u32>()
            .map_err(|_| ())
            .map(Self::from)
    }
}