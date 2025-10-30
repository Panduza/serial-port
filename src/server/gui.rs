use crate::client::{SerialPortClient, SerialPortClientBuilder};
use crate::{ServerState, SERVER_STATE_STORAGE};
use dioxus::prelude::*;
use std::process::Command;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::error;

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

const FAVICON: Asset = asset!("/assets/favicon.ico");
// const TAILWIND_CSS: Asset = asset!("/assets/tailwind.css");

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
    // let mut s_client: Signal<Option<Arc<Mutex<SerialPortClient>>>> = use_signal(|| None);

    // Coroutine to load configuration from server state
    let _coro: Coroutine<()> = use_coroutine({
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

            let client = SerialPortClient::builder()
                .with_ip(addr.clone().expect("address not set").clone())
                .with_power_supply_name(names.get(0).cloned().expect("at least a name"))
                .build()
                .unwrap();

            // match client {
            //     Ok(c) => {
            //         let arc_client = Arc::new(Mutex::new(c));
            //         s_client.set(Some(arc_client));
            //     }
            //     Err(e) => {
            //         error!("Failed to create SerialPortClient: {}", e);
            //     }
            // }
        }
    });

    rsx! {
            document::Link { rel: "icon", href: FAVICON }



            div {
                class: "main-container",

                header {
                    class: "flex justify-between items-center p-4 bg-slate-800 border-b border-slate-700",

                    h1 {
                        class: "text-2xl font-bold text-white",
                        "Panduza Serial Port"
                    }
                // main {
                //     PowerSupplyControl {}
                // }
            }
        }
    }
}
