use crate::config::GlobalConfig;

use bytes::Bytes;
use rand::Rng;
use rumqttc::{AsyncClient, MqttOptions};

use pza_toolkit::config::IPEndpointConfig;
use pza_toolkit::rumqtt_client::RumqttCustomAsyncClient;
use pza_toolkit::rumqtt_init::rumqtt_init_client;
use tokio::sync::broadcast;

// mod data;
// pub use data::MutableData;

// mod error;
// pub use error::ClientError;

/// Builder pattern for creating SerialPortClient instances
pub struct SerialPortClientBuilder {
    /// Name of the power supply unit
    pub psu_name: Option<String>,

    /// MQTT broker configuration
    pub broker: IPEndpointConfig,
}

impl SerialPortClientBuilder {
    /// Create a new builder from user configuration file
    pub fn from_user_config_file() -> Self {
        Self {
            psu_name: None,
            broker: GlobalConfig::from_user_file().broker,
        }
    }

    // ------------------------------------------------------------------------

    /// Create a new builder from broker configuration
    pub fn from_broker_config(broker: IPEndpointConfig) -> Self {
        Self {
            psu_name: None,
            broker,
        }
    }

    // ------------------------------------------------------------------------

    /// Set the power supply name for the client
    pub fn with_power_supply_name<A: Into<String>>(mut self, name: A) -> Self {
        self.psu_name = Some(name.into());
        self
    }

    // ------------------------------------------------------------------------

    /// Build the SerialPortClient instance
    pub fn build(self) -> SerialPortClient {
        let (client, event_loop) = rumqtt_init_client("serial-port");

        SerialPortClient::new_with_client(self.psu_name.unwrap(), client, event_loop)
    }
}

/// Client for interacting with a power supply via MQTT
pub struct SerialPortClient {
    pub psu_name: String,

    mqtt_client: RumqttCustomAsyncClient,

    // send ok
    // receive => channel for real time notifications
    // data buffering (with a size) to allow AI to query recent data
    //
    rx_channel: (broadcast::Sender<Bytes>, broadcast::Receiver<Bytes>),

    ///
    topic_rx: String,
}

impl Clone for SerialPortClient {
    fn clone(&self) -> Self {
        Self {
            psu_name: self.psu_name.clone(),
            mqtt_client: self.mqtt_client.clone(),
            rx_channel: (self.rx_channel.0.clone(), self.rx_channel.1.resubscribe()),
            topic_rx: self.topic_rx.clone(),
        }
    }
}

impl SerialPortClient {
    /// Subscribe to all relevant MQTT topics
    async fn subscribe_to_all(client: AsyncClient, topics: Vec<String>) {
        for topic in topics {
            client
                .subscribe(topic, rumqttc::QoS::AtMostOnce)
                .await
                .unwrap();
        }
    }
    /// Task loop to handle MQTT events and update client state
    async fn task_loop(
        client: SerialPortClient,
        mut event_loop: rumqttc::EventLoop,
        sub_topics: Vec<String>,
    ) {
        // Subscribe to all relevant topics
        client
            .mqtt_client
            .subscribe_to_all(sub_topics.clone())
            .await;

        loop {
            while let Ok(event) = event_loop.poll().await {
                // println!("Notification = {:?}", event);
                // match notification {
                //     Ok(event) => {
                match event {
                    rumqttc::Event::Incoming(incoming) => {
                        // println!("Incoming = {:?}", incoming);

                        match incoming {
                            // rumqttc::Packet::Connect(_) => todo!(),
                            // rumqttc::Packet::ConnAck(_) => todo!(),
                            rumqttc::Packet::Publish(packet) => {
                                // println!("Publish = {:?}", packet);
                                let topic = packet.topic;
                                let payload = packet.payload;

                                client.handle_incoming_message(&topic, payload).await;
                            }

                            _ => {}
                        }
                    }
                    rumqttc::Event::Outgoing(outgoing) => {
                        // println!("Outgoing = {:?}", outgoing);
                        match outgoing {
                            // rumqttc::Outgoing::Publish(packet) => {
                            //     // println!("Publish = {:?}", packet);
                            // }
                            _ => {}
                        }
                    } // }
                      // }
                      // Err(_) => todo!(),
                }
            }
        }
    }

    // ------------------------------------------------------------------------

    /// Handle incoming MQTT messages and update internal state
    async fn handle_incoming_message(&self, topic: &String, payload: Bytes) {
        // if topic == &self.topic_control_oe {
        //     let msg = String::from_utf8(payload.to_vec()).unwrap_or_default();
        //     let enabled = msg.trim().eq_ignore_ascii_case("ON");

        //     // Update internal state
        //     {
        //         let mut data = self.mutable_data.lock().await;
        //         data.enabled = enabled;
        //     }

        //     // Trigger all OE callbacks
        //     let callbacks = self.callbacks.lock().await;
        //     for callback in callbacks.oe_callbacks.values() {
        //         callback(enabled).await;
        //     }
        // } else if topic == &self.topic_control_voltage {
        //     let msg = String::from_utf8(payload.to_vec()).unwrap_or_default();
        //     let voltage_str = msg.trim().to_string();

        //     // Update internal state
        //     {
        //         let mut data = self.mutable_data.lock().await;
        //         data.voltage = voltage_str.clone();
        //     }

        //     // Trigger all voltage callbacks
        //     let callbacks = self.callbacks.lock().await;
        //     for callback in callbacks.voltage_callbacks.values() {
        //         callback(voltage_str.clone()).await;
        //     }
        // } else if topic == &self.topic_control_current {
        //     let msg = String::from_utf8(payload.to_vec()).unwrap_or_default();
        //     let current_str = msg.trim().to_string();

        //     // Update internal state
        //     {
        //         let mut data = self.mutable_data.lock().await;
        //         data.current = current_str.clone();
        //     }

        //     // Trigger all current callbacks
        //     let callbacks = self.callbacks.lock().await;
        //     for callback in callbacks.current_callbacks.values() {
        //         callback(current_str.clone()).await;
        //     }
        // }
    }

    // ------------------------------------------------------------------------

    /// Create a new SerialPortClient with existing MQTT client and event loop
    pub fn new_with_client(
        psu_name: String,
        client: AsyncClient,
        event_loop: rumqttc::EventLoop,
    ) -> Self {
        let cccc = RumqttCustomAsyncClient::new(
            client,
            rumqttc::QoS::AtMostOnce,
            true,
            format!(
                "{}/{}",
                crate::constant::MQTT_TOPIC_PREFIX,
                psu_name.clone()
            ),
        );

        let (channel_tx, channel_rx) = broadcast::channel(32);

        let obj = Self {
            psu_name,
            topic_rx: cccc.topic_with_prefix("rx"),
            mqtt_client: cccc,

            rx_channel: (channel_tx, channel_rx),
        };

        let _task_handler = tokio::spawn(Self::task_loop(
            obj.clone(),
            event_loop,
            vec![obj.topic_rx.clone()],
        ));
        obj
    }

    // ------------------------------------------------------------------------
}
