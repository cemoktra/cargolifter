use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize)]
pub struct Request {
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub branch: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub new_branch: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Commit {
    pub sha: String,
    pub url: String,
}

#[derive(Debug, Deserialize)]
pub struct Response {
    pub commit: Commit,
}
