pub mod emulator;
pub mod standard;

use async_trait::async_trait;
use bytes::Bytes;
use pza_toolkit::rumqtt::client::RumqttCustomAsyncClient;
use thiserror::Error as ThisError;

#[async_trait]
pub trait SerialPortDriver: Send + Sync {
    // --- Lifecycle management ---

    /// Initialize the driver
    async fn initialize(&mut self, mqtt_client: RumqttCustomAsyncClient) -> anyhow::Result<()>;
    /// Shutdown the driver
    async fn shutdown(&mut self) -> anyhow::Result<()>;

    /// Send bytes through the serial port
    async fn send(&mut self, bytes: Bytes) -> anyhow::Result<()>;
}

use rand::{distributions::Alphanumeric, Rng};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::Mutex;
use tracing::info;

use crate::server::config::SerialPortConfig;

#[derive(ThisError, Debug, Clone)]
pub enum FactoryError {
    #[error("No driver found for model: {0}")]
    NoDriver(String),
}

#[derive(Debug)]
pub struct Factory {
    /// This map store Driver generators.
    /// Generator are function that return a SerialPortDriver
    pub map:
        HashMap<String, fn(SerialPortConfig) -> Arc<Mutex<dyn SerialPortDriver + Send + Sync>>>,

    /// The manifest of available power supply devices
    pub manifest: HashMap<String, serde_json::Value>,

    /// The scanner for available devices
    pub scanner: HashMap<String, fn() -> Vec<SerialPortConfig>>,
}

impl Factory {
    /// Create a new empty Factory
    pub fn initialize() -> Self {
        let mut factory = Self {
            map: HashMap::new(),
            manifest: HashMap::new(),
            scanner: HashMap::new(),
        };

        // ----------------------------------------------------------
        factory.register_driver("emulator", |config| {
            Arc::new(Mutex::new(emulator::PowerSupplyEmulator::new(config)))
        });
        factory.manifest.insert(
            "emulator".to_string(),
            emulator::PowerSupplyEmulator::manifest(),
        );

        // ----------------------------------------------------------

        factory.register_driver("standard", |config| {
            Arc::new(Mutex::new(standard::StandardDriver::new(config)))
        });
        factory
            .manifest
            .insert("standard".to_string(), standard::StandardDriver::manifest());
        factory
            .scanner
            .insert("standard".to_string(), standard::StandardDriver::scan);

        // ----------------------------------------------------------
        factory
    }

    /// Register a new Driver generator
    pub fn register_driver<A: Into<String>>(
        &mut self,
        model: A,
        generator: fn(SerialPortConfig) -> Arc<Mutex<dyn SerialPortDriver + Send + Sync>>,
    ) {
        self.map.insert(model.into(), generator);
    }

    pub fn instanciate_driver(
        &self,
        config: SerialPortConfig,
    ) -> Result<Arc<Mutex<dyn SerialPortDriver + Send + Sync>>, FactoryError> {
        if let Some(generator) = self.map.get(&config.model) {
            Ok(generator(config))
        } else {
            Err(FactoryError::NoDriver(config.model))
        }
    }

    /// Scan for available devices
    pub fn scan(&self) -> HashMap<String, SerialPortConfig> {
        let mut result = HashMap::new();

        for (_model, scanner) in &self.scanner {
            let scanned = scanner();
            for config in scanned {
                // Generate a random 10-character string as key
                let random_key: String = rand::thread_rng()
                    .sample_iter(&Alphanumeric)
                    .take(10)
                    .map(char::from)
                    .collect();
                result.insert(random_key, config);
            }
        }

        result
    }
}
