use std::io::BufRead;
use std::{fmt::Display, fs::File, io::BufReader, path::Path};

#[derive(Debug, Default)]
pub struct Asr {
    pub airports: Vec<String>,
    pub artcc_boundary: Vec<String>,
    pub artcc_high_boundary: Vec<String>,
    pub artcc_low_boundary: Vec<String>,
    pub geo: Vec<String>,
    pub low_airways: Vec<String>,
    pub high_airways: Vec<String>,
    pub fixes: Vec<String>,
    pub vors: Vec<String>,
    pub ndbs: Vec<String>,
    pub sids: Vec<String>,
    pub stars: Vec<String>,
    pub regions: Vec<String>,
}
impl Asr {
    pub fn from_file(file_path: impl AsRef<Path>) -> Result<Asr, Error> {
        let file = BufReader::new(File::open(file_path)?);
        let mut asr = Asr::default();
        for line in file.lines() {
            if let Ok(line) = line {
                let line = line.trim();
                if line.is_empty() {
                    continue;
                }
                let sections = line.split(':').collect::<Vec<_>>();
                match sections[0] {
                    "Airports" => {
                        asr.airports.push(sections[1].to_owned());
                    }
                    "ARTCC boundary" => {
                        asr.artcc_boundary.push(sections[1].to_owned());
                    }
                    "ARTCC high boundary" => {
                        asr.artcc_high_boundary.push(sections[1].to_owned());
                    }
                    "ARTCC low boundary" => {
                        asr.artcc_low_boundary.push(sections[1].to_owned());
                    }
                    "Fixes" => {
                        asr.fixes.push(sections[1].to_owned());
                    }
                    "Geo" => {
                        asr.geo.push(sections[1].to_owned());
                    }
                    "Low airways" => {
                        asr.low_airways.push(sections[1].to_owned());
                    }
                    "High airways" => {
                        asr.high_airways.push(sections[1].to_owned());
                    }
                    "NDBs" => {
                        asr.ndbs.push(sections[1].to_owned());
                    }
                    "Stars" => {
                        asr.stars.push(sections[1].to_owned());
                    }
                    "Sids" => {
                        asr.sids.push(sections[1].to_owned());
                    }
                    "VORs" => {
                        asr.vors.push(sections[1].to_owned());
                    }
                    "Regions" => {
                        asr.regions.push(sections[1].to_owned());
                    }

                    _ => {}
                }
            }
        }
        Ok(asr)
    }
}

#[derive(Debug)]
pub enum Error {
    Io(std::io::ErrorKind),
}
impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Error::Io(_) => "IO Error",
            }
        )
    }
}
impl std::error::Error for Error {}
impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Error::Io(value.kind())
    }
}
