use async_trait::async_trait;

use pza_toolkit::rumqtt::client::RumqttCustomAsyncClient;
use tracing::info;

use crate::config::SerialPortConfig;
use crate::drivers::SerialPortDriver;

/// A power supply emulator for testing and development purposes
pub struct PowerSupplyEmulator {
    client: Option<RumqttCustomAsyncClient>,
}

impl PowerSupplyEmulator {
    /// Create a new power supply emulator instance
    pub fn new(config: SerialPortConfig) -> Self {
        Self { client: None }
    }

    //--------------------------------------------------------------------------

    /// Get the manifest information for this driver
    pub fn manifest() -> serde_json::Value {
        serde_json::json!({
            "model": "emulator",
            "description": "A simple power supply emulator for testing and development purposes.",
        })
    }
}

#[async_trait]
impl SerialPortDriver for PowerSupplyEmulator {
    /// Initialize the driver
    async fn initialize(&mut self, mqtt_client: RumqttCustomAsyncClient) -> anyhow::Result<()> {
        info!("Emulator Driver: initialize");

        self.client = Some(mqtt_client);

        Ok(())
    }
    /// Shutdown the driver
    async fn shutdown(&mut self) -> anyhow::Result<()> {
        info!("Emulator Driver: shutdown");
        Ok(())
    }

    /// Send data to the power supply
    async fn send(&mut self, _bytes: bytes::Bytes) -> anyhow::Result<()> {
        Ok(())
    }
}
