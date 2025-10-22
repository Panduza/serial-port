use async_trait::async_trait;

use tracing::info;

use crate::config::PowerSupplyConfig;
use crate::drivers::DriverError;
use crate::drivers::SerialPortDriver;

/// A power supply emulator for testing and development purposes
pub struct PowerSupplyEmulator {
    state_oe: bool,
    #[allow(dead_code)]
    voltage: String,
    #[allow(dead_code)]
    current: String,

    security_min_voltage: Option<f32>,
    security_max_voltage: Option<f32>,
    security_min_current: Option<f32>,
    security_max_current: Option<f32>,
}

impl PowerSupplyEmulator {
    /// Create a new power supply emulator instance
    pub fn new(config: PowerSupplyConfig) -> Self {
        Self {
            state_oe: false,
            voltage: "0".into(),
            current: "0".into(),
            security_min_voltage: config.security_min_voltage,
            security_max_voltage: config.security_max_voltage,
            security_min_current: config.security_min_current,
            security_max_current: config.security_max_current,
        }
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
}
