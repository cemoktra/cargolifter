use serde::Serialize;

#[derive(Debug, Default, Serialize)]
pub struct Request {
    pub r#do: String, // merge
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delete_branch_after_merge: Option<bool>,
}
