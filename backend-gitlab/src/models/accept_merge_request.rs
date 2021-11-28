use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize)]
pub struct Request {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub merge_commit_message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub squash_commit_message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub squash: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub should_remove_source_branch: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub merge_when_pipeline_succeeds: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sha: Option<String>,
}

// TODO: seems to be same response as for create merge request
#[derive(Debug, Deserialize)]
pub struct Response {
    pub id: i32,
    pub iid: i32,
    pub project_id: i32,
    pub title: String,
    // TODO: this is incomplete model, but actually we just need the id
}
