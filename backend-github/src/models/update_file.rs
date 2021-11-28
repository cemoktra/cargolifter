use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct UserData {
    pub name: String,
    pub email: String,
    pub date: String,
}

#[derive(Debug, Default, Serialize)]
pub struct Request {
    pub message: String,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    // required for update
    pub sha: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub branch: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub committer: Option<UserData>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<UserData>,
}

#[derive(Debug, Deserialize)]
pub struct Content {
    pub name: String,
    pub path: String,
    pub sha: String,
    pub size: usize,
    pub url: String,
    pub git_url: String,
    pub html_url: String,
    pub download_url: String,
    pub r#type: String,
}

#[derive(Debug, Deserialize)]
pub struct CommitTree {
    pub url: String,
    pub sha: String,
}

#[derive(Debug, Deserialize)]
pub struct CommitParent {
    pub url: String,
    pub html_url: String,
    pub sha: String,
}

#[derive(Debug, Deserialize)]
pub struct Verification {
    pub verified: bool,
    pub reason: String,
    pub signature: Option<String>,
    pub paylouad: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Commit {
    pub sha: String,
    pub node_id: String,
    pub url: String,
    pub html_url: String,
    pub committer: UserData,
    pub author: UserData,
    pub message: String,
    pub tree: CommitTree,
    pub parents: Vec<CommitParent>,
    pub verification: Verification,
}

#[derive(Debug, Deserialize)]
pub struct Response {
    pub content: Content,
    pub commit: Commit,
}
