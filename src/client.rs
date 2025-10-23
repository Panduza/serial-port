use crate::config::GlobalConfig;

use bytes::Bytes;
use rand::Rng;
use rumqttc::{AsyncClient, MqttOptions};
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;

use pza_toolkit::config::IPEndpointConfig;
use pza_toolkit::rumqtt_client::RumqttCustomAsyncClient;
use pza_toolkit::rumqtt_init::rumqtt_init_client;

// mod data;
// pub use data::MutableData;

// mod error;
// pub use error::ClientError;

use std::collections::HashMap;

/// Generate MQTT topic for a given power supply and suffix
fn psu_topic<A: Into<String>, B: Into<String>>(name: A, suffix: B) -> String {
    format!("power-supply/{}/{}", name.into(), suffix.into())
}

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

    mqtt_client: AsyncClient,

    /// psu/{name}/control/oe
    topic_control_oe: String,
    /// psu/{name}/control/oe/cmd
    topic_control_oe_cmd: String,

    /// psu/{name}/control/voltage
    topic_control_voltage: String,
    /// psu/{name}/control/voltage/cmd
    topic_control_voltage_cmd: String,

    /// psu/{name}/control/current
    topic_control_current: String,
    /// psu/{name}/control/current/cmd
    topic_control_current_cmd: String,
}

impl Clone for SerialPortClient {
    fn clone(&self) -> Self {
        Self {
            psu_name: self.psu_name.clone(),
            mqtt_client: self.mqtt_client.clone(),
            // mutable_data: Arc::clone(&self.mutable_data),
            topic_control_oe: self.topic_control_oe.clone(),
            topic_control_oe_cmd: self.topic_control_oe_cmd.clone(),
            topic_control_voltage: self.topic_control_voltage.clone(),
            topic_control_voltage_cmd: self.topic_control_voltage_cmd.clone(),
            topic_control_current: self.topic_control_current.clone(),
            topic_control_current_cmd: self.topic_control_current_cmd.clone(),
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
        Self::subscribe_to_all(client.mqtt_client.clone(), sub_topics.clone()).await;

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
        // Prepare MQTT topics
        let topic_control_oe = psu_topic(psu_name.clone(), "control/oe");
        let topic_control_oe_cmd = psu_topic(psu_name.clone(), "control/oe/cmd");
        // let topic_control_oe_error = psu_topic(psu_name.clone(), "control/oe/error");
        let topic_control_voltage = psu_topic(psu_name.clone(), "control/voltage");
        let topic_control_voltage_cmd = psu_topic(psu_name.clone(), "control/voltage/cmd");
        let topic_control_current = psu_topic(psu_name.clone(), "control/current");
        let topic_control_current_cmd = psu_topic(psu_name.clone(), "control/current/cmd");
        // let topic_measure_voltage_refresh_freq =
        //     psu_topic(psu_name.clone(), "measure/voltage/refresh_freq");
        // let topic_measure_current_refresh_freq =
        //     psu_topic(psu_name.clone(), "measure/current/refresh_freq");

        let obj = Self {
            psu_name,
            mqtt_client: client,

            topic_control_oe,
            topic_control_oe_cmd,
            // topic_control_oe_error,
            topic_control_voltage,
            topic_control_voltage_cmd,
            topic_control_current,
            topic_control_current_cmd,
            // topic_measure_voltage_refresh_freq,
            // topic_measure_current_refresh_freq,
        };

        let _task_handler = tokio::spawn(Self::task_loop(
            obj.clone(),
            event_loop,
            vec![
                obj.topic_control_oe.clone(),
                obj.topic_control_voltage.clone(),
                obj.topic_control_current.clone(),
            ],
        ));
        obj
    }

    // ------------------------------------------------------------------------

    /// Publish a message to a topic
    pub async fn publish<A: Into<String>>(
        &self,
        topic: A,
        payload: Bytes,
    ) -> Result<(), rumqttc::ClientError> {
        self.mqtt_client
            .publish(topic.into(), rumqttc::QoS::AtLeastOnce, false, payload)
            .await
    }

    // ------------------------------------------------------------------------
}
