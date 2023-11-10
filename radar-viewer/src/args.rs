use std::path::PathBuf;

use clap::Parser;




#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
pub struct Args {

    /// Path to the .sct or .sct file to open
    #[arg(short = 's', value_name = "SECTOR FILE PATH")]
    pub sector_file: PathBuf,

    /// Port of the Sauna UI client
    #[arg(short, value_name = "PORT")]
    pub port: u16,
    
    /// Filters file to apply filters from and save filters to
    #[arg(short, value_name = "FILTERS FILE PATH")]
    pub filters_file: Option<PathBuf>,   
}