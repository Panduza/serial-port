pub mod emulator;
pub mod standard;

use async_trait::async_trait;
use bytes::Bytes;
use pza_toolkit::rumqtt_client::RumqttCustomAsyncClient;
use thiserror::Error as ThisError;

#[derive(ThisError, Debug, Clone)]
pub enum DriverError {
    #[error("An error occurred: {0}")]
    Generic(String),
}

#[async_trait]
pub trait SerialPortDriver: Send + Sync {
    // --- Lifecycle management ---

    /// Initialize the driver
    async fn initialize(&mut self) -> Result<(), DriverError>;
    /// Shutdown the driver
    async fn shutdown(&mut self) -> Result<(), DriverError>;

    /// Send bytes through the serial port
    async fn send(&mut self, bytes: Bytes) -> Result<(), DriverError>;

    /// Set the MQTT client
    ///
    fn set_client(&mut self, client: RumqttCustomAsyncClient);
}
