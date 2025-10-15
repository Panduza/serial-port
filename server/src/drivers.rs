pub mod emulator;
pub mod kd3005p;

use async_trait::async_trait;
use bytes::Bytes;
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

    ///
    async fn send(&mut self, bytes: Bytes) -> Result<(), DriverError>;
}
