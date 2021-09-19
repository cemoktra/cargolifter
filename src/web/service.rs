use crate::{config::storage, git::GitService, storage::Storage};
use std::sync::{Arc, Mutex, RwLock};
use std::io::Read;
use axum::http::StatusCode;
use axum::response::Html;
use axum::{AddExtensionLayer, Router, extract, handler::{get, put, delete}};

pub struct WebService<T> {
    mirror: Option<Arc<Mutex<GitService>>>,
    registry: Option<Arc<Mutex<GitService>>>,
    storage: Arc<RwLock<T>>,
    port: i32,
}

impl<T: 'static + Storage + Send + Sync> WebService<T> {
    pub fn new(mirror: Option<Arc<Mutex<GitService>>>, registry: Option<Arc<Mutex<GitService>>>, storage: Arc<RwLock<T>>, port: i32) -> Self {
        Self {
            mirror,
            registry,
            storage,
            port
        }
    }

    pub async fn run(&self) {
        let app = Router::new()
            .route("/api/v1/crates/:crate_name/:crate_version/download", get(download::<T>))
            .route("/api/v1/crates/new", put(publish))
            .route("/api/v1/crates/:name/:version/yank", delete(yank))
            .route("/api/v1/crates/:name/:version/unyank", put(unyank))
            .route("/api/v1/crates/:name/owners", get(list_owners).put(add_owner).delete(remove_owner))
            .route("/api/v1/crates", get(search_registry))
            .route("/mirror/api/v1/crates", get(search_mirror))
            .layer(AddExtensionLayer::new(self.storage.clone()))
            ;
        
        let host = format!("0.0.0.0:{}", self.port);

        log::info!("starting web service at: {}", host);

        axum::Server::bind(&host.parse().unwrap())
            .serve(app.into_make_service())
            .await
            .unwrap();
    }
}

async fn download<T: Storage>(extract::Path((crate_name, crate_version)): extract::Path<(String, String)>, storage: extract::Extension<Arc<RwLock<T>>>) -> Result<Vec<u8>, StatusCode> {
    log::info!("download endpoint called for {}={}", crate_name, crate_version);

    let mut storage = storage.write().map_err(|e| {
        log::error!("Failed to get write lock on storage: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    match storage.get(&crate_name, &crate_version) {
        Ok(data) => Ok(data),
        Err(_) => {
            log::warn!("failed to read {}={} from storage! trying crates.io", crate_name, crate_version);
            
            let response = ureq::get(&format!("https://crates.io/api/v1/crates/{}/{}/download", crate_name, crate_version)).call().map_err(|e| {
                log::error!("Failed to get crate from crates.io: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;
            if response.has("Content-Length") {
                let content_length = response.header("Content-Length").and_then(|s| s.parse::<usize>().ok()).unwrap_or_default();
                let mut response_bytes: Vec<u8> = Vec::with_capacity(content_length);
                response.into_reader()
                    .read_to_end(&mut response_bytes).unwrap();
                storage.put(&crate_name, &crate_version, &response_bytes).unwrap();

                Ok(response_bytes)
            } else {
                log::error!("crates.io response has no content-length!");
                Err(StatusCode::INTERNAL_SERVER_ERROR)
            }
        },
    }
}

async fn publish() {
    log::info!("publish endpoint called");
}

async fn yank(extract::Path((crate_name, crate_version)): extract::Path<(String, String)>) {
    log::info!("yank endpoint called for {}={}", crate_name, crate_version);
}

async fn unyank(extract::Path((crate_name, crate_version)): extract::Path<(String, String)>) {
    log::info!("unyank endpoint called for {}={}", crate_name, crate_version);
}

async fn list_owners(extract::Path(crate_name): extract::Path<String>) {
    log::info!("list owners called for {}", crate_name);
}

async fn add_owner(extract::Path(crate_name): extract::Path<String>) {
    log::info!("add owner called for {}", crate_name);
}

async fn remove_owner(extract::Path(crate_name): extract::Path<String>) {
    log::info!("remove owner called for {}", crate_name);
}

async fn search_registry() {
    log::info!("search registry called");
}

async fn search_mirror() {
    log::info!("search mirror called");
}