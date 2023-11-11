use std::path::PathBuf;

use clap::Parser;

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Path to the .sct or .sct file to open
    #[arg(short, value_name = "SECTOR FILE PATH")]
    pub sector_file: PathBuf,

    /// Port of the Sauna UI client
    #[arg(short, value_name = "PORT")]
    pub port: u16,

    /// Filters file to apply filters from and save filters to
    #[arg(short, value_name = "ASR FILE PATH")]
    pub asr_file: Option<PathBuf>,

    #[arg(short, value_name = "AIRPORT")]
    pub centre_airport: Option<String>,

    #[arg(short = 'h', value_name = "NAUTICAL MILES")]
    pub screen_height_n_mi: Option<u32>,
}
