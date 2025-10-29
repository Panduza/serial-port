use async_trait::async_trait;

use pza_toolkit::rumqtt::client::RumqttCustomAsyncClient;
use tracing::info;

use crate::config::SerialPortConfig;
use crate::drivers::DriverError;
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
    async fn initialize(&mut self) -> Result<(), DriverError> {
        info!("Emulator Driver: initialize");
        Ok(())
    }
    /// Shutdown the driver
    async fn shutdown(&mut self) -> Result<(), DriverError> {
        info!("Emulator Driver: shutdown");
        Ok(())
    }

    /// Send data to the power supply
    async fn send(&mut self, _bytes: bytes::Bytes) -> Result<(), DriverError> {
        Ok(())
    }

    /// Set the MQTT client
    fn set_client(&mut self, client: pza_toolkit::rumqtt::client::RumqttCustomAsyncClient) {
        self.client = Some(client);
    }
}
