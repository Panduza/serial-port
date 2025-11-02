use crate::client::{SerialPortClient, SerialPortClientBuilder};
use crate::{ServerState, SERVER_STATE_STORAGE};
use dioxus::prelude::*;
use std::sync::Arc;
use tracing::debug;
use tracing::error;
use tracing_subscriber::field::debug;

// mod button_power;
// mod config_button;
// mod current_setter;
// mod device_selector;
// mod voltage_setter;

// use button_power::PowerButton;
// use config_button::ConfigButton;
// use current_setter::CurrentSetter;
// use device_selector::DeviceSelector;
// use voltage_setter::VoltageSetter;

const FAVICON: Asset = asset!("/assets/icons/icon.ico");
const SERIAL_DISPLAY_CSS: Asset = asset!("/assets/serial-display.css");

/// Escape HTML special characters to prevent rendering issues
fn escape_html(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

#[component]
pub fn Gui() -> Element {
    // Inject server state into context
    use_context_provider(|| {
        SERVER_STATE_STORAGE
            .get()
            .expect("Failed to get server state")
            .clone()
    });

    // Signals
    let s_serial_data = use_signal(|| String::new());
    let s_client: Signal<Option<SerialPortClient>> = use_signal(|| None);

    // Coroutine to load configuration from server state and create client
    let _init_coro: Coroutine<()> = use_coroutine({
        let mut s_client = s_client.clone();
        move |_rx| async move {
            // Get server state from context
            let server_state: Arc<ServerState> = use_context();

            let addr = server_state.server_config.lock().await.broker.tcp.clone();

            let names: Vec<String> = server_state
                .instances
                .lock()
                .await
                .keys()
                .cloned()
                .collect();

            match SerialPortClient::builder()
                .with_ip(addr.clone().expect("address not set").clone())
                .with_power_supply_name(names.get(0).cloned().expect("at least a name"))
                .enable_tx_monitoring(true)
                .build()
            {
                Ok(client) => {
                    s_client.set(Some(client));
                }
                Err(e) => {
                    error!("Failed to create SerialPortClient: {}", e);
                }
            }
        }
    });

    // Coroutine to listen to the rx channel and update received data
    let _rx_coro: Coroutine<()> = use_coroutine({
        let mut s_serial_data = s_serial_data.clone();
        let s_client = s_client.clone();
        move |_rx| async move {
            loop {
                if let Some(client) = s_client.read().as_ref() {
                    let mut rx_channel = client.subscribe_rx();

                    while let Ok(data) = rx_channel.recv().await {
                        // Convert bytes to string and append to received data
                        if let Ok(text) = String::from_utf8(data.to_vec()) {
                            s_serial_data.with_mut(|current_data| {
                                // Process text to handle line endings properly
                                let processed_text = text
                                    .replace("\r\n", "\n") // Windows line ending to Unix
                                    .replace("\r", "\n"); // Mac line ending to Unix

                                // Escape HTML special characters
                                let escaped_text = escape_html(processed_text.trim());

                                // Add RX prefix with CSS class for green color
                                let formatted_text = format!(
                                    "<span class=\"rx-data\">RX: {}</span>\n",
                                    escaped_text
                                );
                                current_data.push_str(&formatted_text);

                                // Optionally limit the size to prevent memory issues
                                if current_data.len() > 50000 {
                                    let start = current_data.len() - 40000;
                                    *current_data = current_data[start..].to_string();
                                }
                            });
                        }
                    }
                } else {
                    // Wait a bit before checking again if client is available
                    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                }
            }
        }
    });

    // Coroutine to listen to the tx channel and update sent data
    let _tx_coro: Coroutine<()> = use_coroutine({
        let mut s_serial_data = s_serial_data.clone();
        let s_client = s_client.clone();
        move |_tx| async move {
            loop {
                if let Some(client) = s_client.read().as_ref() {
                    let mut tx_channel = client.subscribe_tx();

                    while let Ok(data) = tx_channel.recv().await {
                        debug!("NEW TX data ");

                        // Convert bytes to string and append to sent data
                        if let Ok(text) = String::from_utf8(data.to_vec()) {
                            s_serial_data.with_mut(|current_data| {
                                // Process text to handle line endings properly
                                let processed_text = text
                                    .replace("\r\n", "\n") // Windows line ending to Unix
                                    .replace("\r", "\n"); // Mac line ending to Unix

                                // Escape HTML special characters
                                let escaped_text = escape_html(processed_text.trim());

                                // Add TX prefix with CSS class for red color
                                let formatted_text = format!(
                                    "<span class=\"tx-data\">TX: {}</span>\n",
                                    escaped_text
                                );
                                current_data.push_str(&formatted_text);

                                debug!(
                                    "Appended TX data to serial data display: {}",
                                    formatted_text
                                );

                                // Optionally limit the size to prevent memory issues
                                if current_data.len() > 50000 {
                                    let start = current_data.len() - 40000;
                                    *current_data = current_data[start..].to_string();
                                }
                            });
                        }
                    }
                } else {
                    // Wait a bit before checking again if client is available
                    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                }
            }
        }
    });

    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: SERIAL_DISPLAY_CSS }
        document::Script {
            r#"
                // Auto-scroll serial terminal to bottom when content changes
                (function() {{
                    let observerInitialized = false;
                    
                    function initializeScrollObserver() {{
                        if (observerInitialized) return;
                        
                        const terminal = document.getElementById('serial-terminal');
                        if (terminal) {{
                            const observer = new MutationObserver(() => {{
                                terminal.scrollTop = terminal.scrollHeight;
                            }});
                            observer.observe(terminal, {{ 
                                childList: true, 
                                subtree: true,
                                characterData: true 
                            }});
                            observerInitialized = true;
                        }}
                    }}
                    
                    // Try to initialize on DOMContentLoaded
                    if (document.readyState === 'loading') {{
                        document.addEventListener('DOMContentLoaded', initializeScrollObserver);
                    }} else {{
                        initializeScrollObserver();
                    }}
                    
                    // Fallback: check periodically if terminal exists (for dynamic loading)
                    const maxAttempts = 50; // 5 seconds max
                    let attempts = 0;
                    const checkTerminal = setInterval(() => {{
                        attempts++;
                        if (observerInitialized || attempts >= maxAttempts) {{
                            clearInterval(checkTerminal);
                        }} else {{
                            initializeScrollObserver();
                        }}
                    }}, 100);
                }})();
            "#
        }

        div {
            class: "main-container",

            header {
                class: "header",

                h1 {
                    class: "header-title",
                    "Panduza Serial Port"
                }
            }

            main {
                class: "main-content",

                div {
                    class: "card",

                    h2 {
                        class: "card-title",
                        "Données série (TX/RX)"
                    }

                    div {
                        id: "serial-terminal",
                        class: "serial-terminal",
                        dangerous_inner_html: "{s_serial_data.read()}"
                    }
                }

                div {
                    class: "card",

                    h2 {
                        class: "card-title",
                        "État de la connexion"
                    }

                    div {
                        class: "status-container",
                        if s_client.read().is_some() {
                            span {
                                class: "status-connected",
                                "✓ Client connecté"
                            }
                        } else {
                            span {
                                class: "status-disconnected",
                                "✗ Client non connecté"
                            }
                        }
                    }
                }
            }
        }
    }
}
