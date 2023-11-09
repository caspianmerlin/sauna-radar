use std::{collections::HashMap, str::FromStr};

use crate::{
    colour::Colour,
    error::Error,
    position::{self, Heading, Position},
    sector::{Line, MaybeColouredLine, MultiLineMaybeColoured},
    waypoint::{self, Airport, Fix, Ndb, RunwayEnd, RunwayModifier, RunwayStrip, Vor},
    AirspaceClass, SectorResult,
};

use self::{artcc::MultiLine, sector_info::PartialSectorInfo};

pub mod artcc;
pub mod sector_info;

#[derive(Debug, Default)]
pub struct PartialSector {
    colours: HashMap<String, Colour>,
    sector_info: PartialSectorInfo,
    airports: Vec<Airport>,
    vors: Vec<Vor>,
    ndbs: Vec<Ndb>,
    fixes: Vec<Fix>,
    artcc_entries: Vec<MultiLineMaybeColoured>,
    artcc_low_entries: Vec<MultiLineMaybeColoured>,
    artcc_high_entries: Vec<MultiLineMaybeColoured>,
    low_airways: Vec<MultiLineMaybeColoured>,
    high_airways: Vec<MultiLineMaybeColoured>,
    sid_entries: Vec<MultiLineMaybeColoured>,
    star_entries: Vec<MultiLineMaybeColoured>,
}
impl PartialSector {
    fn try_fetch_or_decode_colour(&self, value: &str) -> Option<Colour> {
        if let Ok(colour) = Colour::from_str(value) {
            return Some(colour);
        };
        self.colours.get(value).map(|x| *x)
    }
    fn try_fetch_or_decode_lat_lon(&self, lat: &str, lon: &str) -> Option<Position> {
        if let Ok(position) = Position::try_new_from_es(lat, lon) {
            return Some(position);
        }

        for fix in &self.fixes {
            if fix.identifier == lat {
                return Some((fix.position));
            }
        }
        for vor in &self.vors {
            if vor.identifier == lat {
                return Some((vor.position));
            }
        }
        for ndb in &self.ndbs {
            if ndb.identifier == lat {
                return Some((ndb.position));
            }
        }
        for airport in &self.airports {
            if airport.identifier == lat {
                return Some((airport.position));
            }
        }

        return None;
    }

    pub fn parse_colour_line(&mut self, value: &str) -> SectorResult<()> {
        let mut sections = value.split_whitespace();
        let colour_name = sections
            .nth(1)
            .ok_or(Error::InvalidColourDefinition)?
            .to_owned();
        let colour_def = sections.next().ok_or(Error::InvalidColourDefinition)?;
        let colour = colour_def.parse::<Colour>()?;
        self.colours.insert(colour_name, colour);
        Ok(())
    }
    pub fn parse_sector_info_line(&mut self, value: &str) -> SectorResult<()> {
        self.sector_info.parse_line(value)
    }
    pub fn parse_airport_line(&mut self, value: &str) -> SectorResult<()> {
        let mut sections = value.split_whitespace();
        let identifier = sections.next().ok_or(Error::InvalidWaypoint)?.to_owned();
        let tower_frequency = sections.next().ok_or(Error::InvalidWaypoint)?.to_owned();
        let lat = sections.next().ok_or(Error::InvalidWaypoint)?;
        let lon = sections.next().ok_or(Error::InvalidWaypoint)?;
        let position = Position::try_new_from_es(lat, lon)?;
        let airspace_class: AirspaceClass =
            sections.next().ok_or(Error::InvalidWaypoint)?.parse()?;

        let airport = Airport {
            identifier,
            position,
            tower_frequency,
            airspace_class,
            runways: vec![],
        };

        self.airports.push(airport);

        Ok(())
    }

    pub fn parse_runway_line(&mut self, value: &str) -> SectorResult<()> {
        let mut sections = value.split_whitespace();
        let identifier_a = sections.next().ok_or(Error::InvalidRunway)?;
        let identifier_b = sections.next().ok_or(Error::InvalidRunway)?;
        let (number_a, modifier_a) = parse_runway_identifier(identifier_a)?;
        let (number_b, modifier_b) = parse_runway_identifier(identifier_b)?;

        let heading_a = Heading::new(
            sections
                .next()
                .ok_or(Error::InvalidRunway)?
                .parse::<f32>()
                .map_err(|_| Error::InvalidRunway)?,
        )?;
        let heading_b = Heading::new(
            sections
                .next()
                .ok_or(Error::InvalidRunway)?
                .parse::<f32>()
                .map_err(|_| Error::InvalidRunway)?,
        )?;

        let lat_a = sections.next().ok_or(Error::InvalidRunway)?;
        let lon_a = sections.next().ok_or(Error::InvalidRunway)?;

        let lat_b = sections.next().ok_or(Error::InvalidRunway)?;
        let lon_b = sections.next().ok_or(Error::InvalidRunway)?;

        let pos_a = Position::try_new_from_es(lat_a, lon_a)?;
        let pos_b = Position::try_new_from_es(lat_b, lon_b)?;

        let airport = sections.next().ok_or(Error::InvalidRunway)?;
        let airport = self
            .airports
            .iter_mut()
            .find(|entry| entry.identifier == airport)
            .ok_or(Error::InvalidRunway)?;

        let mut runway_end_a = RunwayEnd {
            number: number_a,
            td_threshold_pos: pos_a,
            se_threshold_pos: pos_b,
            modifier: modifier_a,
            magnetic_hdg: heading_a,
        };

        let mut runway_end_b = RunwayEnd {
            number: number_b,
            td_threshold_pos: pos_b,
            se_threshold_pos: pos_a,
            modifier: modifier_b,
            magnetic_hdg: heading_b,
        };

        if number_a > number_b {
            std::mem::swap(&mut runway_end_a, &mut runway_end_b);
        }

        let runway_strip = RunwayStrip {
            end_a: runway_end_a,
            end_b: runway_end_b,
        };

        airport.runways.push(runway_strip);
        Ok(())
    }

    pub fn parse_vor_or_ndb_line(
        &mut self,
        value: &str,
        beacon_type: BeaconType,
    ) -> SectorResult<()> {
        let mut sections = value.split_whitespace();
        let identifier = sections.next().ok_or(Error::InvalidVorOrNdb)?.to_owned();
        let frequency = sections.next().ok_or(Error::InvalidVorOrNdb)?.to_owned();
        let lat = sections.next().ok_or(Error::InvalidVorOrNdb)?;
        let lon = sections.next().ok_or(Error::InvalidVorOrNdb)?;
        let position = Position::try_new_from_es(lat, lon)?;

        match beacon_type {
            BeaconType::Ndb => {
                let ndb = Ndb {
                    identifier,
                    position,
                    frequency,
                };
                self.ndbs.push(ndb);
            }
            BeaconType::Vor => {
                let vor = Vor {
                    identifier,
                    position,
                    frequency,
                };
                self.vors.push(vor);
            }
        }
        Ok(())
    }

    pub fn parse_fixes_line(&mut self, value: &str) -> SectorResult<()> {
        let mut sections = value.split_whitespace();
        let identifier = sections.next().ok_or(Error::InvalidFix)?.to_owned();
        let lat = sections.next().ok_or(Error::InvalidFix)?;
        let lon = sections.next().ok_or(Error::InvalidFix)?;
        let position = Position::try_new_from_es(lat, lon)?;
        let fix = Fix {
            identifier,
            position,
        };
        self.fixes.push(fix);
        Ok(())
    }

    pub fn parse_artcc_or_airway_line(&mut self, value: &str, line_type: ArtccOrAirwayLineType)  -> SectorResult<()>{
        let mut sections = value.split_whitespace().collect::<Vec<_>>();

        // Get the colour from the last section. If there is one, remove that element.
        let colour = sections.last().and_then(|section| self.try_fetch_or_decode_colour(section));
        if colour.is_some() { sections.pop(); };
        
        // Determine whether this is a new section (with a name), or a continuation of a previous section.
        let name = if sections.len() > 4 {
            let first_coord_index = sections.len() - 4;
            Some(sections[0..first_coord_index].join(" "))
        } else if sections.len() == 4 {
            None
        } else {
            return Err(Error::InvalidArtccEntry);
        };

        // Determine which storage to use.
        let storage = match line_type {
            ArtccOrAirwayLineType::Artcc => &mut self.artcc_entries,
            ArtccOrAirwayLineType::ArtccLow => &mut self.artcc_low_entries,
            ArtccOrAirwayLineType::ArtccHigh => &mut self.artcc_high_entries,
            ArtccOrAirwayLineType::LowAirway => &mut self.low_airways,
            ArtccOrAirwayLineType::HighAirway => &mut self.high_airways,
        };

        let element = if let Some(name) = name {
            if let Some(true) = storage.last().map(|element| element.name == name) {
                storage.last_mut().unwrap()
            } else {
                let new_element = MultiLineMaybeColoured { name, lines: vec![]  };
                storage.push(new_element);
                storage.last_mut().unwrap()
            }
        } else {
            
        };


        todo!()
        
    }

    pub fn parse_multi_line_line(
        &mut self,
        value: &str,
        artcc_entry_type: ArtccOrAirwayLineType,
    ) -> SectorResult<()> {
        let mut sections = value.split_whitespace().collect::<Vec<_>>();
        let mut label = String::new();
        let mut length = sections.len();
        if length < 5 {
            return Err(Error::InvalidArtccEntry);
        }
        while length > 4 {
            length -= 1;
            label.push_str(sections.remove(0));
            if length != 4 {
                label.push(' ');
            }
        }

        let mut sections = sections.into_iter();

        let lat = sections.next().ok_or(Error::InvalidFix)?;
        let lon = sections.next().ok_or(Error::InvalidFix)?;
        let pos_a = self
            .try_fetch_or_decode_lat_lon(lat, lon)
            .ok_or(Error::InvalidArtccEntry)?;
        let lat = sections.next().ok_or(Error::InvalidFix)?;
        let lon = sections.next().ok_or(Error::InvalidFix)?;
        let pos_b = self
            .try_fetch_or_decode_lat_lon(lat, lon)
            .ok_or(Error::InvalidArtccEntry)?;
        let line = Line {
            start: pos_a,
            end: pos_b,
        };
        let vec_to_push_to = match artcc_entry_type {
            ArtccOrAirwayLineType::Artcc => &mut self.artcc_entries,
            ArtccOrAirwayLineType::ArtccLow => &mut self.artcc_low_entries,
            ArtccOrAirwayLineType::ArtccHigh => &mut self.artcc_high_entries,
            ArtccOrAirwayLineType::LowAirway => &mut self.low_airways,
            ArtccOrAirwayLineType::HighAirway => &mut self.high_airways,
        };

        if let Some(entry) = vec_to_push_to.iter_mut().find(|entry| entry.name == label) {
            entry.lines.push(line);
        } else {
            vec_to_push_to.push(MultiLine {
                name: label,
                lines: vec![line],
            });
        }

        Ok(())
    }

    pub fn parse_sid_star_line(
        &mut self,
        value: &str,
        sid_star_type: SidStarType,
    ) -> SectorResult<()> {
        let name = value.get(0..26).ok_or(Error::InvalidSidStarEntry)?.trim();
        let mut sections = value
            .get(26..)
            .ok_or(Error::InvalidSidStarEntry)?
            .trim()
            .split_whitespace();
        let lat_a = sections.next().ok_or(Error::InvalidSidStarEntry)?;
        let lon_a = sections.next().ok_or(Error::InvalidSidStarEntry)?;
        let lat_b = sections.next().ok_or(Error::InvalidSidStarEntry)?;
        let lon_b = sections.next().ok_or(Error::InvalidSidStarEntry)?;
        let colour = sections
            .next()
            .and_then(|x| self.try_fetch_or_decode_colour(x));
        let line = self
            .try_fetch_or_decode_lat_lon(lat_a, lon_a)
            .and_then(|start_pos| {
                self.try_fetch_or_decode_lat_lon(lat_b, lon_b)
                    .and_then(|end_pos| {
                        Some(Line {
                            start: start_pos,
                            end: end_pos,
                        })
                    })
            })
            .map(|line| MaybeColouredLine { line, colour });

        let vec = match sid_star_type {
            SidStarType::Sid => &mut self.sid_entries,
            SidStarType::Star => &mut self.star_entries,
        };

        if name.is_empty() {
            let in_progress_entry = vec.last_mut().ok_or(Error::InvalidSidStarEntry)?;
            if let Some(line) = line {
                in_progress_entry.lines.push(line);
            }
        } else {
            let new_entry = MultiLineMaybeColoured {
                name: name.to_owned(),
                lines: if let Some(line) = line {
                    vec![line]
                } else {
                    vec![]
                },
            };
            vec.push(new_entry);
        }
        Ok(())
    }
}

fn parse_runway_identifier(value: &str) -> SectorResult<(u8, RunwayModifier)> {
    let modifier = if value.ends_with('L') {
        RunwayModifier::Left
    } else if value.ends_with('C') {
        RunwayModifier::Centre
    } else if value.ends_with('R') {
        RunwayModifier::Right
    } else if value.ends_with('G') {
        RunwayModifier::Grass
    } else {
        RunwayModifier::None
    };
    let letters_to_trim: &[_] = &['L', 'R', 'C', 'G'];
    let value = value.trim_end_matches(letters_to_trim);

    let mut number: u8 = value.parse().map_err(|_| Error::InvalidRunway)?;
    if number > 36 {
        return Err(Error::InvalidRunway);
    }
    if number == 0 {
        number = 36;
    }

    Ok((number, modifier))
}

pub enum BeaconType {
    Vor,
    Ndb,
}

pub enum ArtccOrAirwayLineType {
    Artcc,
    ArtccHigh,
    ArtccLow,
    LowAirway,
    HighAirway,
}

pub enum SidStarType {
    Sid,
    Star,
}
