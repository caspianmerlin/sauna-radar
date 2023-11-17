use std::path::PathBuf;

use serde::{Serialize, Deserialize};

use self::{colours::RadarColours, filters::RadarFilters};

pub mod colours;
pub mod filters;



#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RadarProfile {
    pub name: String,
    pub zoom_level: f32,
    pub sector_file: PathBuf,
    pub screen_centre: Option<LatLon>,
    pub colours: RadarColours,
    pub filters: RadarFilters,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LatLon {
    pub lat: f32,
    pub lon: f32,
}

#[test]
fn test_create_profile() {
    let name = String::from("EGAA Approach");
    let sector_file = PathBuf::from(r#"C:\Users\chpme\AppData\Roaming\EuroScope\UK\Belfast\Sector\Belfast.sct"#);
    let zoom_level = 70.0;
    let screen_centre = Some(LatLon { lat: 54.5325, lon: -5.9975 });
    let colours = RadarColours::read_from_symbology_file(r#"C:\Users\chpme\AppData\Roaming\EuroScope\UK\Belfast\Settings\Symbology.txt"#).unwrap();
    let filters = RadarFilters::read_from_asr_file(r#"C:\Users\chpme\AppData\Roaming\EuroScope\UK\Belfast\Settings\EGAA.asr"#).unwrap();
    let radar_profile = RadarProfile {
        name,
        sector_file,
        zoom_level,
        screen_centre,
        colours,
        filters
    };
    
    let toml = toml::to_string_pretty(&radar_profile).unwrap();
    println!("{toml}");
}