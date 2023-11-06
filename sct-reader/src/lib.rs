#![allow(unused)]

use std::str::FromStr;

use error::Error;
mod reader;
mod position;
mod waypoint;
mod sector;
mod colour;
mod error;
mod partial;

pub type SectorResult<T> = std::result::Result<T, error::Error>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum AirspaceClass {
    A,
    B,
    C,
    D,
    E,
    F,
    G,
}
impl FromStr for AirspaceClass {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let result = match s {
            "A" => AirspaceClass::A,
            "B" => AirspaceClass::B,
            "C" => AirspaceClass::C,
            "D" => AirspaceClass::D,
            "E" => AirspaceClass::E,
            "F" => AirspaceClass::F,
            "G" => AirspaceClass::G,
            _ => return Err(Error::InvalidAirspaceClass),
        };
        Ok(result)
    }
}