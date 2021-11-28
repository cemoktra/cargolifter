use serde::Deserialize;

#[derive(Clone, Deserialize, Debug)]
pub enum BackendType {
    Github(crate::config::GithubConfig),
    Gitlab(crate::config::GitlabConfig),
}

#[derive(Clone, Deserialize, Debug)]
pub struct BackendConfig {
    pub r#type: BackendType,
}
