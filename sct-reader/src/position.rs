use std::fmt::Display;

use crate::{error::Error, SectorResult};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Position {
    pub lat: f64,
    pub lon: f64,
}
impl Position {
    pub fn try_new_from_es(lat: &str, lon: &str) -> SectorResult<Position> {
        let lat = coord_from_es(lat)
            .ok_or(Error::InvalidPosition)?;
        let lon = coord_from_es(lon)
            .ok_or(Error::InvalidPosition)?;
        Ok(Position { lat, lon })
    }
    pub fn validate(self) -> SectorResult<Self> {
        let valid = (-90.0..=90.0).contains(&self.lat) &&
        (-180.0..=180.0).contains(&self.lon);
        return if valid {
            Ok(self)
        } else {
            Err(Error::InvalidPosition)
        };
    }
}


//N051.07.25.010
//E002.39.13.334
pub fn coord_from_es(value: &str) -> Option<f64> {
    let multiply_by = if value.starts_with(&['N', 'n', 'E', 'e']) {
        1.0
    } else {
        -1.0
    };
    let mut sections = value.get(1..)?.splitn(3, '.');
    let degs = sections.next()?.parse::<f64>().ok()?;
    let mins = sections.next()?.parse::<f64>().ok()?;
    let secs = sections.next()?.parse::<f64>().ok()?;

    let coord = degs + (mins / 60.0) + (secs / 3600.0);
    return Some(coord * multiply_by);
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Heading(f32);
impl Heading {
    pub fn new(heading: f32) -> SectorResult<Heading> {
        heading.try_into()
    }
    pub fn new_from_u16(heading: u16) -> SectorResult<Heading> {
        let value: f32 = heading.try_into().map_err(|_| Error::InvalidHeading)?;
        Self::new(value)
    }
    pub fn value(&self) -> f32 {
        self.0
    }
    pub fn value_u16(&self) -> u16 {
        self.0.round() as u16
    }
    pub fn reciprocal(&self) -> Heading {
        let new = if self.0 < 180.0 {
            self.0 + 180.0
        } else {
            self.0 - 180.0
        };
        Heading(new)
    }
}
impl Display for Heading {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:03}", self.0)
    }
}

impl TryFrom<f32> for Heading {
    type Error = Error;
    fn try_from(mut value: f32) -> Result<Self, Self::Error> {
        if value > 360.0 {
            return Err(Error::InvalidHeading);
        }
        if value == 0.0 {
            value = 360.0;
        }
        Ok(Heading(value))
    }
}
