use std::path::PathBuf;

use async_trait::async_trait;
use aws_sdk_s3::ByteStream;
use bytes::Buf;
use cargolifter_core::config::S3Config;

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
impl cargolifter_core::Storage for S3Storage {
    async fn get(
        &self,
        crate_name: &str,
        crate_version: &str,
    ) -> Result<Vec<u8>, cargolifter_core::models::StorageError> {
        let path = PathBuf::new();
        let path = path.join(cargolifter_core::utils::get_crate_path(crate_name));
        let path = path.join(crate_version);
        tracing::info!("trying to get '{}'", path.to_str().unwrap());

        match self
            .client
            .get_object()
            .set_bucket(Some(self.config.bucket.clone()))
            .set_key(Some(path.to_str().unwrap().into()))
            .send()
            .await
        {
            Ok(resp) => {
                let stream = resp.body;
                match stream.collect().await {
                    Ok(bytes) => Ok(bytes.chunk().to_vec()),
                    Err(_) => Err(cargolifter_core::models::StorageError::DetailMeLater),
                }
            }
            Err(_) => Err(cargolifter_core::models::StorageError::DetailMeLater),
        }
    }

    async fn put(
        &mut self,
        crate_name: &str,
        crate_version: &str,
        data: &[u8],
    ) -> Result<(), cargolifter_core::models::StorageError> {
        let path = PathBuf::new();
        let path = path.join(cargolifter_core::utils::get_crate_path(crate_name));
        let path = path.join(crate_version);
        tracing::info!("adding '{}' to storage", path.to_str().unwrap());

        let byte_stream = ByteStream::from(data.to_vec());
        match self
            .client
            .put_object()
            .set_bucket(Some(self.config.bucket.clone()))
            .set_key(Some(path.to_str().unwrap().into()))
            .set_body(Some(byte_stream))
            .send()
            .await
        {
            Ok(_) => Ok(()),
            Err(_) => Err(cargolifter_core::models::StorageError::DetailMeLater),
        }
    }
}
