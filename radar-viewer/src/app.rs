use std::{error::Error, ops::DerefMut, sync::{atomic::AtomicBool, Arc}};

use clap::Parser;
use common::{ipc::radar_to_ui, api_requests::ApiRequestType};
use macroquad::{input::{is_key_pressed, mouse_position, is_mouse_button_pressed}, miniquad::{KeyCode, MouseButton}, window::{self, screen_width, screen_height}, text::draw_text, color::{WHITE, Color, GREEN, RED}, shapes::draw_rectangle, math::Vec2, ui::{widgets::InputText, hash, root_ui}};

use crate::{args::Args, console::Console, aircraft::AircraftManager, radar::manager::RadarManager, api_link::{ApiLink, Message}};

const MAX_IPC_MESSAGES: usize = 10;

const HELP_TXT: &str = "F1 - Show / hide help    F2 - Toggle FMS lines    F3 - Filters    F5 - Speed vectors    F6 - Previous display    F7 - Next display    F11 - Toggle fullscreen";




pub struct Application {
    args: Args,
    radar_manager: RadarManager,
    aircraft_manager: AircraftManager,
    api_link: ApiLink,
    console: Console,
    show_help: bool,
    full_screen: bool,
    input: String,
}

impl Application {

    pub fn new(program_wants_to_terminate: Arc<AtomicBool>) -> Result<Self, Box<dyn Error>> {
        let args = Args::parse();
        let console = Console::new(log::Level::Info);
        let radar_manager = RadarManager::new(&args);
        let aircraft_manager = AircraftManager::new();
        let api_link = ApiLink::new(args.api_hostname.clone(), args.port, program_wants_to_terminate, args.terminate_on_connection_fail);

        Ok(
            Self { args, radar_manager, aircraft_manager, api_link, console, show_help: true, full_screen: false, input: String::new() }
        )
    }

    pub fn update(&mut self) {

        // Deal with any key presses
        let mouse_position = mouse_position();
        let ui_has_mouse = root_ui().is_mouse_over(Vec2::new(mouse_position.0, mouse_position.1));
        if let Some(text_command_request) = self.console.update(&mut self.aircraft_manager) {
            self.api_link.send(radar_to_ui::PacketType::ApiRequest(ApiRequestType::TextCommand(text_command_request)));
        }
        if !ui_has_mouse {
            if is_mouse_button_pressed(MouseButton::Left) {
                if let Some(position_calculator) = self.radar_manager.active_display().map(|x| x.position_calculator()) {
                    if let Some(_) = self.aircraft_manager.check_if_ac_clicked(mouse_position, position_calculator) {
                        self.console.set_focus_to_input();
                    }
                }
            }
        }

        if is_key_pressed(KeyCode::F1) {
            self.show_help = !self.show_help;
        }
        else if is_key_pressed(KeyCode::F11) {
            self.full_screen = !self.full_screen;
            window::set_fullscreen(self.full_screen);
        }
        

        // Deal with any packets from the UI
        for message in self.api_link.poll(MAX_IPC_MESSAGES) {
            match message {
                Message::AircraftDataUpdate(aircraft_updates) => self.aircraft_manager.handle_aircraft_updates(aircraft_updates),
                Message::LogMessage(log_message) => self.console.handle_log_message(log_message),
            }
        }

        self.radar_manager.update(&mut self.aircraft_manager);
        //self.aircraft_manager.update();
        //self.console.update();

    }

    pub fn draw(&mut self) {
        // Radar manager must be first
        self.radar_manager.draw(&mut self.aircraft_manager);
        self.console.draw();

        // Console

        
        

        // Draw UI
        self.console.draw();


        if self.show_help {
            draw_text(HELP_TXT, 10., 20.0, 20., WHITE);
        }
    }

}