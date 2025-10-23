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
        // Determine the port name from configuration
        let port_name = match &self.config.endpoint {
            Some(endpoint) => {
                // If name is provided, use it
                if let Some(name) = &endpoint.name {
                    name.clone()
                } else if let Some(usb_config) = &endpoint.usb {
                    // Try to find the port by USB configuration
                    let available_ports = serialport::available_ports().map_err(|e| {
                        DriverError::Generic(format!("Failed to list available ports: {}", e))
                    })?;

                    let mut matching_port = None;
                    for port_info in available_ports {
                        if let serialport::SerialPortType::UsbPort(usb_info) = &port_info.port_type
                        {
                            let vid_match = usb_config.vid.map_or(true, |vid| vid == usb_info.vid);
                            let pid_match = usb_config.pid.map_or(true, |pid| pid == usb_info.pid);
                            let serial_match = usb_config.serial.as_ref().map_or(true, |serial| {
                                usb_info
                                    .serial_number
                                    .as_ref()
                                    .map_or(false, |usb_serial| usb_serial == serial)
                            });

                            if vid_match && pid_match && serial_match {
                                matching_port = Some(port_info.port_name);
                                break;
                            }
                        }
                    }

                    matching_port.ok_or_else(|| {
                        DriverError::Generic("No matching USB device found".to_string())
                    })?
                } else {
                    return Err(DriverError::Generic(
                        "No port name or USB configuration provided".to_string(),
                    ));
                }
            }
            None => {
                return Err(DriverError::Generic(
                    "No endpoint configuration provided".to_string(),
                ));
            }
        };

        // Get baud rate from configuration or use default
        let baud_rate = self
            .config
            .endpoint
            .as_ref()
            .and_then(|e| e.baud_rate)
            .unwrap_or(115200);

        // Open the serial port
        let port = SerialPort::open(&port_name, baud_rate).map_err(|e| {
            DriverError::Generic(format!("Failed to open port {}: {}", port_name, e))
        })?;

        self.driver = Some(Arc::new(Mutex::new(port)));
        info!(
            "Successfully opened serial port: {} at {} baud",
            port_name, baud_rate
        );

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
