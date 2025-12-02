use std::sync::Arc;
use tokio::sync::Mutex;

use rmcp::handler::server::router::prompt::PromptRouter;
use rmcp::handler::server::router::tool::ToolRouter;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::*;
use rmcp::prompt_handler;
use rmcp::prompt_router;
use rmcp::service::RequestContext;
use rmcp::tool;
use rmcp::tool_handler;
use rmcp::tool_router;
use rmcp::ErrorData as McpError;
use rmcp::RoleServer;
use rmcp::ServerHandler;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use tracing::debug;
use tracing::info;

use bytes::{Buf, BytesMut};
use pza_serial_port_client::SerialPortClient;

use crate::server::config::ServerConfig;

#[derive(Serialize, Deserialize, JsonSchema)]
struct SendBytesParams {
    /// Data to send, encoded as hexadecimal string (e.g., "48656c6c6f" for "Hello")
    data: String,
}

#[derive(Serialize, Deserialize, JsonSchema)]
struct SendTextParams {
    /// Text data to send to the serial port
    text: String,
}

#[derive(Serialize, Deserialize, JsonSchema)]
struct ReadBytesParams {
    /// Maximum number of bytes to read (optional, defaults to all available data)
    #[serde(skip_serializing_if = "Option::is_none")]
    max_bytes: Option<usize>,
    /// Whether to clear the buffer after reading (defaults to false)
    #[serde(skip_serializing_if = "Option::is_none")]
    clear_buffer: Option<bool>,
}

#[derive(Serialize, Deserialize, JsonSchema)]
struct ReadTextParams {
    /// Maximum number of characters to read (optional, defaults to all available data)
    #[serde(skip_serializing_if = "Option::is_none")]
    max_chars: Option<usize>,
    /// Whether to clear the buffer after reading (defaults to false)
    #[serde(skip_serializing_if = "Option::is_none")]
    clear_buffer: Option<bool>,
}

#[derive(Serialize, Deserialize, JsonSchema)]
struct WaitForTextParams {
    /// The text to wait for
    expected_text: String,
    /// Timeout in milliseconds (defaults to 5000ms)
    #[serde(skip_serializing_if = "Option::is_none")]
    timeout_ms: Option<u64>,
    /// Whether to clear the buffer after finding the text (defaults to true)
    #[serde(skip_serializing_if = "Option::is_none")]
    clear_buffer: Option<bool>,
}

#[derive(Clone)]
struct PowerSupplyState {
    client: SerialPortClient,
    received_data: Arc<Mutex<BytesMut>>,
}

/// Service structure that handles MCP protocol interactions and manages
/// connections to the Panduza platform.
#[derive(Clone)]
pub struct PowerSupplyService {
    /// Power Supply Name provided by the user
    instance_name: String,

    /// Tool router for MCP tools
    tool_router: ToolRouter<PowerSupplyService>,
    /// Prompt router for MCP prompts
    prompt_router: PromptRouter<PowerSupplyService>,

    state: Arc<Mutex<PowerSupplyState>>,
}

impl PowerSupplyService {
    //--------------------------------------------------------------------------

    pub async fn new(config: ServerConfig, instance_name: String) -> anyhow::Result<Self> {
        let client = SerialPortClient::builder()
            .with_ip(config.broker.tcp.unwrap().clone())
            .with_power_supply_name(instance_name.clone())
            .build()?;
        debug!("Client initialized");

        // Create shared buffer for received data
        let received_data = Arc::new(Mutex::new(BytesMut::new()));

        // Create the state
        let state = Arc::new(Mutex::new(PowerSupplyState {
            client: client.clone(),
            received_data: received_data.clone(),
        }));

        // Spawn a task to listen for incoming data from the rx channel
        let rx_data_buffer = received_data.clone();
        tokio::spawn(async move {
            let mut rx_channel = client.subscribe_rx();

            loop {
                match rx_channel.recv().await {
                    Ok(data) => {
                        // Store received data in the buffer
                        let mut buffer = rx_data_buffer.lock().await;
                        buffer.extend_from_slice(&data);

                        // Optional: limit buffer size to prevent memory issues
                        if buffer.len() > 10_000 {
                            // Keep only the last 8000 bytes
                            let excess = buffer.len() - 8_000;
                            buffer.advance(excess);
                        }

                        debug!(
                            "Received {} bytes, total buffer size: {}",
                            data.len(),
                            buffer.len()
                        );
                    }
                    Err(e) => {
                        tracing::error!("Error receiving data from rx channel: {}", e);
                        // Small delay before retrying to avoid busy loop
                        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                    }
                }
            }
        });

        Ok(Self {
            instance_name,
            tool_router: Self::tool_router(),
            prompt_router: Self::prompt_router(),
            state,
        })
    }
}

#[tool_router]
impl PowerSupplyService {
    //--------------------------------------------------------------------------

    /// Send data to the serial port
    #[tool(
        description = "Send byte data to the serial port. Data should be provided as a hexadecimal string (e.g., '48656c6c6f' for 'Hello')"
    )]
    async fn send_byte_data(
        &self,
        params: Parameters<SendBytesParams>,
    ) -> Result<CallToolResult, McpError> {
        let hex_data = &params.0.data;
        let client = {
            let psu_state = self.state.lock().await;
            psu_state.client.clone()
        };

        // Convert hex string to bytes
        let bytes_data = hex::decode(hex_data).map_err(|e| {
            McpError::new(
                ErrorCode::INVALID_PARAMS,
                format!("Invalid hex data: {}", e),
                None,
            )
        })?;

        // Convert to bytes::Bytes and send via the client
        let bytes_to_send = bytes::Bytes::from(bytes_data);

        // Send the data via the SerialPortClient
        client.send(bytes_to_send.clone()).await.map_err(|e| {
            McpError::new(
                ErrorCode::INTERNAL_ERROR,
                format!("Failed to send data to serial port: {}", e),
                None,
            )
        })?;

        info!(
            "Successfully sent {} bytes to serial port",
            bytes_to_send.len()
        );
        Ok(CallToolResult::success(vec![Content::text(format!(
            "Sent {} bytes to serial port: {}",
            bytes_to_send.len(),
            hex_data
        ))]))
    }

    /// Send text data to the serial port
    #[tool(
        description = "Send text data to the serial port. The text will be converted to bytes using UTF-8 encoding."
    )]
    async fn send_text_data(
        &self,
        params: Parameters<SendTextParams>,
    ) -> Result<CallToolResult, McpError> {
        let text_data = &params.0.text;
        let client = {
            let psu_state = self.state.lock().await;
            psu_state.client.clone()
        };

        // Convert text to bytes using UTF-8 encoding
        let bytes_to_send = bytes::Bytes::from(text_data.as_bytes().to_vec());

        // Send the data via the SerialPortClient
        client.send(bytes_to_send.clone()).await.map_err(|e| {
            McpError::new(
                ErrorCode::INTERNAL_ERROR,
                format!("Failed to send text to serial port: {}", e),
                None,
            )
        })?;

        info!(
            "Successfully sent {} bytes of text to serial port: '{}'",
            bytes_to_send.len(),
            text_data
        );
        Ok(CallToolResult::success(vec![Content::text(format!(
            "Sent {} bytes of text to serial port: '{}'",
            bytes_to_send.len(),
            text_data
        ))]))
    }

    /// Read byte data from the serial port buffer
    #[tool(
        description = "Read byte data that has been received from the serial port. Returns data as hexadecimal string."
    )]
    async fn read_byte_data(
        &self,
        params: Parameters<ReadBytesParams>,
    ) -> Result<CallToolResult, McpError> {
        let max_bytes = params.0.max_bytes.unwrap_or(usize::MAX);
        let clear_buffer = params.0.clear_buffer.unwrap_or(false);

        let psu_state = self.state.lock().await;
        let mut buffer = psu_state.received_data.lock().await;

        // Determine how many bytes to read
        let bytes_to_read = std::cmp::min(max_bytes, buffer.len());

        if bytes_to_read == 0 {
            return Ok(CallToolResult::success(vec![Content::text(
                "No data available in buffer".to_string(),
            )]));
        }

        // Read the data
        let data_bytes = if clear_buffer {
            // Take the specified number of bytes and remove them from buffer
            let data = buffer.split_to(bytes_to_read);
            data.freeze()
        } else {
            // Just read without clearing by copying the data
            let data = buffer
                .iter()
                .take(bytes_to_read)
                .copied()
                .collect::<Vec<u8>>();
            bytes::Bytes::from(data)
        };

        // Convert to hex string
        let hex_data = hex::encode(&data_bytes);

        info!(
            "Read {} bytes from serial port buffer (hex: {})",
            bytes_to_read, hex_data
        );

        Ok(CallToolResult::success(vec![Content::text(format!(
            "Read {} bytes from serial port buffer:\nHex: {}\nText (if UTF-8): {}",
            bytes_to_read,
            hex_data,
            String::from_utf8_lossy(&data_bytes)
        ))]))
    }

    /// Read text data from the serial port buffer
    #[tool(
        description = "Read text data that has been received from the serial port. Returns data as UTF-8 string."
    )]
    async fn read_text_data(
        &self,
        params: Parameters<ReadTextParams>,
    ) -> Result<CallToolResult, McpError> {
        let max_chars = params.0.max_chars.unwrap_or(usize::MAX);
        let clear_buffer = params.0.clear_buffer.unwrap_or(false);

        let psu_state = self.state.lock().await;
        let mut buffer = psu_state.received_data.lock().await;

        if buffer.is_empty() {
            return Ok(CallToolResult::success(vec![Content::text(
                "No data available in buffer".to_string(),
            )]));
        }

        // Convert buffer to string and handle the character limit
        let full_text = String::from_utf8_lossy(&buffer);
        let text_to_read = if max_chars >= full_text.len() {
            full_text.to_string()
        } else {
            full_text.chars().take(max_chars).collect()
        };

        let bytes_consumed = text_to_read.as_bytes().len();

        // Clear buffer if requested
        if clear_buffer {
            if bytes_consumed >= buffer.len() {
                buffer.clear();
            } else {
                buffer.advance(bytes_consumed);
            }
        }

        info!(
            "Read {} characters ({} bytes) from serial port buffer as text",
            text_to_read.len(),
            bytes_consumed
        );

        Ok(CallToolResult::success(vec![Content::text(format!(
            "Read {} characters from serial port buffer:\n{}",
            text_to_read.len(),
            text_to_read
        ))]))
    }

    /// Wait for specific text to arrive on the serial port
    #[tool(
        description = "Wait for a specific text string to be received from the serial port within a timeout period."
    )]
    async fn wait_for_text(
        &self,
        params: Parameters<WaitForTextParams>,
    ) -> Result<CallToolResult, McpError> {
        let expected_text = &params.0.expected_text;
        let timeout_ms = params.0.timeout_ms.unwrap_or(5000);
        let clear_buffer = params.0.clear_buffer.unwrap_or(true);

        let psu_state = self.state.lock().await;
        let buffer_ref = psu_state.received_data.clone();
        drop(psu_state); // Release the lock early

        let start_time = std::time::Instant::now();
        let timeout_duration = std::time::Duration::from_millis(timeout_ms);

        loop {
            // Check if timeout has been reached
            if start_time.elapsed() > timeout_duration {
                return Ok(CallToolResult::success(vec![Content::text(format!(
                    "Timeout: Expected text '{}' not found within {}ms",
                    expected_text, timeout_ms
                ))]));
            }

            // Check current buffer content
            {
                let mut buffer = buffer_ref.lock().await;
                let current_text = String::from_utf8_lossy(&buffer);

                if let Some(pos) = current_text.find(expected_text) {
                    // Found the expected text
                    let result_text = if clear_buffer {
                        // Clear everything up to and including the expected text
                        let bytes_to_clear = pos + expected_text.len();
                        let bytes_to_clear = std::cmp::min(bytes_to_clear, buffer.len());
                        buffer.advance(bytes_to_clear);
                        format!(
                            "Found expected text '{}' at position {} (buffer cleared)",
                            expected_text, pos
                        )
                    } else {
                        format!(
                            "Found expected text '{}' at position {}",
                            expected_text, pos
                        )
                    };

                    info!(
                        "Found expected text '{}' after {:?}",
                        expected_text,
                        start_time.elapsed()
                    );

                    return Ok(CallToolResult::success(vec![Content::text(result_text)]));
                }
            }

            // Small delay before checking again
            tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
        }
    }
}

#[prompt_router]
impl PowerSupplyService {
    // No prompts specified in requirements, but trait requires this implementation
    // Implementation block is needed for the macro to work properly
}

#[tool_handler]
#[prompt_handler]
impl ServerHandler for PowerSupplyService {
    //--------------------------------------------------------------------------

    /// Get server information and capabilities
    fn get_info(&self) -> ServerInfo {
        debug!("MCP get_info called");

        ServerInfo {
            protocol_version: ProtocolVersion::V_2024_11_05,
            capabilities: ServerCapabilities::builder()
                .enable_tools()
                .enable_prompts()
                .build(),
            server_info: Implementation::from_build_env(),
            instructions: Some(format!(
                r#"""This server provides access to a serial port.
The name of this serial port is "{}" and can be used by the user to request actions.
            """#,
                self.instance_name
            )),
        }
    }
}
