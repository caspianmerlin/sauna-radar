use std::{io::{BufRead, BufReader, BufWriter}, collections::HashMap, fs::File, time::Instant};

use crate::{SectorResult, sector::Sector, colour::Colour, partial::{PartialSector, BeaconType, MultiLineType, SidStarType}, error::Error};
use std::io::Write;




pub struct SctReader<R: BufRead> {
    source: R,
    current_section: FileSection,
    partial_sector: PartialSector,
    errors: Vec<(String, Error)>,
}
impl<R: BufRead> SctReader<R> {
    pub fn new(source: R) -> Self {
        Self { 
            source,
            current_section: FileSection::ColourDefinitions,
            partial_sector: PartialSector::default(),
            errors: vec![],
        }
    }



    pub fn try_read(mut self) -> SectorResult<Sector> {
        let timer = Instant::now();
        for (mut line_number, line) in self.source.lines().enumerate() {
            let mut line = line?;
            let mut line = line.trim_end();
            line_number += 1;
            
            if line.is_empty() || line.starts_with(';') { continue; }
            if line.contains(';') {
                let mut line_split = line.split(';');
                line = line_split.next().unwrap().trim_end();
            }
            if line.starts_with('[') {
                match parse_file_section(line) {
                    Ok(new_section) => self.current_section = new_section,
                    Err(e) => self.errors.push((line.to_owned(), e)),
                }
                continue;
            }

            let result = match self.current_section {
                FileSection::ColourDefinitions => self.partial_sector.parse_colour_line(line),
                FileSection::Info => self.partial_sector.parse_sector_info_line(line),
                FileSection::Airport => self.partial_sector.parse_airport_line(line),
                FileSection::Runway => self.partial_sector.parse_runway_line(line),
                FileSection::Vor => self.partial_sector.parse_vor_or_ndb_line(line, BeaconType::Vor),
                FileSection::Ndb => self.partial_sector.parse_vor_or_ndb_line(line, BeaconType::Ndb),
                FileSection::Fixes => self.partial_sector.parse_fixes_line(line),
                FileSection::Artcc => self.partial_sector.parse_multi_line_line(line, MultiLineType::Artcc),
                FileSection::ArtccHigh => self.partial_sector.parse_multi_line_line(line, MultiLineType::ArtccHigh),
                FileSection::ArtccLow => self.partial_sector.parse_multi_line_line(line, MultiLineType::ArtccLow),
                FileSection::LowAirway => self.partial_sector.parse_multi_line_line(line, MultiLineType::LowAirway),
                FileSection::HighAirway => self.partial_sector.parse_multi_line_line(line, MultiLineType::HighAirway),
                FileSection::Sid => self.partial_sector.parse_multi_line_maybe_coloured(line, SidStarType::Sid),
                FileSection::Star => self.partial_sector.parse_multi_line_maybe_coloured(line, SidStarType::Star),
                _ => Ok(()),
            };
            if let Err(e) = result {
                self.errors.push((line.to_owned(), e));
            }
        }
        let elapsed = timer.elapsed();
        let mut output = BufWriter::new(File::create("output.txt").unwrap());
        writeln!(output, "Took {} ms", elapsed.as_millis()).unwrap();
        write!(output, "{:#?}", self.partial_sector).unwrap();

        write!(output, "\n\n{:#?}", self.errors).unwrap();



        todo!()
    }

    

}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FileSection {
    ColourDefinitions,
    Info,
    Vor,
    Ndb,
    Airport,
    Runway,
    Fixes,
    Geo,
    LowAirway,
    HighAirway,
    Artcc,
    ArtccHigh,
    ArtccLow,
    Sid,
    Star,
    Regions,
    Labels,
}

fn parse_file_section(value: &str) -> SectorResult<FileSection> {
    let new_section = match value.to_uppercase().as_str() {
        "[INFO]" => FileSection::Info,
        "[AIRPORT]" => FileSection::Airport,
        "[VOR]" => FileSection::Vor,
        "[NDB]" => FileSection::Ndb,
        "[RUNWAY]" => FileSection::Runway,
        "[FIXES]" => FileSection::Fixes,
        "[ARTCC]" => FileSection::Artcc,
        "[ARTCC HIGH]" => FileSection::ArtccHigh,
        "[ARTCC LOW]" => FileSection::ArtccLow,
        "[SID]" => FileSection::Sid,
        "[STAR]" => FileSection::Star,
        "[LOW AIRWAY]" => FileSection::LowAirway,
        "[HIGH AIRWAY]" => FileSection::HighAirway,
        "[GEO]" => FileSection::Geo,
        "[REGIONS]" => FileSection::Regions,
        "[LABELS]" => FileSection::Labels,
        _ => return Err(Error::InvalidFileSection),
    };
    Ok(new_section)
}


#[test]
fn test() {
    let file = File::open(r#"C:\Users\chpme\Desktop\Belfast__thisone\Sector\Belfast.sct"#).unwrap();
    let reader = BufReader::new(file);
    let sct_reader = SctReader::new(reader);
    if let Err(e) = sct_reader.try_read() {
        println!("{}", e);
    }
}