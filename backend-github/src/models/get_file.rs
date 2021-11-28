use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Response {
    pub r#type: String,
    pub encoding: String,
    pub size: usize,
    pub name: String,
    pub path: String,
    pub content: String,
    pub sha: String,
    pub url: String,
    pub git_url: String,
    pub html_url: String,
    pub download_url: String,
}
