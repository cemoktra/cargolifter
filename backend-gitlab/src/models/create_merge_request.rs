use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize)]
pub struct Request {
    pub source_branch: String,
    pub target_branch: String,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assignee_id: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assignee_ids: Option<Vec<i32>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reviewer_ids: Option<Vec<i32>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_project_id: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub labels: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub milestone_id: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remove_source_branch: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_collaboration: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_maintainer_to_push: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub squash: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct Response {
    pub id: i32,
    pub iid: i32,
    pub project_id: i32,
    pub title: String,
    // TODO: this is incomplete model, but actually we just need the id
}
