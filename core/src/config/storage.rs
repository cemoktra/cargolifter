use serde::Deserialize;

#[derive(Clone, Deserialize, Debug)]
pub enum StorageType {
    FileSystem(crate::config::FileSystemConfig),
    S3(crate::config::S3Config),
}

#[derive(Clone, Deserialize, Debug)]
pub struct StorageConfig {
    pub r#type: StorageType,
}
