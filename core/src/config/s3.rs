use serde::Deserialize;

/// S3 storage configuration
#[derive(Clone, Deserialize, Debug)]
pub struct S3Config {
    // Bucket name for storage
    pub bucket: String,
    // AWS Credentials
    pub credentials: Option<S3Credentials>,
}

/// S3 credentials
#[derive(Clone, Deserialize, Debug)]
pub struct S3Credentials {
    pub access_key: String,
    pub secret_key: String,
    pub secret_token: Option<String>,
}
