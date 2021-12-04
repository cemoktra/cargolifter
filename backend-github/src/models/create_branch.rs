use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
pub struct Request {
    pub r#ref: String,
    pub sha: String,
}

#[derive(Debug, Deserialize)]
pub struct Object {
    pub sha: String,
    pub r#type: String,
    pub url: String,
}

#[derive(Debug, Deserialize)]
pub struct Response {
    pub r#ref: String,
    pub node_id: String,
    pub url: String,
    pub object: Object,
}
