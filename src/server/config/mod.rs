mod path;
mod tui;
use pza_toolkit::config::MqttBrokerConfig;
pub use pza_toolkit::config::{IPEndpointConfig, SerialPortEndpointConfig};
use serde::{de, Deserialize, Serialize};
use serde_json;
use std::collections::HashMap;
// use std::path::Path;
// use tracing::{error, info};
use pza_toolkit::dioxus::logger::LoggerBuilder;
use tracing::{debug, Level};

use crate::server::config::tui::TuiConfig;
// use crate::constants::DEFAULT_MCP_PORT;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GuiConfig {
    /// Enable or disable the GUI
    pub enable: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct McpServerConfig {
    /// Enable or disable the MCP server
    pub enable: bool,
    /// Bind address of the MCP server
    pub host: String,
    /// Port of the MCP server
    pub port: u16,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SerialPortConfig {
    /// Unique identifier for the power supply
    pub model: String,

    /// Optional description of the power supply
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Serial port configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub endpoint: Option<SerialPortEndpointConfig>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ServerConfig {
    /// GUI configuration
    pub tui: TuiConfig,

    /// MCP server configuration
    pub mcp: McpServerConfig,

    /// MQTT broker configuration
    pub broker: MqttBrokerConfig,

    /// Power supply configurations, keyed by their unique identifiers
    pub runners: Option<HashMap<String, SerialPortConfig>>,
}

impl Default for ServerConfig {
    fn default() -> Self {
        // Create a default power supply configuration for an emulator device
        let mut devices = HashMap::new();
        devices.insert(
            "emulator".to_string(),
            SerialPortConfig {
                model: "emulator".to_string(),
                description: None,
                endpoint: Some(SerialPortEndpointConfig {
                    name: Some("emulator".to_string()),
                    baud_rate: Some(9600),
                    usb: None,
                }),
            },
        );

        Self {
            tui: TuiConfig { enable: Some(true) },
            mcp: McpServerConfig {
                enable: true,
                host: "127.0.0.1".to_string(),
                port: 50051,
            },
            broker: MqttBrokerConfig::default(),
            runners: Some(devices),
        }
    }
}

impl ServerConfig {
    /// Load the global configuration from the configuration file
    ///
    pub fn from_user_file() -> anyhow::Result<Self> {
        let config_path = path::server_config_file()
            .ok_or_else(|| anyhow::anyhow!("Failed to determine server configuration file path"))?;

        pza_toolkit::config::read_config::<ServerConfig>(&config_path)
    }

    /// Apply service overrides from CLI arguments
    ///
    pub fn apply_overrides(mut self, overrides: &crate::server::cli::ServicesOverrides) -> Self {
        // if self.tui.enable.is_none() {
        //     self.tui.enable = Some(true);
        // }
        // if overrides.no_mcp {
        //     self.mcp.enable = false;
        // }
        // if overrides.no_tui {
        //     self.tui.enable = Some(false);
        // }
        // if overrides.no_runners {
        //     self.runners = None;
        // }
        self
    }

    /// List MCP server URLs from the configuration
    ///
    fn list_mcp_servers_urls(&self) -> Vec<String> {
        let mut urls = Vec::new();

        if let Some(runners) = &self.runners {
            for (name, config) in runners {
                let url = format!(
                    "http://{}:{}/{}/{}",
                    self.mcp.host,
                    self.mcp.port,
                    pza_serial_port_client::SERVER_TYPE_NAME,
                    name
                );
                urls.push(url);
            }
        }

        urls
    }

    /// List MCP server URLs as a JSON string
    fn list_mcp_servers_urls_as_json_string(&self) -> String {
        let urls = self.list_mcp_servers_urls();
        serde_json::to_string_pretty(&urls).unwrap_or_else(|_| "[]".to_string())
    }

    /// Print MCP server URLs to stdout
    pub fn print_mcp_servers_urls(&self) {
        let urls_json = self.list_mcp_servers_urls_as_json_string();
        println!("{}", urls_json);
    }
    /// Get the names of all configured runners
    pub fn runner_names(&self) -> Vec<String> {
        match &self.runners {
            Some(runners) => runners.keys().cloned().collect(),
            None => Vec::new(),
        }
    }

    /// Determine if tracing should be enabled based on TUI configuration
    pub fn should_enable_tracing(&self) -> bool {
        // Enable tracing if TUI is disabled
        !self.tui.enable.unwrap_or(false)
    }

    /// Setup tracing based on the configuration
    pub fn setup_tracing(self) -> Self {
        if self.should_enable_tracing() {
            LoggerBuilder::default()
                .with_level(Level::TRACE)
                // .display_target(true)
                .filter_rumqttd()
                .filter_dioxus_core()
                .filter_dioxus_signals()
                .filter_warnings()
                .build()
                .expect("failed to init logger");
        }
        self
    }

    /// Trace the current configuration for debugging purposes
    pub fn trace_config(self) -> Self {
        debug!("Configuration file path: {:?}", path::server_config_file());
        debug!("Configuration after overrides: {:?}", self);
        self
    }
}
