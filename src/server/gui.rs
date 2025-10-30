use dioxus::prelude::*;
use std::process::Command;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::client::{SerialPortClient, SerialPortClientBuilder};

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
const TAILWIND_CSS: Asset = asset!("/assets/tailwind.css");
const MAIN_CSS: Asset = asset!("/assets/main.css");

#[component]
pub fn Gui() -> Element {
    let mut runtime_status = use_signal(|| "Initializing...".to_string());

    // Use effect to monitor runtime status
    use_effect(move || {
        spawn(async move {
            tokio::time::sleep(std::time::Duration::from_secs(2)).await;
            runtime_status.set("Background services running".to_string());
        });
    });

    rsx! {
            document::Link { rel: "icon", href: FAVICON }
            document::Link { rel: "stylesheet", href: TAILWIND_CSS }
            document::Link { rel: "stylesheet", href: MAIN_CSS }

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
