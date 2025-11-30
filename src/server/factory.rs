use rand::{distributions::Alphanumeric, Rng};
use std::{collections::HashMap, sync::Arc};
use thiserror::Error as ThisError;
use tokio::sync::Mutex;
use tracing::info;

use crate::{config::SerialPortConfig, drivers::SerialPortDriver};

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
            Arc::new(Mutex::new(
                crate::drivers::emulator::PowerSupplyEmulator::new(config),
            ))
        });
        factory.manifest.insert(
            "emulator".to_string(),
            crate::drivers::emulator::PowerSupplyEmulator::manifest(),
        );

        // ----------------------------------------------------------

        factory.register_driver("standard", |config| {
            Arc::new(Mutex::new(crate::drivers::standard::StandardDriver::new(
                config,
            )))
        });
        factory.manifest.insert(
            "standard".to_string(),
            crate::drivers::standard::StandardDriver::manifest(),
        );
        factory.scanner.insert(
            "standard".to_string(),
            crate::drivers::standard::StandardDriver::scan,
        );

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

    // pub fn write_scan_results_to_file(&self) -> Result<(), Box<dyn std::error::Error>> {
    //     // Get the factory scan results file path
    //     let scan_file_path =
    //         crate::path::scan_file().ok_or("Unable to determine factory scan results file path")?;

    //     info!(
    //         "Writing factory scan results to: {}",
    //         scan_file_path.display()
    //     );

    //     // Scan for devices
    //     let scan_results = self.scan();

    //     pza_toolkit::config::write_config(&scan_file_path, &scan_results)?;
    //     Ok(())
    // }

    // /// Write the manifest data to the factory manifest file
    // pub fn write_manifest_to_file(&self) -> Result<(), Box<dyn std::error::Error>> {
    //     // Ensure the user root directory exists
    //     pza_toolkit::path::ensure_user_root_dir_exists()?;

    //     // Get the factory manifest file path
    //     let manifest_file_path = crate::path::factory_manifest_file()
    //         .ok_or("Unable to determine factory manifest file path")?;

    //     info!(
    //         "Writing factory manifest to: {}",
    //         manifest_file_path.display()
    //     );

    //     // Serialize the manifest data to pretty JSON
    //     let json_content = serde_json::to_string_pretty(&self.manifest)?;

    //     // Write to file
    //     std::fs::write(manifest_file_path, json_content)?;

    //     Ok(())
    // }
}
