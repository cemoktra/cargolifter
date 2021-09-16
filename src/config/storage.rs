use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub enum StorageType {
    FileSystem,
}

#[derive(Deserialize, Debug)]
pub struct StorageConfig {
    pub r#type: StorageType,
    pub path: String,
}
