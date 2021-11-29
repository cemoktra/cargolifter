use serde::Deserialize;

/// Backend configuration
#[derive(Clone, Deserialize, Debug)]
pub enum BackendType {
    /// Github backend
    Github(crate::config::GithubConfig),
    /// Gitlab backend
    Gitlab(crate::config::GitlabConfig),
}