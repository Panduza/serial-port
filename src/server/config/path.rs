use pza_toolkit::path::server_configs_dir;
use std::path::PathBuf;

/// Get the path to the server configuration file
///
pub fn server_config_file() -> Option<PathBuf> {
    server_configs_dir().map(|root| {
        root.join(format!(
            "pza-{}.json5",
            pza_serial_port_client::SERVER_TYPE_NAME
        ))
    })
}
