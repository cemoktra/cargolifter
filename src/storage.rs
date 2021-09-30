use crate::config::storage::StorageConfig;
use aws_sdk_s3::{
    error::{GetObjectError, PutObjectError},
    SdkError,
};
use std::io::Read;
use tokio::{sync::*, task::JoinHandle};

use self::{filesystem::FileSystemStorage, s3::S3Storage};
use axum::async_trait;

pub mod filesystem;
pub mod s3;

pub enum StorageError {
    DetailMeLater,
}

impl std::convert::From<std::io::Error> for StorageError {
    fn from(_: std::io::Error) -> Self {
        StorageError::DetailMeLater
    }
}

impl std::convert::From<SdkError<GetObjectError>> for StorageError {
    fn from(_: SdkError<GetObjectError>) -> Self {
        StorageError::DetailMeLater
    }
}

impl std::convert::From<SdkError<PutObjectError>> for StorageError {
    fn from(_: SdkError<PutObjectError>) -> Self {
        StorageError::DetailMeLater
    }
}

#[async_trait]
pub trait Storage {
    async fn get(
        &self,
        crate_name: &str,
        crate_version: &str,
        mirror: bool,
    ) -> Result<Vec<u8>, StorageError>;
    async fn put(
        &mut self,
        crate_name: &str,
        crate_version: &str,
        mirror: bool,
        data: &Vec<u8>,
    ) -> Result<(), StorageError>;
}

pub enum StorageCommand {
    Get(GetRequest),
    Put(PutRequest),
    MirrorGet(MirrorGetRequest),
}

pub struct GetRequest {
    pub crate_name: String,
    pub crate_version: String,
    pub result_sender: oneshot::Sender<Option<Vec<u8>>>,
}

pub struct PutRequest {
    pub crate_name: String,
    pub crate_version: String,
    pub data: Vec<u8>,
    pub result_sender: oneshot::Sender<bool>,
}

pub struct MirrorGetRequest {
    pub crate_name: String,
    pub crate_version: String,
    pub result_sender: oneshot::Sender<Option<Vec<u8>>>,
}

pub struct StorageService;

impl StorageService {
    pub fn run(config: StorageConfig) -> (JoinHandle<()>, mpsc::Sender<StorageCommand>) {
        let (sender, mut receiver) = tokio::sync::mpsc::channel::<StorageCommand>(16);
        let handle = tokio::spawn(async move {
            let mut storage: Box<dyn Storage + Send + Sync> = match config.r#type {
                crate::config::storage::StorageType::FileSystem(config) => {
                    Box::new(FileSystemStorage::new(&config.path))
                }
                crate::config::storage::StorageType::S3(config) => {
                    Box::new(S3Storage::new(config).await)
                }
            };

            loop {
                match receiver.recv().await {
                    Some(command) => {
                        match command {
                            StorageCommand::MirrorGet(request) => {
                                match storage
                                    .get(&request.crate_name, &request.crate_version, true)
                                    .await
                                {
                                    Ok(data) => match request.result_sender.send(Some(data)) {
                                        Ok(_) => {}
                                        Err(_) => {
                                            log::error!("Failed to send storage command result!");
                                        }
                                    },
                                    Err(_) => {
                                        log::warn!(
                                            "failed to read '{}' version '{}' from storage! Trying crates.io ...",
                                            request.crate_name,
                                            request.crate_version
                                        );

                                        match ureq::get(&format!(
                                            "https://crates.io/api/v1/crates/{}/{}/download",
                                            request.crate_name, request.crate_version
                                        ))
                                        .call()
                                        {
                                            Ok(response) => {
                                                if response.has("Content-Length") {
                                                    let content_length = response
                                                        .header("Content-Length")
                                                        .and_then(|s| s.parse::<usize>().ok())
                                                        .unwrap_or_default();
                                                    let mut response_bytes: Vec<u8> =
                                                        Vec::with_capacity(content_length);
                                                    response
                                                        .into_reader()
                                                        .read_to_end(&mut response_bytes)
                                                        .unwrap();

                                                    match storage
                                                        .put(
                                                            &request.crate_name,
                                                            &request.crate_version,
                                                            true,
                                                            &response_bytes,
                                                        )
                                                        .await
                                                    {
                                                        Ok(_) => {
                                                            match request
                                                                .result_sender
                                                                .send(Some(response_bytes))
                                                            {
                                                                Ok(_) => {}
                                                                Err(_) => {
                                                                    log::error!("Failed to send storage command result!");
                                                                }
                                                            }
                                                        }
                                                        Err(_) => {
                                                            match request.result_sender.send(None) {
                                                                Ok(_) => {}
                                                                Err(_) => {
                                                                    log::error!("Failed to send storage command result!");
                                                                }
                                                            }
                                                        }
                                                    }
                                                } else {
                                                    log::error!(
                                                        "crates.io response has no content-length!"
                                                    );

                                                    // TODO: move this redundant code to function
                                                    match request.result_sender.send(None) {
                                                        Ok(_) => {}
                                                        Err(_) => {
                                                            log::error!("Failed to send storage command result!");
                                                        }
                                                    }
                                                }
                                            }
                                            Err(e) => {
                                                log::error!(
                                                    "Failed to get crate from crates.io: {}",
                                                    e
                                                );

                                                match request.result_sender.send(None) {
                                                    Ok(_) => {}
                                                    Err(_) => {
                                                        log::error!("Failed to send storage command result!");
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                            StorageCommand::Put(request) => {
                                match storage
                                    .put(
                                        &request.crate_name,
                                        &request.crate_version,
                                        false,
                                        &request.data,
                                    )
                                    .await
                                {
                                    Ok(_) => match request.result_sender.send(true) {
                                        Ok(_) => {}
                                        Err(_) => {
                                            log::error!("Failed to send storage command result!");
                                        }
                                    },
                                    Err(_) => {
                                        log::error!(
                                            "failed to add '{}' version '{}' to storage!",
                                            request.crate_name,
                                            request.crate_version
                                        );
                                        match request.result_sender.send(false) {
                                            Ok(_) => {}
                                            Err(_) => {
                                                log::error!(
                                                    "Failed to send storage command result!"
                                                );
                                            }
                                        }
                                    }
                                }
                            }
                            StorageCommand::Get(request) => {
                                match storage
                                    .get(&request.crate_name, &request.crate_version, false)
                                    .await
                                {
                                    Ok(data) => match request.result_sender.send(Some(data)) {
                                        Ok(_) => {}
                                        Err(_) => {
                                            log::error!("Failed to send storage command result!");
                                        }
                                    },
                                    Err(_) => {
                                        log::error!(
                                            "failed to read '{}' version '{}' from storage!",
                                            request.crate_name,
                                            request.crate_version
                                        );
                                        match request.result_sender.send(None) {
                                            Ok(_) => {}
                                            Err(_) => {
                                                log::error!(
                                                    "Failed to send storage command result!"
                                                );
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    None => {
                        log::warn!("Did not receive a StorageCommand!")
                    }
                }
            }
        });
        (handle, sender)
    }
}
