use std::error::Error;

use clap::Parser;
use macroquad::{input::is_key_pressed, miniquad::KeyCode, window, text::draw_text, color::WHITE};

use crate::{args::Args, console::Console, aircraft::AircraftManager, ipc::{IpcManager, Message}, radar::manager::RadarManager};

const MAX_IPC_MESSAGES: usize = 10;

const HELP_TXT: &str = "F1 - Show / hide help    F2 - Toggle FMS lines    F3 - Filters    F6 - Previous display    F7 - Next display    F11 - Toggle fullscreen";




pub struct Application {
    args: Args,
    radar_manager: RadarManager,
    aircraft_manager: AircraftManager,
    ipc_manager: IpcManager,
    console: Console,

    show_help: bool,
    full_screen: bool,
}

impl Application {

    pub fn new() -> Result<Self, Box<dyn Error>> {
        let args = Args::try_parse()?;
        let console = Console::new();
        let radar_manager = RadarManager::new(args.radar_profile_paths.clone());
        let aircraft_manager = AircraftManager::new();
        let ipc_manager = IpcManager::new(args.port);

        Ok(
            Self { args, radar_manager, aircraft_manager, ipc_manager, console, show_help: true, full_screen: false }
        )
    }

    pub fn update(&mut self) {

        // Deal with any key presses
        if is_key_pressed(KeyCode::F1) {
            self.show_help = !self.show_help;
        }
        else if is_key_pressed(KeyCode::F11) {
            self.full_screen = !self.full_screen;
            window::set_fullscreen(self.full_screen);
        }
        

        // Deal with any packets from the UI
        for message in self.ipc_manager.poll(MAX_IPC_MESSAGES) {
            match message {
                Message::AircraftDataUpdate(aircraft_updates) => self.aircraft_manager.handle_aircraft_updates(aircraft_updates),
                Message::LogMessage(log_message) => self.console.handle_log_message(&log_message),
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

        // Draw UI

        if self.show_help {
            draw_text(HELP_TXT, 10., 20.0, 20., WHITE);
        }
    }

}