use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct CargoLifterConfig {
    pub backend: crate::config::BackendType,
    pub web: crate::config::WebServiceConfig,
    pub storage: crate::config::StorageType,
}
