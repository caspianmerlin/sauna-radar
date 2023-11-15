use crate::{asr::Asr, radar_colour_to_mq_colour};

use self::{items::*, mapped_vec::MappedVec, draw::{Draw, DrawableObjectType}};
use std::collections::HashMap;
use ipc::profile::{filters::{RadarFilters, WaypointFilter}, colours::RadarColours};
use macroquad::{prelude::Color, ui::{Ui, hash}};
use sct_reader::waypoint::Waypoint;

pub mod draw;
pub mod items;
pub mod mapped_vec;
pub mod ui;

#[derive(Debug)]
pub struct Sector {
    pub name: String,
    pub default_centre_pt: Position,
    pub n_mi_per_deg_lat: f32,
    pub n_mi_per_deg_lon: f32,
    pub magnetic_variation: f32,

    pub airports: MappedVec<NamedPoint>,
    pub vors: MappedVec<NamedPoint>,
    pub ndbs: MappedVec<NamedPoint>,
    pub fixes: MappedVec<NamedPoint>,

    pub artcc_entries: MappedVec<LineGroup>,
    pub artcc_low_entries: MappedVec<LineGroup>,
    pub artcc_high_entries: MappedVec<LineGroup>,
    pub low_airways: MappedVec<LineGroup>,
    pub high_airways: MappedVec<LineGroup>,
    pub sid_entries: MappedVec<LineGroup>,
    pub star_entries: MappedVec<LineGroup>,
    pub geo_entries: MappedVec<LineGroup>,

    pub regions: MappedVec<PolyGroup>,

    pub labels: MappedVec<LabelGroup>,
}

impl Sector {
    pub fn load_filters_from_profile(&mut self, filters: &RadarFilters) {
        fn load_waypoint_settings_to_mapped_vec_named_points(filters: &Vec<WaypointFilter>, mapped_vec: &mut MappedVec<NamedPoint>) {
            for filter in filters {
                if let Some(sector_item) = mapped_vec.get_by_name_mut(&filter.name) {
                    sector_item.set_visibility(filter.show_symbol);
                    sector_item.show_identifier = filter.show_text;
                }
            }
        }

        load_waypoint_settings_to_mapped_vec_named_points(&filters.airports, &mut self.airports);
        load_waypoint_settings_to_mapped_vec_named_points(&filters.vors, &mut self.vors);
        load_waypoint_settings_to_mapped_vec_named_points(&filters.ndbs, &mut self.ndbs);
        load_waypoint_settings_to_mapped_vec_named_points(&filters.fixes, &mut self.fixes);

        fn load_string_settings_to_mapped_vec<V>(filters: &Vec<String>, mapped_vec: &mut MappedVec<V>) where V: SetVisibility {
            for filter in filters {
                if let Some(sector_item) = mapped_vec.get_by_name_mut(&filter) {
                    sector_item.set_visibility(true);
                }
            }
        }

        load_string_settings_to_mapped_vec(&filters.artcc, &mut self.artcc_entries);
        load_string_settings_to_mapped_vec(&filters.artcc_low, &mut self.artcc_low_entries);
        load_string_settings_to_mapped_vec(&filters.artcc_high, &mut self.artcc_high_entries);
        load_string_settings_to_mapped_vec(&filters.low_airways, &mut self.low_airways);
        load_string_settings_to_mapped_vec(&filters.high_airways, &mut self.high_airways);
        load_string_settings_to_mapped_vec(&filters.sids, &mut self.sid_entries);
        load_string_settings_to_mapped_vec(&filters.stars, &mut self.star_entries);
        load_string_settings_to_mapped_vec(&filters.geography, &mut self.geo_entries);
        load_string_settings_to_mapped_vec(&filters.regions, &mut self.regions);

        for filter_label_group in filters.free_text.iter() {
            if let Some(label_group) = self.labels.get_by_name_mut(&filter_label_group.name) {
                for filter_label in filter_label_group.entries.iter() {
                    for label in label_group.labels.entries() {
                        if filter_label == &label.text {
                            label.set_visibility(true);
                        }
                    }
                }
            };
        }
        
    }
    
    pub fn ui_window(&mut self, ui: &mut Ui, search: &str) {
        self.fixes.for_each(|fix| {
            if fix.identifier.starts_with(search) {
                ui.checkbox(hash!(&fix.identifier), &fix.identifier, &mut fix.show_symbol);
            }
            
        });
    }

    pub fn draw(&mut self, position_calculator: &crate::radar::position_calc::PositionCalculator, colours: &RadarColours) {
        self.regions.for_each(|region| {
            region.draw(position_calculator);
        });
        self.geo_entries.for_each(|entry| {
            entry.draw(position_calculator, radar_colour_to_mq_colour(&colours.geography));
        });
        self.artcc_entries.for_each(|entry| {
            entry.draw(position_calculator, radar_colour_to_mq_colour(&colours.artcc));
        });
        self.artcc_low_entries.for_each(|entry| {
            entry.draw(position_calculator, radar_colour_to_mq_colour(&colours.artcc_low));
        });
        self.artcc_high_entries.for_each(|entry| {
            entry.draw(position_calculator, radar_colour_to_mq_colour(&colours.artcc_high));
        });
        self.low_airways.for_each(|entry| {
            entry.draw(position_calculator, radar_colour_to_mq_colour(&colours.low_airways));
        });
        self.high_airways.for_each(|entry| {
            entry.draw(position_calculator, radar_colour_to_mq_colour(&colours.high_airways));
        });
        self.sid_entries.for_each(|entry| {
            entry.draw(position_calculator, radar_colour_to_mq_colour(&colours.sids));
        });
        self.star_entries.for_each(|entry| {
            entry.draw(position_calculator, radar_colour_to_mq_colour(&colours.stars));
        });
        
        self.fixes.for_each(|entry| {
            entry.draw(position_calculator, radar_colour_to_mq_colour(&colours.fixes_symbol), radar_colour_to_mq_colour(&colours.fixes_name), DrawableObjectType::Fix);
        });
        self.vors.for_each(|entry| {
            entry.draw(position_calculator, radar_colour_to_mq_colour(&colours.vors_symbol), radar_colour_to_mq_colour(&colours.vors_name), DrawableObjectType::Vor);
        });
        self.ndbs.for_each(|entry| {
            entry.draw(position_calculator, radar_colour_to_mq_colour(&colours.ndbs_symbol), radar_colour_to_mq_colour(&colours.ndbs_name), DrawableObjectType::Ndb);
        });
        self.airports.for_each(|entry| {
            entry.draw(position_calculator, radar_colour_to_mq_colour(&colours.airports_symbol), radar_colour_to_mq_colour(&colours.airports_name), DrawableObjectType::Airport);
        });
        self.labels.for_each(|entry| {
            entry.labels.for_each(|entry| {
                entry.draw(position_calculator, radar_colour_to_mq_colour(&colours.free_text));
            }); 
            
        });
    }
}

impl From<sct_reader::sector::Sector> for Sector {
    fn from(value: sct_reader::sector::Sector) -> Self {
        let name = value.sector_info.name;
        let default_centre_pt = value.sector_info.default_centre_pt.into();
        let n_mi_per_deg_lat = value.sector_info.n_mi_per_deg_lat as f32;
        let n_mi_per_deg_lon = value.sector_info.n_mi_per_deg_lon as f32;
        let magnetic_variation = value.sector_info.magnetic_variation as f32;

        fn mapped_vec_from_waypoints<W: Waypoint>(value: Vec<W>) -> MappedVec<NamedPoint> {
            let mut mapped_vec = MappedVec::new();
            value.into_iter().for_each(|wp| {
                let named_point: NamedPoint = wp.into();
                let key = named_point.identifier.clone();
                mapped_vec.insert(key, named_point);
            });
            mapped_vec
        }

        let airports = mapped_vec_from_waypoints(value.airports);
        let vors = mapped_vec_from_waypoints(value.vors);
        let ndbs = mapped_vec_from_waypoints(value.ndbs);
        let fixes = mapped_vec_from_waypoints(value.fixes);

        fn mapped_vec_from_line_group(
            value: Vec<sct_reader::line::LineGroup<sct_reader::line::ColouredLine>>,
        ) -> MappedVec<LineGroup> {
            let mut mapped_vec = MappedVec::with_capacity(value.len());
            value.into_iter().for_each(|entry| {
                let line_group = LineGroup::from(entry);
                let key = line_group.identifier.clone();
                mapped_vec.insert(key, line_group);
            });
            mapped_vec
        }

        let artcc_entries = mapped_vec_from_line_group(value.artcc_entries);
        let artcc_low_entries = mapped_vec_from_line_group(value.artcc_low_entries);
        let artcc_high_entries = mapped_vec_from_line_group(value.artcc_high_entries);
        let low_airways = mapped_vec_from_line_group(value.low_airways);
        let high_airways = mapped_vec_from_line_group(value.high_airways);
        let sid_entries = mapped_vec_from_line_group(value.sid_entries);
        let star_entries = mapped_vec_from_line_group(value.star_entries);
        let geo_entries = mapped_vec_from_line_group(value.geo_entries);

        let mut regions = MappedVec::with_capacity(value.regions.len());
        value.regions.into_iter().for_each(|region_group| {
            let poly_group = PolyGroup::from(region_group);
            let key = poly_group.identifier.clone();
            regions.insert(key, poly_group);
        });

        let mut label_groups = MappedVec::with_capacity(value.labels.len());
        value.labels.into_iter().for_each(|sct_label_group| {
            let mut label_group = LabelGroup  { name: sct_label_group.name.clone(), labels: MappedVec::with_capacity(sct_label_group.labels.len()) };
            for sector_label in sct_label_group.labels {
                let label = Label::from(sector_label);
                let label = Label::from(label);
                let key = label.text.clone();
                label_group.labels.insert(key, label);
            }
            label_groups.insert(sct_label_group.name, label_group);
        });
        Sector { name, default_centre_pt, n_mi_per_deg_lat, n_mi_per_deg_lon, magnetic_variation, airports, vors, ndbs, fixes, artcc_entries, artcc_low_entries, artcc_high_entries, low_airways, high_airways, sid_entries, star_entries, geo_entries, regions, labels: label_groups }
    }

    

}
