use std::error::Error;

use clap::Parser;

use crate::{args::Args, console::Console, aircraft::AircraftManager, ipc::{IpcManager, Message}, radar::manager::RadarManager};

const MAX_IPC_MESSAGES: usize = 10;




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

        // Deal with any packets from the UI
        for message in self.ipc_manager.poll(MAX_IPC_MESSAGES) {
            match message {
                Message::AircraftDataUpdate(aircraft_updates) => self.aircraft_manager.handle_aircraft_updates(&aircraft_updates),
                Message::LogMessage(log_message) => self.console.handle_log_message(&log_message),
            }
        }

        self.radar_manager.update(&mut self.aircraft_manager);
        //self.aircraft_manager.update();
        //self.console.update();

    }

    pub fn draw(&mut self) {
        self.radar_manager.draw(&mut self.aircraft_manager);
        self.console.draw();

        // Draw UI
    }

}