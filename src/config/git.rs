use serde::Deserialize;

#[derive(Clone, Deserialize, Debug)]
pub struct GitConfig {
    pub remote_url: String,
    pub clone_path: String,
    pub username: Option<String>,
    pub email: Option<String>,
    pub branch: Option<String>
}
