use serde::Deserialize;


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
