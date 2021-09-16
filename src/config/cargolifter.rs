use crate::config::git::GitConfig;
use crate::config::storage::StorageConfig;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct CargoLifterConfig {
    pub mirror: Option<GitConfig>,
    pub registry: Option<GitConfig>,
    pub service_port: i32,
    pub storage_config: StorageConfig,
}
