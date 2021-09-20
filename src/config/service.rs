use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct ServiceConfig {
    pub port: i32,
}
