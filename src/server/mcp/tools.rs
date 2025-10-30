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

use crate::client::SerialPortClient;
use crate::client::SerialPortClientBuilder;

use crate::config::ServerMainConfig;

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

#[derive(Clone)]
struct PowerSupplyState {
    client: SerialPortClient,
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

    pub fn new(config: ServerMainConfig, instance_name: String) -> anyhow::Result<Self> {
        let client = SerialPortClientBuilder::default()
            .with_ip(config.broker.tcp.unwrap().clone())
            .with_power_supply_name(instance_name.clone())
            .build()?;
        debug!("Client initialized");

        Ok(Self {
            instance_name,
            tool_router: Self::tool_router(),
            prompt_router: Self::prompt_router(),
            state: Arc::new(Mutex::new(PowerSupplyState { client })),
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
