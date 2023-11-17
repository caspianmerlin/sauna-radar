use std::path::PathBuf;

pub fn get_config_dir() -> Option<PathBuf> {
    let mut dir = dirs::config_dir();
    if let Some(dir) = &mut dir {
        dir.push("sauna-ui-rs");
    }
    dir
}