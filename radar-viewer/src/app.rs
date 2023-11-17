use std::error::Error;

use clap::Parser;

use crate::{args::Args, shutdown::LockFile, RadarDisplayManager, console::Console, aircraft::AircraftManager, ipc::IpcManager};





pub struct Application<'a> {
    args: Args,
    lock_file: LockFile<'a>,
    radar_display_manager: RadarDisplayManager,
    aircraft_manager: AircraftManager,
    ipc_manager: IpcManager,
    console: Console,

    show_help: bool,
    full_screen: bool,
}

impl<'a> Application<'a> {

    pub fn new() -> Result<Self, Box<dyn Error>> {
        let args = Args::try_parse()?;
        let lock_file = LockFile::initialise();
        let radar_display_manager = RadarDisplayManager::default();
        let aircraft_manager = AircraftManager::new();
        let ipc_manager = IpcManager::new(args.port);
    }



}