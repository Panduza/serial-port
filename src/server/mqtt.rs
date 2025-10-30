use crate::{constants, drivers::SerialPortDriver};
use bytes::Bytes;
use rumqttc::{AsyncClient, MqttOptions};
use std::{sync::Arc, time::Duration};
use tokio::sync::Mutex;

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
    topic_control_oe: String,
    /// psu/{name}/control/oe/cmd"
    topic_control_oe_cmd: String,

    /// psu/{name}/control/voltage
    topic_control_voltage: String,
    /// psu/{name}/control/voltage/cmd
    topic_control_voltage_cmd: String,

    /// psu/{name}/control/voltage
    topic_control_current: String,
    /// psu/{name}/control/current/cmd
    topic_control_current_cmd: String,

    /// psu/{name}/measure/voltage/refresh_freq
    topic_measure_voltage_refresh_freq: String,
    /// psu/{name}/measure/current/refresh_freq
    topic_measure_current_refresh_freq: String,
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
            topic_control_oe: custom_client.topic_with_prefix("control/oe"),
            topic_control_oe_cmd: custom_client.topic_with_prefix("control/oe/cmd"),
            topic_control_voltage: custom_client.topic_with_prefix("control/voltage"),
            topic_control_voltage_cmd: custom_client.topic_with_prefix("control/voltage/cmd"),
            topic_control_current: custom_client.topic_with_prefix("control/current"),
            topic_control_current_cmd: custom_client.topic_with_prefix("control/current/cmd"),
            topic_measure_voltage_refresh_freq: custom_client
                .topic_with_prefix("measure/voltage/refresh_freq"),
            topic_measure_current_refresh_freq: custom_client
                .topic_with_prefix("measure/current/refresh_freq"),

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
            .subscribe_to_all(vec![runner.topic_control_oe_cmd.clone()])
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

    /// Subscribe to all relevant MQTT topics
    async fn subscribe_to_all(client: AsyncClient, topics: Vec<&String>) {
        for topic in topics {
            client
                .subscribe(topic, rumqttc::QoS::AtMostOnce)
                .await
                .unwrap();
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

    /// Handle output enable/disable commands
    async fn handle_output_enable_command(&self, payload: Bytes) {
        // Handle ON/OFF payload
        let cmd = String::from_utf8(payload.to_vec()).unwrap();
        let mut driver = self.driver.lock().await;
        if cmd == "ON" {
            // driver
            //     .enable_output()
            //     .await
            //     .expect("Failed to enable output");
        } else if cmd == "OFF" {
            // driver
            //     .disable_output()
            //     .await
            //     .expect("Failed to disable output");
        } else {
            // Invalid command
            self.client
                .client
                .publish(
                    self.topic_control_oe.clone(),
                    rumqttc::QoS::AtLeastOnce,
                    true,
                    Bytes::from("ERROR"),
                )
                .await
                .unwrap();
            return;
        }

        // Wait a bit for the device to process the command
        tokio::time::sleep(Duration::from_millis(200)).await;

        // // Read back the actual output enable state to confirm
        // let oe_value = driver.output_enabled().await.expect("Failed to get state");
        // let payload_back = Bytes::from(if oe_value { "ON" } else { "OFF" });

        // // Confirm the new state by publishing it
        // self.client
        //     .publish(
        //         self.topic_control_oe.clone(),
        //         rumqttc::QoS::AtLeastOnce,
        //         true,
        //         payload_back,
        //     )
        //     .await
        //     .unwrap();
    }

    // --------------------------------------------------------------------------------

    /// Handle voltage setting commands
    async fn handle_voltage_command(&self, payload: Bytes) {
        let cmd = String::from_utf8(payload.to_vec()).unwrap();
        let mut driver = self.driver.lock().await;
        // driver
        //     .set_voltage(cmd)
        //     .await
        //     .expect("Failed to set voltage");

        // // Wait a bit for the device to process the command
        // tokio::time::sleep(Duration::from_millis(200)).await;

        // // Read back the actual set voltage to confirm
        // let voltage = driver.get_voltage().await.expect("Failed to get voltage");
        // let payload_back = Bytes::from(voltage);

        // // Confirm the new state by publishing it
        // self.client
        //     .publish(
        //         self.topic_control_voltage.clone(),
        //         rumqttc::QoS::AtLeastOnce,
        //         true,
        //         payload_back,
        //     )
        //     .await
        //     .unwrap();
    }

    // --------------------------------------------------------------------------------

    /// Handle incoming MQTT messages
    /// TODO => handle error return here
    async fn handle_incoming_message(&self, topic: &String, payload: Bytes) {
        // ON/OFF Output Enable
        if topic.eq(&self.topic_control_oe_cmd) {
            self.handle_output_enable_command(payload).await;
        }
        // Set Voltage
        else if topic.eq(&self.topic_control_voltage_cmd) {
            self.handle_voltage_command(payload).await;
        }
        // Set Measurement Refresh Frequencies
        else if topic.eq(&self.topic_measure_voltage_refresh_freq) {
            let cmd = String::from_utf8(payload.to_vec()).unwrap();
            if let Ok(_freq) = cmd.parse::<u64>() {
                // Set voltage measurement refresh frequency
                // (Implementation depends on the driver capabilities)
            }
        } else if topic.eq(&self.topic_measure_current_refresh_freq) {
            let cmd = String::from_utf8(payload.to_vec()).unwrap();
            if let Ok(_freq) = cmd.parse::<u64>() {
                // Set current measurement refresh frequency
                // (Implementation depends on the driver capabilities)
            }
        }
    }
}
