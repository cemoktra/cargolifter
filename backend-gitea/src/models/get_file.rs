use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Response {
    pub name: String,
    pub path: String,
    pub sha: String,
    pub encoding: String,
    pub content: String,
}
