use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::Mutex;

use tracing::info;

use crate::config::SerialPortConfig;
use crate::drivers::DriverError;
use crate::drivers::SerialPortDriver;
use serial2_tokio::SerialPort;

///
pub struct StandardDriver {
    /// Configuration
    config: SerialPortConfig,
    // The underlying driver instance
    driver: Option<Arc<Mutex<SerialPort>>>,
}

impl StandardDriver {
    /// Create a new power supply emulator instance
    pub fn new(config: SerialPortConfig) -> Self {
        Self {
            config,
            driver: None,
        }
    }

    //--------------------------------------------------------------------------

    /// Get the manifest information for this driver
    pub fn manifest() -> serde_json::Value {
        serde_json::json!({
            "model": "standard",
            "description": "A ",
        })
    }
}

#[async_trait]
impl SerialPortDriver for StandardDriver {
    /// Initialize the driver
    async fn initialize(&mut self) -> Result<(), DriverError> {
        // info!("Kd3005p Driver: initialize");
        // let mut dev = ka3005p::find_serial_port().unwrap();

        //
        let port = SerialPort::open("/dev/ttyUSB0", 115200).unwrap();
        self.driver = Some(Arc::new(Mutex::new(port)));
        // let mut buffer = [0; 256];
        // loop {
        //     let read = port.read(&mut buffer).await?;
        //     port.write_all(&buffer[..read]).await?;
        // }

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
