use crate::{constants, drivers::SerialPortDriver};
use bytes::Bytes;
use std::{sync::Arc, time::Duration};
use tokio::sync::Mutex;
use tracing::trace;

use pza_toolkit::rumqtt::client::{init_client, RumqttCustomAsyncClient};

#[derive(Debug)]
/// Handler for the MQTT Runner task
pub struct MqttRunnerHandler {
    /// Task handler
    pub task_handler: tokio::task::JoinHandle<()>,
}

/// MQTT Runner for handling power supply commands and measurements
pub struct MqttRunner {
    /// MQTT client
    client: RumqttCustomAsyncClient,
    /// Instance name
    name: String,

    /// Driver instance
    driver: Arc<Mutex<dyn SerialPortDriver + Send + Sync>>,

    /// psu/{name}/status
    topic_status: String,
    /// psu/{name}/error
    topic_error: String,

    /// psu/{name}/control/oe
    topic_tx: String,
}

impl MqttRunner {
    // --------------------------------------------------------------------------------

    /// Start the runner
    pub fn start(
        name: String,
        driver: Arc<Mutex<dyn SerialPortDriver + Send + Sync>>,
    ) -> anyhow::Result<MqttRunnerHandler> {
        let (client, event_loop) = init_client("tttt");

        let custom_client = RumqttCustomAsyncClient::new(
            client,
            rumqttc::QoS::AtMostOnce,
            true,
            format!("{}/{}", constants::SERVER_TYPE_NAME, name),
        );

        // Create runner object
        let runner = MqttRunner {
            name: name.clone(),
            driver,
            topic_status: custom_client.topic_with_prefix("status"),
            topic_error: custom_client.topic_with_prefix("error"),

            topic_tx: custom_client.topic_with_prefix("tx"),

            client: custom_client,
        };

        let task_handler = tokio::spawn(Self::task_loop(event_loop, runner));

        Ok(MqttRunnerHandler { task_handler })
    }

    // --------------------------------------------------------------------------------

    /// The main async task loop for the MQTT runner
    async fn task_loop(mut event_loop: rumqttc::EventLoop, runner: MqttRunner) {
        // Subscribe to all relevant topics
        runner
            .client
            .subscribe_to_all(vec![runner.topic_tx.clone()])
            .await;

        runner.initialize().await;

        loop {
            while let Ok(event) = event_loop.poll().await {
                match event {
                    rumqttc::Event::Incoming(incoming) => match incoming {
                        rumqttc::Packet::Publish(packet) => {
                            let topic = packet.topic;
                            let payload = packet.payload;
                            runner.handle_incoming_message(&topic, payload).await;
                        }
                        _ => {}
                    },
                    rumqttc::Event::Outgoing(_outgoing) => {}
                }
            }
        }
    }

    // --------------------------------------------------------------------------------

    /// Initialize the runner (if needed)
    async fn initialize(&self) {
        let mut driver = self.driver.lock().await;

        driver
            .initialize(self.client.clone())
            .await
            .expect("Driver init failed");
    }

    // --------------------------------------------------------------------------------

    /// Handle incoming MQTT messages
    /// TODO => handle error return here
    async fn handle_incoming_message(&self, topic: &String, payload: Bytes) {
        // ON/OFF Output Enable
        if topic.eq(&self.topic_tx) {
            trace!("Received TX command on topic {}: {:?}", topic, payload);
            let mut driver = self.driver.lock().await;

            if let Err(e) = driver.send(payload).await {
                tracing::error!("Error sending data to serial port: {}", e);
            }
        }
    }
}
