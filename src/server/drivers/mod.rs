pub mod emulator;
pub mod standard;

use async_trait::async_trait;
use bytes::Bytes;
use pza_toolkit::rumqtt::client::RumqttCustomAsyncClient;
use thiserror::Error as ThisError;

#[async_trait]
pub trait SerialPortDriver: Send + Sync {
    // --- Lifecycle management ---

    /// Initialize the driver
    async fn initialize(&mut self, mqtt_client: RumqttCustomAsyncClient) -> anyhow::Result<()>;
    /// Shutdown the driver
    async fn shutdown(&mut self) -> anyhow::Result<()>;

    /// Send bytes through the serial port
    async fn send(&mut self, bytes: Bytes) -> anyhow::Result<()>;
}
