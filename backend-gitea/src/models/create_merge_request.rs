use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize)]
pub struct Request {
    pub title: String,
    pub base: String,
    pub head: String,    
}

#[derive(Debug, Deserialize)]
pub struct Response {
    pub id: u64,
    pub number: u64,
}
