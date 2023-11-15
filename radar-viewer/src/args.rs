use std::path::PathBuf;

use clap::Parser;

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Port of the Sauna UI client
    #[arg(short, value_name = "PORT")]
    pub port: u16,

    /// Path to Sauna Radar Profile file
    #[arg(short = 'r', value_name = "PATH")]
    pub radar_profile_paths: Vec<PathBuf>,
}
