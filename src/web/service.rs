use crate::{config::storage, git::GitService, storage::Storage};
use std::sync::{Arc, Mutex};
use axum::{AddExtensionLayer, Router, extract, handler::{get, put, delete}};

pub struct WebService<T> {
    state: Arc<WebServiceState<T>>,
    port: i32,
}

struct WebServiceState<T> {
    mirror: Option<Arc<Mutex<GitService>>>,
    registry: Option<Arc<Mutex<GitService>>>,
    storage: T
} 

impl<T: 'static + Send + Sync + Storage> WebService<T> {
    pub fn new(mirror: Option<Arc<Mutex<GitService>>>, registry: Option<Arc<Mutex<GitService>>>, storage: T, port: i32) -> Self {
        Self {
            state: Arc::new(WebServiceState {
                mirror,
                registry,
                storage: storage
            }),
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
            .layer(AddExtensionLayer::new(self.state.clone()))
            ;
        
        let host = format!("0.0.0.0:{}", self.port);

        log::info!("starting web service at: {}", host);

        axum::Server::bind(&host.parse().unwrap())
            .serve(app.into_make_service())
            .await
            .unwrap();
    }
}

async fn download<T: Send + Sync + Storage>(extract::Path((crate_name, crate_version)): extract::Path<(String, String)>, state: extract::Extension<Arc<WebServiceState<T>>>) -> &'static str {
    log::info!("download endpoint called for {}={}", crate_name, crate_version);

    let state = state.0;
    let data = match state.storage.get(&crate_name, &crate_version) {
        Ok(data) => data,
        Err(e) => {
            log::warn!("failed to read {}={} from storage! trying crates.io", crate_name, crate_version);
            match surf::get(format!("https://crates.io/api/v1/crates/{}/{}/download", crate_name, crate_version)).send().await {
                Ok(result) => todo!(),
                Err(_) => {
                    
                },
            };
            Vec::new()
        },
    };

    "Hello World"
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