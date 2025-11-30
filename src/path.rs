use pza_toolkit::path::server_configs_dir;
use std::path::PathBuf;

/// Get the path to the server configuration file
///
pub fn server_config_file() -> Option<PathBuf> {
    server_configs_dir().map(|root| {
        root.join(format!(
            "{}-{}.json5",
            crate::constants::FILE_NAME_PREFIX,
            crate::constants::SERVER_TYPE_NAME
        ))
    })
}
