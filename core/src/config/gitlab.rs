use serde::Deserialize;

#[derive(Clone, Deserialize, Debug)]
pub struct GitlabConfig {
    pub host: Option<String>,
    pub project_id: usize,
    pub cargolifter_token: Option<String>,
}
