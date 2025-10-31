use crate::client::SerialPortClient;
use pza_toolkit::config::IPEndpointConfig;
use pza_toolkit::rumqtt::client::init_client;

/// Builder pattern for creating SerialPortClient instances
pub struct SerialPortClientBuilder {
    /// Name of the instance
    pub instance_name: Option<String>,

    /// MQTT broker configuration
    pub ip: Option<IPEndpointConfig>,

    /// Enable transmission monitoring
    pub enable_tx_monitoring: bool,
}

impl Default for SerialPortClientBuilder {
    fn default() -> Self {
        Self {
            instance_name: None,
            ip: None,
            enable_tx_monitoring: false, // Explicitly set to false
        }
    }
}

impl SerialPortClientBuilder {
    // ------------------------------------------------------------------------

    /// Create a new builder from broker configuration
    pub fn with_ip(mut self, ip: IPEndpointConfig) -> Self {
        self.ip = Some(ip);
        self
    }

    // ------------------------------------------------------------------------

    /// Set the power supply name for the client
    pub fn with_power_supply_name<A: Into<String>>(mut self, name: A) -> Self {
        self.instance_name = Some(name.into());
        self
    }

    pub fn enable_tx_monitoring(mut self, enable: bool) -> Self {
        self.enable_tx_monitoring = enable;
        self
    }

    // ------------------------------------------------------------------------

    /// Build the SerialPortClient instance
    pub fn build(self) -> anyhow::Result<SerialPortClient> {
        let (client, event_loop) = init_client("serial-port");

        Ok(SerialPortClient::new_with_client(
            self.instance_name.unwrap(),
            client,
            event_loop,
            self.enable_tx_monitoring,
        ))
    }
}
