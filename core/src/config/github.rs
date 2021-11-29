use serde::Deserialize;

/// Github backend configuration
#[derive(Clone, Deserialize, Debug)]
pub struct GithubConfig {
    /// Hostname (defaults to api.github.com)
    pub host: Option<String>,
    /// Repository owner
    pub owner: String,
    /// Repository name
    pub repo: String,
    /// Token used for merge requests (defaults to user pushing the crate)
    pub cargolifter_token: Option<String>,
    /// Defaults to main
    pub default_branch: Option<String>,
}
