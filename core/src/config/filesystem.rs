use serde::Deserialize;

#[derive(Clone, Deserialize, Debug)]
pub struct FileSystemConfig {
    pub path: String,
}
