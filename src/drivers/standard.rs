use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::Mutex;

use tracing::info;

use crate::config::SerialPortConfig;
use crate::drivers::DriverError;
use crate::drivers::SerialPortDriver;
use serial2_tokio::SerialPort;

use pza_toolkit::config::UsbEndpointConfig;
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

    //--------------------------------------------------------------------------

    /// Scan for available devices
    pub fn scan() -> Vec<SerialPortConfig> {
        let mut result = Vec::new();

        serialport::available_ports().unwrap().iter().for_each(|p| {
            let mut usb = None;

            match &p.port_type {
                serialport::SerialPortType::UsbPort(usb_info) => {
                    usb = Some(UsbEndpointConfig {
                        vid: Some(usb_info.vid),
                        pid: Some(usb_info.pid),
                        serial: usb_info.serial_number.clone(),
                    });
                }
                _ => {}
            }

            println!("Found port: {}", p.port_name);
            result.push(SerialPortConfig {
                model: "standard".to_string(),
                description: None,
                endpoint: Some(crate::config::SerialPortEndpointConfig {
                    name: Some(p.port_name.clone()),
                    usb: usb,
                    baud_rate: Some(115200),
                }),
            });
        });

        result
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
