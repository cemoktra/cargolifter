use argh::FromArgs;
use futures::join;
use git::service::{GitMirror, GitRegistry};
use storage::StorageService;
use web::service::WebService;

mod auth;
mod config;
mod git;
mod mirror;
mod storage;
mod tools;
mod web;

use crate::{
    config::cargolifter::CargoLifterConfig,
    git::service::{GitMirrorCommand, GitRegistryCommand},
    mirror::MirrorService,
};

/// CargoLifter custom registry / crates.io mirror
#[derive(FromArgs)]
struct Arguments {
    /// path to config file
    #[argh(option, short = 'c')]
    config: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::Builder::from_default_env()
        .filter(None, log::LevelFilter::Info)
        .init();

    // parse command line
    let args: Arguments = argh::from_env();

    // read config file
    let file = std::fs::File::open(args.config)?;
    let config: CargoLifterConfig = serde_json::from_reader(std::io::BufReader::new(file))?;

    // init mirror git
    let (mirror_git_handle, mirror_git_sender, mirror_service) = match config.mirror {
        Some(config) => {
            let (git_mirror_handle, mirror_sender) = GitMirror::run(&config);
            let mirror_run_handle = MirrorService::run(mirror_sender.clone());
            (
                Some(git_mirror_handle),
                mirror_sender,
                Some(mirror_run_handle),
            )
        }
        None => {
            let (sender, _) = tokio::sync::mpsc::channel::<GitMirrorCommand>(1);
            (None, sender, None)
        }
    };

    let (git_registry_handle, git_registry_sender) = match config.registry {
        Some(config) => {
            let (handle, sender) = GitRegistry::run(&config);
            (Some(handle), sender)
        }
        None => {
            let (sender, _) = tokio::sync::mpsc::channel::<GitRegistryCommand>(1);
            (None, sender)
        }
    };

    let (storage_handle, storage_sender) = StorageService::run(config.storage.clone());

    // init web service
    let web_service = WebService::new(
        mirror_git_sender,
        git_registry_sender,
        storage_sender,
        config.service.port,
    );
    web_service.run().await;

    if let Some(handle) = mirror_service {
        join!(handle).0.unwrap();
    }
    if let Some(handle) = mirror_git_handle {
        join!(handle).0.unwrap();
    }
    if let Some(handle) = git_registry_handle {
        join!(handle).0.unwrap();
    }
    join!(storage_handle).0.unwrap();

    Ok(())
}
