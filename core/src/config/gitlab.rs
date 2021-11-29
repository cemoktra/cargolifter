use serde::Deserialize;

/// Gitlab storage configuration
#[derive(Clone, Deserialize, Debug)]
pub struct GitlabConfig {
    /// Hostname (defaults to gitlab.com)
    pub host: Option<String>,
    /// Project ID
    pub project_id: usize,
    /// Token used for merge requests (defaults to user pushing the crate)
    pub cargolifter_token: Option<String>,
    /// Defaults to main
    pub default_branch: Option<String>,
}
