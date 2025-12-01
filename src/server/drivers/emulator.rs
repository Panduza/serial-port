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
    //--------------------------------------------------------------------------

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

        self.client = Some(mqtt_client.clone());

        // Spawn a task to periodically send test data on the rx topic
        tokio::spawn(async move {
            let mut counter = 0u32;
            loop {
                tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

                let test_message = format!("Emulator test message #{}\n", counter);
                let topic = mqtt_client.topic_with_prefix("rx");

                if let Err(e) = mqtt_client.publish(topic, test_message.into_bytes()).await {
                    tracing::error!("Failed to publish emulator test message: {}", e);
                }

                counter += 1;
            }
        });

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
