use serde::Deserialize;

#[derive(Clone, Deserialize, Debug)]
pub enum BackendType {
    Gitea(crate::config::GiteaConfig),
    Github(crate::config::GithubConfig),
    Gitlab(crate::config::GitlabConfig),
}
