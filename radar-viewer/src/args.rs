use std::path::PathBuf;

use clap::Parser;

#[derive(Debug, Parser)]
#[command(author, version, about, allow_hyphen_values = true, long_about = None, disable_help_flag = true)]
pub struct Args {

    /// Hostname of the Sauna API
    #[arg(short = 'h', value_name = "API_HOSTNAME")]
    pub api_hostname: String,

    /// Port of the Sauna API
    #[arg(short, value_name = "API_PORT")]
    pub port: u16,

    /// Path to sector file (.sct)
    #[arg(short, value_name = "SECTOR_FILE_PATH")]
    pub sector_file_path: PathBuf,

    /// Path to symbology file (.txt)
    #[arg(short = 'c', value_name = "SYMBOLOGY_FILE_PATH")]
    pub symbology_file_path: PathBuf,

    /// Path to ASR file (.asr)
    #[arg(short, value_name = "ASR_FILE_PATH")]
    pub asr_file_path: PathBuf,

    /// Centre lat
    #[arg(short = 'y', value_name = "CENTRE_LAT")]
    pub centre_lat: f32,

    /// Centre lon
    #[arg(short = 'x', value_name = "CENTRE_LON")]
    pub centre_lon: f32,

    /// Zoom level
    #[arg(short, value_name = "ZOOM_LEVEL")]
    pub zoom_level_n_mi: f32,
}
