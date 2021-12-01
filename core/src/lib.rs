pub mod commands;
pub mod config;
pub mod models;

use async_trait::async_trait;
use commands::{publish, is_published, yank};
use models::PublishedVersion;

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

pub enum StorageCommand {
    Get(models::StorageGetRequest),
    Put(models::StoragePutRequest),
}

#[async_trait]
pub trait Backend {
    async fn get_file(
        &self,
        token: &str,
        crate_path: &str,
    ) -> Result<(String, String, String), reqwest::Error>;

    async fn create_file(
        &self,
        token: &str,
        crate_path: &str,
        branch_name: &str,
        version: &PublishedVersion,
    ) -> Result<(), reqwest::Error>;

    async fn update_file(
        &self,
        token: &str,
        crate_path: &str,
        branch_name: &str,
        versions: &[PublishedVersion],
        current_sha: &str,
    ) -> Result<(), reqwest::Error>;

    async fn delete_branch(
        &self,
        token: &str,
        branch_name: &str,
    ) -> Result<(), reqwest::Error>;

    async fn create_pull_request(
        &self,
        token: &str,
        title: &str,
        branch_name: &str,
    ) -> Result<u64, reqwest::Error>;

    async fn merge_pull_request(
        &self,
        token: &str,
        id: u64,
    ) -> Result<(), reqwest::Error>;

    async fn delete_pull_request(
        &self,
        token: &str,
        id: u64,
    ) -> Result<(), reqwest::Error>;
}

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

pub fn get_crate_path(name: &str) -> String {
    match name.len() {
        1 => "1".into(),
        2 => "2".into(),
        3 => format!("3/{}", name[0..1].to_string()),
        _ => {
            format!("{}/{}", name[0..2].to_string(), name[2..4].to_string())
        }
    }
}

pub fn get_crate_file_path(name: &str) -> String {
    format!("{}/{}", get_crate_path(name), name)
}

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
                            match publish::execute(&self.backend, &token, &req).await {
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
                            match yank::execute(&self.backend, &token, &req).await {
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
                            match is_published::execute(&self.backend, &token, &name, &version).await {
                                Ok(result) => {
                                    if sender.send(result).is_err() {
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
