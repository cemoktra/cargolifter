use argh::FromArgs;
use cargolifter_backend_github::Github;
use cargolifter_backend_gitlab::Gitlab;
use cargolifter_core::{BackendService, StorageService};
use cargolifter_storage_filesystem::FileSystemStorage;
use cargolifter_storage_s3::S3Storage;
use cargolifter_web::WebService;

/// CargoLifter custom registry
#[derive(FromArgs)]
struct Arguments {
    /// path to config file
    #[argh(option, short = 'c')]
    config: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt().init();

    // parse command line
    let args: Arguments = argh::from_env();

    // read config file
    let file = std::fs::File::open(args.config)?;
    let config: cargolifter_core::config::CargoLifterConfig =
        serde_json::from_reader(std::io::BufReader::new(file))?;

    let (backend_handle, backend_sender) = match config.backend {
        cargolifter_core::config::BackendType::Github(config) => {
            let github = Github::from(config);
            let backend = BackendService::new(github);
            backend.run()
        }
        cargolifter_core::config::BackendType::Gitlab(config) => {
            let gitlab = Gitlab::from(config);
            let backend = BackendService::new(gitlab);
            backend.run()
        }
    };

    let (storage_handle, storage_sender) = match config.storage {
        cargolifter_core::config::StorageType::FileSystem(config) => {
            let filesystem = FileSystemStorage::new(&config.path);
            let storage = StorageService::new(filesystem);
            storage.run()
        }
        cargolifter_core::config::StorageType::S3(config) => {
            let s3 = S3Storage::new(config).await;
            let storage = StorageService::new(s3);
            storage.run()
        }
    };

    let web = WebService::new(backend_sender, storage_sender, config.web);
    web.run().await;
    let _ = futures::join!(backend_handle, storage_handle);

    Ok(())
}
