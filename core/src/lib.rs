
/// Configuration structs
pub mod config;
/// Cargo JSON models
pub mod models;
/// Common functions
pub mod utils;

use async_trait::async_trait;

/// Commands to send to the git backend
pub enum BackendCommand {
    Publish(
        String,
        models::PublishRequest,
        tokio::sync::oneshot::Sender<bool>,
    ),
    Yank(
        String,
        models::YankRequest,
        tokio::sync::oneshot::Sender<bool>,
    ),
    IsVersionPublished(
        String,
        String,
        String,
        tokio::sync::oneshot::Sender<bool>,
    ),
}

/// Commands to send to the storage
pub enum StorageCommand {
    Get(models::StorageGetRequest),
    Put(models::StoragePutRequest),
}

/// Trait for all backends
#[async_trait]
pub trait Backend {
    async fn publish_crate(
        &self,
        token: &str,
        request: &models::PublishRequest,
    ) -> Result<(), reqwest::Error>;

    async fn yank_crate(
        &self,
        token: &str,
        request: &models::YankRequest,
    ) -> Result<(), reqwest::Error>;
    async fn is_version_published(
        &self,
        token: &str,
        crate_name: &str,
        crate_version: &str,
    ) -> Result<bool, reqwest::Error>;
}

/// Trait for all storages
#[async_trait]
pub trait Storage {
    async fn get(
        &self,
        crate_name: &str,
        crate_version: &str,
    ) -> Result<Vec<u8>, models::StorageError>;
    async fn put(
        &mut self,
        crate_name: &str,
        crate_version: &str,
        data: &[u8],
    ) -> Result<(), models::StorageError>;
}

// Runnable backend service
pub struct BackendService<T: Backend + Sync + Send> {
    backend: T,
}

impl<T: Backend + Sync + Send + 'static> BackendService<T> {
    pub fn new(backend: T) -> Self {
        Self { backend }
    }

    pub fn run(
        self,
    ) -> (
        tokio::task::JoinHandle<()>,
        tokio::sync::mpsc::Sender<BackendCommand>,
    ) {
        let (sender, mut receiver) = tokio::sync::mpsc::channel::<BackendCommand>(16);
        let handle = tokio::spawn(async move {
            loop {
                match receiver.recv().await {
                    Some(command) => match command {
                        BackendCommand::Publish(token, req, sender) => {
                            match self.backend.publish_crate(&token, &req).await {
                                Ok(_) => {
                                    if sender.send(true).is_err() {
                                        tracing::error!("Failed to send publish result!");
                                    }
                                }
                                Err(e) => {
                                    tracing::error!("Publish failed: {}", e);
                                    if sender.send(false).is_err() {
                                        tracing::error!("Failed to send publish result!");
                                    }
                                }
                            }
                        }
                        BackendCommand::Yank(token, req, sender) => {
                            match self.backend.yank_crate(&token, &req).await {
                                Ok(_) => {
                                    if sender.send(true).is_err() {
                                        tracing::error!("Failed to send yank result!");
                                    }
                                }
                                Err(e) => {
                                    tracing::error!("Publish failed: {}", e);
                                    if sender.send(false).is_err() {
                                        tracing::error!("Failed to send yank result!");
                                    }
                                }
                            }
                        }
                        BackendCommand::IsVersionPublished(token, name, version, sender) => {
                            match self.backend.is_version_published(&token, &name, &version).await {
                                Ok(_) => {
                                    if sender.send(true).is_err() {
                                        tracing::error!("Failed to send isPublished result!");
                                    }
                                }
                                Err(e) => {
                                    tracing::error!("Publish failed: {}", e);
                                    if sender.send(false).is_err() {
                                        tracing::error!("Failed to send isPublished result!");
                                    }
                                }
                            }
                        }
                    },
                    None => {
                        tracing::warn!("Did not receive a BackendCommand!")
                    }
                }
            }
        });

        (handle, sender)
    }
}

// Runnable storage service
pub struct StorageService<T: Storage + Sync + Send> {
    storage: T,
}

impl<T: Storage + Sync + Send + 'static> StorageService<T> {
    pub fn new(storage: T) -> Self {
        Self { storage }
    }

    pub fn run(
        mut self,
    ) -> (
        tokio::task::JoinHandle<()>,
        tokio::sync::mpsc::Sender<StorageCommand>,
    ) {
        let (sender, mut receiver) = tokio::sync::mpsc::channel::<StorageCommand>(16);
        let handle = tokio::spawn(async move {
            loop {
                match receiver.recv().await {
                    Some(command) => match command {
                        StorageCommand::Get(req) => {
                            match self.storage.get(&req.crate_name, &req.crate_version).await {
                                Ok(data) => {
                                    if req.result_sender.send(Some(data)).is_err() {
                                        tracing::error!("Failed to send storage result!");
                                    }
                                }
                                Err(e) => {
                                    tracing::error!("Storage get failed: {}", e);
                                    if req.result_sender.send(None).is_err() {
                                        tracing::error!("Failed to send storage result!");
                                    }
                                }
                            }
                        }
                        StorageCommand::Put(req) => {
                            match self
                                .storage
                                .put(&req.crate_name, &req.crate_version, &req.data)
                                .await
                            {
                                Ok(_) => {
                                    if req.result_sender.send(true).is_err() {
                                        tracing::error!("Failed to send storage result!");
                                    }
                                }
                                Err(e) => {
                                    tracing::error!("Storage put failed: {}", e);
                                    if req.result_sender.send(false).is_err() {
                                        tracing::error!("Failed to send storage result!");
                                    }
                                }
                            }
                        }
                    },
                    None => {
                        tracing::warn!("Did not receive a StorageCommand!")
                    }
                }
            }
        });

        (handle, sender)
    }
}
