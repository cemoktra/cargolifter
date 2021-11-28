use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct WebServiceConfig {
    pub port: i32,
}
