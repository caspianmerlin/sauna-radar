use ipc::filters::FilterSettings;
use macroquad::{window, prelude::Color};
use sct_reader::line::{ColouredLine, Line as SectorLine};

use super::{line::{Line, LineType}, position_calc::PositionCalculator};

#[derive(Debug)]
pub struct RadarDisplay {
    position_calculator: PositionCalculator,
    filter_settings: FilterSettings,
}