use serde::Deserialize;

/// Web service configuration
#[derive(Deserialize, Debug)]
pub struct WebServiceConfig {
    /// Port
    pub port: i32,
}
