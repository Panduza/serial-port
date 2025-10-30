use pza_toolkit::config::MqttBrokerConfig;
pub use pza_toolkit::config::{IPEndpointConfig, SerialPortEndpointConfig};
use serde::{de, Deserialize, Serialize};
use serde_json;
use std::collections::HashMap;
use std::path::Path;
use tracing::{error, info};

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
pub struct ServerMainConfig {
    /// GUI configuration
    pub gui: GuiConfig,

    /// MCP server configuration
    pub mcp: McpServerConfig,

    /// MQTT broker configuration
    pub broker: MqttBrokerConfig,

    /// Power supply configurations, keyed by their unique identifiers
    pub devices: Option<HashMap<String, SerialPortConfig>>,
}

impl Default for ServerMainConfig {
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
            gui: GuiConfig { enable: true },
            mcp: McpServerConfig {
                enable: false,
                host: "127.0.0.1".to_string(),
                port: 50051,
            },
            broker: MqttBrokerConfig::default(),
            devices: Some(devices),
        }
    }
}

impl ServerMainConfig {
    /// Load the global configuration from the configuration file
    ///
    pub fn from_user_file() -> anyhow::Result<Self> {
        let config_path = crate::path::server_config_file()
            .ok_or_else(|| anyhow::anyhow!("Failed to determine server configuration file path"))?;

        pza_toolkit::config::read_config::<ServerMainConfig>(&config_path)
    }
}
