use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize)]
pub struct Request {
    pub state: String,
}
#[derive(Debug, Deserialize)]
pub struct Response {
    pub id: String,
    pub state: bool,
}
