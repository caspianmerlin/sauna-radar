use std::error::Error;

use clap::Parser;

use crate::{args::Args, console::Console, aircraft::AircraftManager, ipc::IpcManager, radar::manager::RadarManager};





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



}