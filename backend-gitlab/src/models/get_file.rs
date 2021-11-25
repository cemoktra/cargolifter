use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Response {
    pub file_name: String,
    pub file_path: String,
    pub size: usize,
    pub encoding: String,
    pub content: String,
    pub content_sha256: String,
    pub r#ref: String,
    pub blob_id: String,
    pub commit_id: String,
    pub last_commit_id: String,
}
