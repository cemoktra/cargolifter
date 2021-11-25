use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize)]
pub struct Request {
    pub branch: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_branch: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub encoding: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author_email: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author_name: Option<String>,
    pub content: String,
    pub commit_message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_commit_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Response {
    pub file_path: String,
    pub branch: String,
}
