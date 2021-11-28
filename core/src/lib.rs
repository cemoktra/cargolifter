pub mod config;
pub mod models;

use async_trait::async_trait;

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
}

pub enum StorageCommand {
    Get(models::StorageGetRequest),
    Put(models::StoragePutRequest),
}

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
        data: &Vec<u8>,
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
                            match self.backend.publish_crate(&token, &req).await {
                                Ok(_) => {
                                    if let Err(_) = sender.send(true) {
                                        tracing::error!("Failed to send publish result!");
                                    }
                                }
                                Err(e) => {
                                    tracing::error!("Publish failed: {}", e);
                                    if let Err(_) = sender.send(false) {
                                        tracing::error!("Failed to send publish result!");
                                    }
                                }
                            }
                        }
                        BackendCommand::Yank(token, req, sender) => {
                            match self.backend.yank_crate(&token, &req).await {
                                Ok(_) => {
                                    if let Err(_) = sender.send(true) {
                                        tracing::error!("Failed to send yank result!");
                                    }
                                }
                                Err(e) => {
                                    tracing::error!("Publish failed: {}", e);
                                    if let Err(_) = sender.send(false) {
                                        tracing::error!("Failed to send yank result!");
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
                                    if let Err(_) = req.result_sender.send(Some(data)) {
                                        tracing::error!("Failed to send storage result!");
                                    }
                                }
                                Err(e) => {
                                    tracing::error!("Storage get failed: {}", e);
                                    if let Err(_) = req.result_sender.send(None) {
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
                                    if let Err(_) = req.result_sender.send(true) {
                                        tracing::error!("Failed to send storage result!");
                                    }
                                }
                                Err(e) => {
                                    tracing::error!("Storage put failed: {}", e);
                                    if let Err(_) = req.result_sender.send(false) {
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
