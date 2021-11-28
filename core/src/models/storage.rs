#[derive(Debug)]
pub enum StorageError {
    DetailMeLater,
}

impl std::error::Error for StorageError {}

impl std::fmt::Display for StorageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Storage error occured")
    }
}

impl std::convert::From<std::io::Error> for StorageError {
    fn from(_: std::io::Error) -> Self {
        StorageError::DetailMeLater
    }
}

pub struct StorageGetRequest {
    pub crate_name: String,
    pub crate_version: String,
    pub result_sender: tokio::sync::oneshot::Sender<Option<Vec<u8>>>,
}

pub struct StoragePutRequest {
    pub crate_name: String,
    pub crate_version: String,
    pub data: Vec<u8>,
    pub result_sender: tokio::sync::oneshot::Sender<bool>,
}
