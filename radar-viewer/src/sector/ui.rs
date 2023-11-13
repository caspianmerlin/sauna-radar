use std::default;

use macroquad::{ui::{widgets::{Window, ComboBox, Group, InputText}, hash, root_ui, Layout}, prelude::Vec2};

use super::Sector;

static CATEGORIES: &[&str] = &["Airports", "VORs", "NDBs", "Fixes", "ARTCC Boundaries", "ARTCC Boundaries (low)", "ARTCC Boundaries (high)", "Low Airways", "High Airways", "SIDs", "STARs", "Geography", "Regions"];

#[derive(Debug)]
pub struct SectorUi {
    visible: bool,
    selected_section: usize,
    search_terms: [String; 13],
}
impl SectorUi {
    pub fn new() -> SectorUi {
        SectorUi { visible: false, selected_section: 0, search_terms: Default::default() }
    }
    pub fn toggle_visibility(&mut self) {
        self.visible = !self.visible;
    }
    pub fn show_ui(&mut self, sector: &mut Sector) {
        if !self.visible { return; }

        Window::new(hash!(), Vec2::new(200.0, 200.0), Vec2::new(300.0, 600.0)).label("Filters").titlebar(true).ui(&mut root_ui(), |ui| {
            Group::new(hash!(), Vec2::new(290., 570.)).layout(Layout::Vertical).ui(ui, |ui| {
                ComboBox::new(hash!(), CATEGORIES).label(" ").ratio(1.0).ui(ui, &mut self.selected_section);
                InputText::new(hash!()).ratio(1.0).ui(ui, &mut self.search_terms[0]);
                sector.ui_window(ui, &mut self.search_terms[0]);
            });
            
            
        });

    }
}