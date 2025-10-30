use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::Mutex;

use anyhow::anyhow;
use tracing::info;

use crate::config::SerialPortConfig;
use crate::drivers::SerialPortDriver;
use serial2_tokio::SerialPort;

use pza_toolkit::config::UsbEndpointConfig;
use pza_toolkit::rumqtt::client::RumqttCustomAsyncClient;
///
pub struct StandardDriver {
    /// Configuration
    config: SerialPortConfig,
    // The underlying driver instance
    driver: Option<Arc<Mutex<SerialPort>>>,

    client: Option<RumqttCustomAsyncClient>,
}

impl StandardDriver {
    /// Create a new power supply emulator instance
    pub fn new(config: SerialPortConfig) -> Self {
        Self {
            config,
            driver: None,
            client: None,
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
            let mut name = Some(p.port_name.clone());
            let mut usb = None;

            match &p.port_type {
                serialport::SerialPortType::UsbPort(usb_info) => {
                    usb = Some(UsbEndpointConfig {
                        vid: Some(usb_info.vid),
                        pid: Some(usb_info.pid),
                        serial: usb_info.serial_number.clone(),
                    });
                    // name not required when usb config is provided
                    name = None;
                }
                _ => {}
            }

            println!("Found port: {}", p.port_name);
            result.push(SerialPortConfig {
                model: "standard".to_string(),
                description: None,
                endpoint: Some(crate::config::SerialPortEndpointConfig {
                    name: name,
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
    async fn initialize(&mut self, mqtt_client: RumqttCustomAsyncClient) -> anyhow::Result<()> {
        self.client = Some(mqtt_client);

        // Determine the port name from configuration
        let port_name = match &self.config.endpoint {
            Some(endpoint) => {
                // If name is provided, use it
                if let Some(name) = &endpoint.name {
                    name.clone()
                } else if let Some(usb_config) = &endpoint.usb {
                    // Try to find the port by USB configuration
                    let available_ports = serialport::available_ports()?;

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

                    matching_port.ok_or_else(|| anyhow!("No matching USB device found"))?
                } else {
                    return Err(anyhow!("No port name or USB configuration provided"));
                }
            }
            None => {
                return Err(anyhow!("No endpoint configuration provided"));
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
        let port = SerialPort::open(&port_name, baud_rate)?;

        self.driver = Some(Arc::new(Mutex::new(port)));
        info!(
            "Successfully opened serial port: {} at {} baud",
            port_name, baud_rate
        );

        // Spawn a task for continuous reading from the serial port
        if let (Some(driver), Some(client)) = (self.driver.clone(), self.client.clone()) {
            tokio::spawn(async move {
                let mut buffer = [0u8; 1024];
                loop {
                    // Lock the driver to read from the serial port
                    let mut read_result = {
                        let mut port = driver.lock().await;
                        use tokio::io::AsyncReadExt;
                        port.read(&mut buffer).await
                    };

                    match read_result {
                        Ok(bytes_read) if bytes_read > 0 => {
                            // Convert the read data to bytes and publish via MQTT
                            let data = bytes::Bytes::copy_from_slice(&buffer[..bytes_read]);
                            let topic = client.topic_with_prefix("rx");

                            if let Err(e) = client.publish(topic, data.to_vec()).await {
                                tracing::error!("Failed to publish serial data to MQTT: {}", e);
                            }
                        }
                        Ok(_) => {
                            // No data read, continue loop
                            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
                        }
                        Err(e) => {
                            tracing::error!("Error reading from serial port: {}", e);
                            // Sleep before retrying to avoid busy loop
                            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                        }
                    }
                }
            });
        }

        Ok(())
    }

    /// Shutdown the driver
    async fn shutdown(&mut self) -> anyhow::Result<()> {
        info!("Emulator Driver: shutdown");
        Ok(())
    }

    async fn send(&mut self, bytes: bytes::Bytes) -> anyhow::Result<()> {
        if let Some(driver) = &self.driver {
            let mut port = driver.lock().await;
            use tokio::io::AsyncWriteExt;

            // Write the bytes to the serial port
            port.write_all(&bytes).await?;

            // Ensure the data is sent immediately
            port.flush().await?;

            info!("Sent {} bytes to serial port", bytes.len());
            Ok(())
        } else {
            Err(anyhow!("Serial port not initialized"))
        }
    }
}
