use std::default;

use macroquad::{ui::{widgets::{Window, ComboBox, Group, InputText, Checkbox}, hash, root_ui, Layout, Ui}, prelude::Vec2};

use super::{Sector, mapped_vec::MappedVec, items::{NamedPoint, LineGroup, PolyGroup}};

static CATEGORIES: &[&str] = &["Airports", "VORs", "NDBs", "Fixes", "ARTCC Boundaries", "ARTCC Boundaries (low)", "ARTCC Boundaries (high)", "Low Airways", "High Airways", "SIDs", "STARs", "Geography", "Regions"];

#[derive(Debug)]
pub struct SectorUi {
    visible: bool,
    selected_section: usize,
    search_terms: [String; 13],
    show_only_visible: bool,
}
impl SectorUi {
    pub fn new() -> SectorUi {
        SectorUi { visible: false, selected_section: 0, search_terms: Default::default(), show_only_visible: false }
    }
    pub fn toggle_visibility(&mut self) {
        self.visible = !self.visible;
    }
    pub fn visible(&self) -> bool {
        self.visible
    }
    pub fn show_ui(&mut self, sector: &mut Sector) {

        if !self.visible { return; }

        for (i, text) in self.search_terms.iter_mut().enumerate() {
            if i == self.selected_section {
                *text = text.to_uppercase();
            } else {
                text.clear();
            }
        }

        Window::new(hash!(), Vec2::new(200.0, 200.0), Vec2::new(300.0, 600.0)).label("Filters").titlebar(true).ui(&mut root_ui(), |ui| {
            Group::new(hash!(), Vec2::new(290., 570.)).layout(Layout::Horizontal).ui(ui, |ui| {
                ComboBox::new(hash!(), CATEGORIES).label("H").ratio(1.0).ui(ui, &mut self.selected_section);
                InputText::new(hash!()).ratio(1.0).ui(ui, &mut self.search_terms[self.selected_section]);
                Checkbox::new(hash!()).label("Only show visible").ui(ui, &mut self.show_only_visible);

                match self.selected_section {
                    0 => named_point_ui(ui, &mut sector.airports, &self.search_terms[self.selected_section], self.show_only_visible),
                    1 => named_point_ui(ui, &mut sector.vors, &self.search_terms[self.selected_section], self.show_only_visible),
                    2 => named_point_ui(ui, &mut sector.ndbs, &self.search_terms[self.selected_section], self.show_only_visible),
                    3 => named_point_ui(ui, &mut sector.fixes, &self.search_terms[self.selected_section], self.show_only_visible),
                    4 => line_group_ui(ui, &mut sector.artcc_entries, &self.search_terms[self.selected_section], self.show_only_visible),
                    5 => line_group_ui(ui, &mut sector.artcc_low_entries, &self.search_terms[self.selected_section], self.show_only_visible),
                    6 => line_group_ui(ui, &mut sector.artcc_high_entries, &self.search_terms[self.selected_section], self.show_only_visible),
                    7 => line_group_ui(ui, &mut sector.low_airways, &self.search_terms[self.selected_section], self.show_only_visible),
                    8 => line_group_ui(ui, &mut sector.high_airways, &self.search_terms[self.selected_section], self.show_only_visible),
                    9 => line_group_ui(ui, &mut sector.sid_entries, &self.search_terms[self.selected_section], self.show_only_visible),
                    10 => line_group_ui(ui, &mut sector.star_entries, &self.search_terms[self.selected_section], self.show_only_visible),
                    11 => line_group_ui(ui, &mut sector.geo_entries, &self.search_terms[self.selected_section], self.show_only_visible),
                    12 => region_ui(ui, &mut sector.regions, &self.search_terms[self.selected_section], self.show_only_visible),
                    _ => unreachable!(),
                }
            });
            
            
        });

    }
}

fn named_point_ui(ui: &mut Ui, named_points: &mut MappedVec<NamedPoint>, search_box: &str, show_only_visible: bool) {
    named_points.for_each(|point: &mut NamedPoint| {
        if show_only_visible {
            if !point.show_identifier && !point.show_symbol {
                return;
            }
        }
        if point.identifier.starts_with(search_box) {
            ui.tree_node(hash!(&point.identifier), &point.identifier, |ui| {
                ui.checkbox(hash!(), "Name", &mut point.show_identifier);
                ui.checkbox(hash!(), "Symbol", &mut point.show_symbol);
            });
        }
    });
}

fn line_group_ui(ui: &mut Ui, line_groups: &mut MappedVec<LineGroup>, search_box: &str, show_only_visible: bool) {
    line_groups.for_each(|line_group| {
        if show_only_visible {
            if !line_group.show {
                return;
            }
        }
        let identifier_uppercase = line_group.identifier.to_uppercase();
        if identifier_uppercase.starts_with(search_box) {
            ui.checkbox(hash!(&line_group.identifier), &line_group.identifier, &mut line_group.show);
        }
    });
}

fn region_ui(ui: &mut Ui, regions: &mut MappedVec<PolyGroup>, search_box: &str, show_only_visible: bool) {
    regions.for_each(|region| {
        if show_only_visible {
            if !region.show {
                return;
            }
        }
        let identifier_uppercase = region.identifier.to_uppercase();
        if identifier_uppercase.starts_with(search_box) {
            ui.checkbox(hash!(&region.identifier), &region.identifier, &mut region.show);
        }
    });
}