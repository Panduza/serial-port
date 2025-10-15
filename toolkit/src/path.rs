use std::fs;
use std::io;
use std::path::PathBuf;

/// Get the user root directory for Panduza
///
/// Returns the path to the `.panduza` directory inside the user's home directory.
///
/// # Returns
/// `Some(PathBuf)` containing the path to `~/.panduza`,
/// or `None` if the home directory cannot be determined.
pub fn user_root_dir() -> Option<PathBuf> {
    dirs::home_dir().map(|home| home.join(".panduza"))
}

/// Ensure that the user root directory exists
///
/// Creates the `.panduza` directory in the user's home directory if it doesn't exist.
///
/// # Returns
/// `Ok(())` if the directory exists or was created successfully,
/// or an `io::Error` if creation failed.
pub fn ensure_user_root_dir_exists() -> io::Result<()> {
    user_root_dir()
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::NotFound,
                "Unable to determine home directory",
            )
        })
        .and_then(fs::create_dir_all)
}
