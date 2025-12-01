use crate::server::mcp::McpServer;
use crate::ServerState;
use anyhow::Context;
use pza_toolkit::rumqtt::broker::start_broker_in_thread;
use std::sync::Arc;
use tracing::{info, warn};

/// Start background services for the server
///
pub async fn server_services(server_state: Arc<ServerState>) -> anyhow::Result<()> {
    // Start built-in MQTT broker if configured
    {
        let broker_config = server_state
            .server_config
            .as_ref()
            .lock()
            .await
            .broker
            .clone();
        if broker_config.use_builtin == Some(true) {
            start_broker_in_thread(broker_config.clone())
                .with_context(|| "Fail to start broker")?;
            info!("Started built-in MQTT broker");
        }
    }

    {
        server_state
            .start_runtime()
            .await
            .with_context(|| "Fail to start service runtime")?;
    }

    {
        let mcp_config = server_state.server_config.as_ref().lock().await.mcp.clone();

        if mcp_config.enable {
            let instance_names = server_state.instances_names().await;
            let ccc = server_state.server_config.as_ref().lock().await.clone();
            McpServer::run(ccc, instance_names)
                .await
                .with_context(|| "Fail to start MCP server")?;
        } else {
            warn!("MCP server is disabled in configuration, not starting it.");
        }
    }

    loop {
        // Placeholder for service tasks
        tokio::time::sleep(std::time::Duration::from_secs(10)).await;
    }
}
