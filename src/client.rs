use bytes::Bytes;
use pza_toolkit::rumqtt::client::RumqttCustomAsyncClient;
use rumqttc::AsyncClient;
use tokio::sync::broadcast;

pub mod builder;
pub use builder::SerialPortClientBuilder;

/// Client for interacting with a power supply via MQTT
pub struct SerialPortClient {
    /// Name of the serial port instance
    pub instance_name: String,

    /// MQTT client
    mqtt_client: RumqttCustomAsyncClient,

    /// Channel for receiving output current updates
    rx_channel: (broadcast::Sender<Bytes>, broadcast::Receiver<Bytes>),

    /// Topic for receiving MQTT messages
    topic_rx: String,
}

impl Clone for SerialPortClient {
    fn clone(&self) -> Self {
        Self {
            instance_name: self.instance_name.clone(),
            mqtt_client: self.mqtt_client.clone(),
            rx_channel: (self.rx_channel.0.clone(), self.rx_channel.1.resubscribe()),
            topic_rx: self.topic_rx.clone(),
        }
    }
}

impl SerialPortClient {
    /// Create a new SerialPortClient builder
    pub fn builder() -> SerialPortClientBuilder {
        SerialPortClientBuilder::default()
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

                                client
                                    .handle_incoming_message(&topic, payload)
                                    .await
                                    .expect("error handling incoming message    ");
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
    async fn handle_incoming_message(&self, topic: &String, payload: Bytes) -> anyhow::Result<()> {
        if topic == &self.topic_rx {
            self.rx_channel.0.send(payload)?;
        }
        Ok(())
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
                crate::constants::SERVER_TYPE_NAME,
                psu_name.clone()
            ),
        );

        let (channel_tx, channel_rx) = broadcast::channel(32);

        let obj = Self {
            instance_name: psu_name,
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

    /// Subscribe to output current state changes
    pub fn subscribe_rx(&self) -> broadcast::Receiver<Bytes> {
        self.rx_channel.0.subscribe()
    }

    // ------------------------------------------------------------------------

    pub async fn send(&self, bytes: Bytes) -> anyhow::Result<()> {
        self.mqtt_client
            .publish(self.mqtt_client.topic_with_prefix("tx"), bytes.to_vec())
            .await?;
        Ok(())
    }

    // ------------------------------------------------------------------------
}
