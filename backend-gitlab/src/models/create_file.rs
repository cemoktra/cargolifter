use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize)]
pub struct Request {
    pub branch: String,
    pub start_branch: Option<String>,
    pub encoding: Option<String>,
    pub author_email: Option<String>,
    pub author_name: Option<String>,
    pub content: String,
    pub commit_message: String,
}

#[derive(Debug, Deserialize)]
pub struct Response {
    pub file_path: String,
    pub branch: String,
}
