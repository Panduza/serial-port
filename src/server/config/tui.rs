use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TuiConfig {
    /// Enable or disable the TUI
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enable: Option<bool>,
}
