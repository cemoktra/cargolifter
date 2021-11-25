pub mod models;

use async_trait::async_trait;

pub enum BackendCommand {
    Publish(
        String,
        models::PublishRequest,
        tokio::sync::oneshot::Sender<bool>,
    ),
}

#[async_trait]
pub trait Backend {
    async fn publish_crate(
        &self,
        token: &str,
        request: &models::PublishRequest,
    ) -> Result<(), reqwest::Error>;
}

pub struct BackendService<T: Backend + Sync + Send> {
    backend: T,
}

impl<T: Backend + Sync + Send + 'static> BackendService<T> {
    pub fn new(backend: T) -> Self {
        Self { backend }
    }

    pub fn run(
        self
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
                                        tracing::error!("Failed to send mirror result!");
                                    }
                                }
                                Err(e) => {
                                    tracing::error!("Publish failed: {}", e);
                                    if let Err(_) = sender.send(false) {
                                        tracing::error!("Failed to send mirror result!");
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
