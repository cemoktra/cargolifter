use serde::{Deserialize, Serialize};


#[derive(Debug, Serialize, Deserialize)]
pub struct YankRequest {
    pub name: String,
    pub vers: String,
    pub yank: bool
}