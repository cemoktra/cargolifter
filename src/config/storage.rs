use serde::Deserialize;

#[derive(Clone, Deserialize, Debug)]
pub enum StorageType {
    FileSystem(FileSystemConfig),
    S3(S3Config),
}

#[derive(Clone, Deserialize, Debug)]
pub struct FileSystemConfig {
    pub path: String,
}

#[derive(Clone, Deserialize, Debug)]
pub struct S3Config {
    pub bucket: String,
    pub credentials: Option<S3Credentials>,
}

#[derive(Clone, Deserialize, Debug)]
pub struct S3Credentials {
    pub access_key: String,
    pub secret_key: String,
    pub secret_token: Option<String>,
}

#[derive(Clone, Deserialize, Debug)]
pub struct StorageConfig {
    pub r#type: StorageType,
}
