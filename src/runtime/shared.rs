use std::path::PathBuf;

use crate::framework::prelude::*;

pub fn generate_session_id() -> String {
    uuid(5)
}

pub fn lattice_config_dir() -> Option<PathBuf> {
    dirs::config_dir().map(|config_dir| config_dir.join("Lattice"))
}
