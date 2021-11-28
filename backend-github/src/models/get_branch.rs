use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Commit {
    pub sha: String,
    pub node_id: String,
    // omitted
}

#[derive(Debug, Deserialize)]
pub struct Response {
    pub name: String,
    pub commit: Commit,
}
