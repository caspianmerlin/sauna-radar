use std::{ops::DerefMut, sync::mpsc::Receiver};

use common::api_requests::text_command::TextCommandRequest;
use macroquad::{texture::Image, color::{GREEN, Color, RED}, ui::{Skin, widgets::{InputText, Editbox}, hash, root_ui, InputHandler, self, Id}, math::Vec2, window::{self, screen_width}, shapes::draw_rectangle, text::draw_text, input::is_key_pressed, miniquad::KeyCode};

use crate::{logger::Logger, aircraft::{AircraftManager, Aircraft}};





#[derive(Debug)]
pub struct Console {
    input_txt: String,
    input_txt_hash: Id,
    input_txt_needs_focus: bool,
    editbox_skin: Skin,
    lines: Vec<String>,
    log_rx: Receiver<String>,
}
impl Console {
    pub fn new(logging_level: log::Level) -> Self {
        Self { input_txt: String::new(), input_txt_hash: hash!("INPUTTXT"), input_txt_needs_focus: false, editbox_skin: editbox_skin(), lines: vec![], log_rx: Logger::initialise(logging_level) }
    }

    fn last_x(&mut self, x: usize) -> Option<&[String]> {
        let start_i = self.lines.len().saturating_sub(x);
        self.lines.get(start_i..)
    }

    pub fn update(&mut self, aircraft_manager: &AircraftManager) -> Option<TextCommandRequest> {
        if let Ok(message) = self.log_rx.try_recv() {
            self.lines.push(message);
        }

        if self.input_txt_needs_focus {
            self.input_txt_needs_focus = false;
            root_ui().set_input_focus(self.input_txt_hash);
        }

        if let Some(callsign) = aircraft_manager.current_selected() {
            if is_key_pressed(KeyCode::Enter) {
                if let Some(text_command_request) = try_parse_text_command(callsign, &self.input_txt, aircraft_manager) {
                    self.input_txt.clear();
                    return Some(text_command_request);
                }
            }
        }

        None
    }

    pub fn handle_log_message(&mut self, log_message: String) {
        self.lines.push(log_message);
    }
    pub fn set_focus_to_input(&mut self) {
        self.input_txt.clear();
        self.input_txt_needs_focus = true;
    }

    pub fn draw(&mut self) {

        let x = 10.0;
        let y = window::screen_height() - 100.0;



        draw_rectangle(x, y, screen_width() - 20.0, 90.0, Color::from_rgba(7, 2, 94, 150));
        let y = y + 15.;
        if let Some(lines) = self.last_x(4) {
            for (i, line) in lines.iter().enumerate() {
                draw_text(line, x + 5., y + (i as f32 * 15.), 18., GREEN);
            }
        }
        

        root_ui().push_skin(&self.editbox_skin);
        InputText::new(self.input_txt_hash).position(Vec2::new(10.0, window::screen_height() - 30.0)).size(Vec2::new(window::screen_width() - 20., 18.0)).ratio(1.0)
        .ui(root_ui().deref_mut(), &mut self.input_txt);
        root_ui().pop_skin();
    }
}




fn editbox_skin() -> Skin {
    let editbox_style = macroquad::ui::root_ui()
    .style_builder()
    .background(
        Image::empty()
    )
    .text_color(GREEN).font_size(20)
    .build();
    
    Skin {
        editbox_style,
        ..macroquad::ui::root_ui().default_skin()
    }
}




fn try_parse_text_command(callsign: &str, txt: &str, aircraft_manager: &AircraftManager) -> Option<TextCommandRequest> {
    let mut split = txt.split_whitespace();

    let command = split.next()?.to_string();
    let args = split.map(|arg| arg.to_owned()).collect::<Vec<_>>();
    let request = TextCommandRequest {
        callsign: callsign.to_owned(),
        command,
        args
    };
    Some(request)
}