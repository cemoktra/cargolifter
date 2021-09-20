use crate::config::git::GitConfig;
use crate::config::storage::StorageConfig;
use crate::config::service::ServiceConfig;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct CargoLifterConfig {
    pub mirror: Option<GitConfig>,
    pub registry: Option<GitConfig>,
    pub service: ServiceConfig,
    pub storage: StorageConfig,
}
