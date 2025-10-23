use pza_toolkit::path::user_root_dir;
use std::path::PathBuf;

///
///
pub fn global_config_file() -> Option<PathBuf> {
    user_root_dir().map(|root| root.join("serial-port-server.json5"))
}

///
///
pub fn factory_manifest_file() -> Option<PathBuf> {
    user_root_dir().map(|root| root.join("serial-port-factory.json5"))
}

///
///
pub fn scan_file() -> Option<PathBuf> {
    user_root_dir().map(|root| root.join("serial-port-scan.json5"))
}
