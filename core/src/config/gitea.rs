use serde::Deserialize;

#[derive(Clone, Deserialize, Debug)]
pub struct GiteaConfig {
    pub host: String,
    pub owner: String,
    pub repo: String,
    pub cargolifter_token: Option<String>,
    pub default_branch: Option<String>,
}
