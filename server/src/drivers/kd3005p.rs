use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::Mutex;

use ka3005p::Command;
use ka3005p::Switch;
use tracing::info;

use crate::config::PowerSupplyConfig;
use crate::drivers::DriverError;
use crate::drivers::SerialPortDriver;

use ka3005p::Ka3005p;

/// A power supply emulator for testing and development purposes
pub struct Kd3005pDriver {
    /// Configuration for the power supply
    config: PowerSupplyConfig,

    /// The underlying driver instance
    driver: Option<Arc<Mutex<Ka3005p>>>,
}

impl Kd3005pDriver {
    /// Create a new power supply emulator instance
    pub fn new(config: PowerSupplyConfig) -> Self {
        Self {
            config,
            driver: None,
        }
    }

    //--------------------------------------------------------------------------

    /// Get the manifest information for this driver
    pub fn manifest() -> serde_json::Value {
        serde_json::json!({
            "model": "kd3005p",
            "description": "A simple power supply from Korad",
            "security_min_voltage": Some(0.0_f32),
            "security_max_voltage": Some(30.0_f32),
            "security_min_current": Some(0.0_f32),
            "security_max_current": Some(3.0_f32),
        })
    }
}

#[async_trait]
impl SerialPortDriver for Kd3005pDriver {
    /// Initialize the driver
    async fn initialize(&mut self) -> Result<(), DriverError> {
        info!("Kd3005p Driver: initialize");
        let mut dev = ka3005p::find_serial_port().unwrap();

        dev.execute(Command::Ovp(Switch::On)).unwrap();
        dev.execute(Command::Ocp(Switch::On)).unwrap();

        self.driver = Some(Arc::new(Mutex::new(dev)));

        Ok(())
    }
    /// Shutdown the driver
    async fn shutdown(&mut self) -> Result<(), DriverError> {
        info!("Emulator Driver: shutdown");
        Ok(())
    }

    async fn send(&mut self, _bytes: bytes::Bytes) -> Result<(), DriverError> {
        Ok(())
    }
}
