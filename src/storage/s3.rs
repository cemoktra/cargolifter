use std::path::PathBuf;

use crate::config::storage::S3Config;
use crate::tools::crate_name_to_path;
use aws_sdk_s3::ByteStream;
use axum::async_trait;
use bytes::Buf;

use super::Storage;
use super::StorageError;

pub struct S3Storage {
    client: aws_sdk_s3::Client,
    config: S3Config,
}

impl S3Storage {
    pub async fn new(s3config: S3Config) -> Self {
        let loader = aws_config::from_env();
        if let Some(credentials) = &s3config.credentials {
            let credentials = aws_types::Credentials::new(
                credentials.access_key.clone(),
                credentials.secret_key.clone(),
                credentials.secret_token.clone(),
                None,
                "s3storage",
            );
            let config = loader.credentials_provider(credentials).load().await;
            Self {
                client: aws_sdk_s3::Client::new(&config),
                config: s3config,
            }
        } else {
            let config = loader.load().await;
            Self {
                client: aws_sdk_s3::Client::new(&config),
                config: s3config,
            }
        }
    }
}

#[async_trait]
impl Storage for S3Storage {
    async fn get(
        &self,
        crate_name: &str,
        crate_version: &str,
        mirror: bool,
    ) -> Result<Vec<u8>, StorageError> {
        let path = if mirror {
            PathBuf::new().join("mirror")
        } else {
            PathBuf::new()
        };
        let path = path.join(crate_name_to_path(crate_name));
        let path = path.join(format!("{}", crate_version));
        log::info!("trying to get '{}'", path.to_str().unwrap());

        let resp = self
            .client
            .get_object()
            .set_bucket(Some(self.config.bucket.clone()))
            .set_key(Some(path.to_str().unwrap().into()))
            .send()
            .await?;

        let stream = resp.body;
        match stream.collect().await {
            Ok(bytes) => Ok(bytes.chunk().to_vec()),
            Err(_) => Err(StorageError::DetailMeLater),
        }
    }

    async fn put(
        &mut self,
        crate_name: &str,
        crate_version: &str,
        mirror: bool,
        data: &Vec<u8>,
    ) -> Result<(), StorageError> {
        let path = if mirror {
            PathBuf::new().join("mirror")
        } else {
            PathBuf::new()
        };
        let path = path.join(crate_name_to_path(crate_name));
        let path = path.join(format!("{}", crate_version));
        log::info!("adding '{}' to storage", path.to_str().unwrap());

        let byte_stream = ByteStream::from(data.clone());
        let _resp = self
            .client
            .put_object()
            .set_bucket(Some(self.config.bucket.clone()))
            .set_key(Some(path.to_str().unwrap().into()))
            .set_body(Some(byte_stream))
            .send()
            .await?;

        Ok(())
    }
}
