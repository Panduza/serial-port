use bytes::Bytes;
use serde::{Deserialize, Serialize};
use serde_with::{base64::Base64, serde_as};

/// Error payload for communicating error messages
#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BytesPayload {
    /// PZA identifier
    pub pza_id: String,
    /// Data
    #[serde_as(as = "Base64")]
    pub data: Bytes,
}

impl BytesPayload {
    pub fn from_data(data: Bytes) -> Self {
        Self {
            pza_id: super::generate_pza_id(),
            data,
        }
    }

    /// Serialize the BytesPayload to JSON bytes
    pub fn to_json_bytes(&self) -> anyhow::Result<Bytes> {
        Ok(Bytes::from(serde_json::to_string(self)?))
    }

    /// Deserialize a StatusPayload from JSON bytes
    pub fn from_json_bytes(bytes: Bytes) -> anyhow::Result<Self> {
        Ok(serde_json::from_slice(&bytes)?)
    }
}
