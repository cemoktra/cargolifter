use serde::{Deserialize, Serialize};

/// Update yank status
#[derive(Debug, Serialize, Deserialize)]
pub struct YankRequest {
    pub name: String,
    pub vers: String,
    pub yank: bool,
}
