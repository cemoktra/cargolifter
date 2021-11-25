use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize)]
pub struct Request {
    pub source_branch: String,
    pub target_branch: String,
    pub title: String,
    pub assignee_id: Option<i32>,
    pub assignee_ids: Option<Vec<i32>>,
    pub reviewer_ids: Option<Vec<i32>>,
    pub description: Option<String>,
    pub target_project_id: Option<i32>,
    pub labels: Option<String>,
    pub milestone_id: Option<i32>,
    pub remove_source_branch: Option<bool>,
    pub allow_collaboration: Option<bool>,
    pub allow_maintainer_to_push: Option<bool>,
    pub squash: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct Response {
    pub id: i32,
    pub iid: i32,
    pub project_id: i32,
    pub title: String,
}
