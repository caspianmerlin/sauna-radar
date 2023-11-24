use serde::{Serialize, Deserialize};


const EARTH_RADIUS_M: f32 = 6_371_000.0;

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default, Serialize, Deserialize)]
pub struct Position {
    pub lat: f32,
    pub lon: f32,

    /// Indicated altitude. Display as FL if the aircraft has standard set.
    pub alt: f32,
}

impl From<sct_reader::position::Position<sct_reader::position::Valid>> for Position {
    fn from(value: sct_reader::position::Position<sct_reader::position::Valid>) -> Self {
        Position {
            lat: value.lat as f32,
            lon: value.lon as f32,
            alt: 0.0,
        }
    }
}
impl Position {
    pub fn new(lat: f32, lon: f32) -> Position {
        Position {
            lat,
            lon,
            alt: 0.0,
        }
    }
    pub fn new_with_alt(lat: f32, lon: f32, alt: f32) -> Position {
        Position { lat, lon, alt }
    }

    pub fn get_point_at_dist_and_brg(&self, distance_m: f32, bearing: f32) -> Position {
        let origin_lat = self.lat.to_radians();
        let origin_lon = self.lon.to_radians();
        let a = bearing.to_radians();
        let lat = (origin_lat.sin() * (distance_m / EARTH_RADIUS_M).cos() + origin_lat.cos() * (distance_m / EARTH_RADIUS_M).sin() * a.cos()).asin();

        let lon = origin_lon + (a.sin() * (distance_m / EARTH_RADIUS_M).sin() * origin_lat.cos()).atan2((distance_m / EARTH_RADIUS_M).cos() - origin_lat.sin() * lat.sin());

        Position { lat: lat.to_degrees(), lon: lon.to_degrees(), alt: 0.0 }
    }
}