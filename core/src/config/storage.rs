use serde::Deserialize;

/// Storage configuration
#[derive(Clone, Deserialize, Debug)]
pub enum StorageType {
    /// Store crates on local filesystem
    FileSystem(crate::config::FileSystemConfig),
    /// Store crates on S3
    S3(crate::config::S3Config),
}
