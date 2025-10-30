use crate::client::SerialPortClient;
use pza_toolkit::config::IPEndpointConfig;
use pza_toolkit::rumqtt::client::init_client;

#[derive(Default)]
/// Builder pattern for creating SerialPortClient instances
pub struct SerialPortClientBuilder {
    /// Name of the power supply unit
    pub psu_name: Option<String>,

    /// MQTT broker configuration
    pub ip: Option<IPEndpointConfig>,
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
        self.psu_name = Some(name.into());
        self
    }

    // ------------------------------------------------------------------------

    /// Build the SerialPortClient instance
    pub fn build(self) -> anyhow::Result<SerialPortClient> {
        let (client, event_loop) = init_client("serial-port");

        Ok(SerialPortClient::new_with_client(
            self.psu_name.unwrap(),
            client,
            event_loop,
        ))
    }
}
