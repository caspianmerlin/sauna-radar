use std::path::PathBuf;

use common::radar_profile::colours::RadarColour;
use macroquad::color::Color;

pub fn get_config_dir() -> Option<PathBuf> {
    let mut dir = dirs::config_dir();
    if let Some(dir) = &mut dir {
        dir.push("sauna-ui-rs");
    }
    dir
}

pub fn radar_colour_to_mq_colour(radar_colour: &RadarColour) -> Color {
    Color::from_rgba(radar_colour.r, radar_colour.g, radar_colour.b, 255)
}