use tokio::sync::mpsc::Sender;

use crate::git::service::GitMirrorCommand;

pub struct MirrorService;

impl MirrorService {
    pub fn run(sender: Sender<GitMirrorCommand>) -> tokio::task::JoinHandle<()> {
        tokio::task::spawn(async move {
            loop {
                log::info!("syncing with crates.io");
                let (tx, rx) = tokio::sync::oneshot::channel::<bool>();
                match sender.send(GitMirrorCommand::Mirror(tx)).await {
                    Ok(_) => match rx.await {
                        Ok(result) => {
                            log::info!("mirror completed with result: {}", result);
                        }
                        Err(e) => {
                            log::error!("failed to receive mirror result: {}", e)
                        }
                    },
                    Err(e) => {
                        log::error!("failed to send mirror command: {}", e)
                    }
                }
                tokio::time::sleep(tokio::time::Duration::from_secs(1800)).await;
            }
        })
    }
}
