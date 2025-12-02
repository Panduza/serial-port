mod bytes;
mod error;
mod status;

pub use error::ErrorPayload;
pub use status::Status;
pub use status::StatusPayload;

/// Type alias for PZA ID
pub type PzaId = String;

/// Generate a random 5-character PZA ID
pub fn generate_pza_id() -> String {
    pza_toolkit::rand::generate_random_string(5)
}
