use argh::FromArgs;
use storage::FileSystemStorage;

mod config;
mod git;
mod mirror;
mod storage;
mod web;
mod tools;

use crate::{config::cargolifter::CargoLifterConfig, git::GitService, mirror::MirrorService, web::service::WebService};

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
    let (mirror_git, mirror_service) = match config.mirror {
        Some(config) => {
            let git = GitService::from_config(&config)?;
            let mirror_service = MirrorService::new(git.clone());
            (Some(git), Some(mirror_service))
        }
        None => (None, None),
    };
    // init registry git
    let registry_git = match config.registry {
        Some(config) => Some(GitService::from_config(&config)?),
        None => None,
    };

    let _mirror_join = if let Some(service) = mirror_service {
        Some(service.run())
    } else {
        None
    };

    let storage = FileSystemStorage::new(&config.storage_config.path);

    // init web service
    let web_service = WebService::new(mirror_git, registry_git, storage, config.service_port);
    web_service.run().await;

    Ok(())
}
