use serde::Deserialize;

#[derive(Clone, Deserialize, Debug)]
pub enum BackendType {
    Github(crate::config::GithubConfig),
    Gitlab(crate::config::GitlabConfig),
}