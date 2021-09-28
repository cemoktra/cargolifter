use crate::git::service::GitService;
use std::sync::{Arc, Mutex};

pub struct MirrorService {
    git: Arc<Mutex<GitService>>,
}

impl MirrorService {
    pub fn new(git: Arc<Mutex<GitService>>) -> Self {
        Self { git }
    }

    pub fn run(&self) -> tokio::task::JoinHandle<()> {
        let git = self.git.clone();
        tokio::task::spawn(async move {
            loop {
                log::info!("syncing with crates.io");
                match git.lock() {
                    Ok(lock) => match lock.pull_crates_io() {
                        Ok(_) => {}
                        Err(e) => {
                            log::error!("failed to mirror crates.io: {}", e)
                        }
                    },
                    Err(e) => {
                        log::error!("failed to receive lock on mirror repo: {}", e)
                    }
                };
                tokio::time::sleep(tokio::time::Duration::from_secs(1800)).await;
            }
        })
    }
}
