use serde::Deserialize;

/// Filesystem storage configuration
#[derive(Clone, Deserialize, Debug)]
pub struct FileSystemConfig {
    // Root folder for storage
    pub path: String,
}
