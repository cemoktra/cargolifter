use crate::git::service::GitMirrorCommand;
use crate::git::service::GitRegistryCommand;
use crate::storage::StorageCommand;
use crate::web::download::*;
use crate::web::publish::*;
use crate::web::yank::*;
use axum::{
    handler::{delete, get, put},
    AddExtensionLayer, Router,
};
use std::sync::Arc;
use tokio::sync::mpsc::Sender;

pub struct WebService {
    mirror: Arc<Sender<GitMirrorCommand>>,
    registry: Arc<Sender<GitRegistryCommand>>,
    storage: Arc<Sender<StorageCommand>>,
    port: i32,
}

impl WebService {
    pub fn new(
        mirror: Sender<GitMirrorCommand>,
        registry: Sender<GitRegistryCommand>,
        storage: Sender<StorageCommand>,
        port: i32,
    ) -> Self {
        Self {
            mirror: Arc::new(mirror),
            registry: Arc::new(registry),
            storage: Arc::new(storage),
            port,
        }
    }

    pub async fn run(&self) {
        let host = format!("0.0.0.0:{}", self.port);
        log::info!("starting web service at: {}", host);

        let app = Router::new()
            .route(
                "/api/v1/crates/:crate_name/:crate_version/download",
                get(registry_download),
            )
            .route(
                "/api/v1/mirror/:crate_name/:crate_version/download",
                get(mirror_download),
            )
            .route("/api/v1/crates/new", put(publish))
            .route("/api/v1/crates/:name/:version/yank", delete(yank))
            .route("/api/v1/crates/:name/:version/unyank", put(unyank))
            // .route("/api/v1/crates/:name/owners", get(list_owners).put(add_owner).delete(remove_owner))
            // .route("/api/v1/crates", get(search_registry))
            // .route("/mirror/api/v1/crates", get(search_mirror))
            .layer(AddExtensionLayer::new(self.storage.clone()))
            .layer(AddExtensionLayer::new(self.registry.clone()))
            .layer(AddExtensionLayer::new(self.mirror.clone()));

        axum::Server::bind(&host.parse().unwrap())
            .serve(app.into_make_service())
            .await
            .unwrap();
    }
}

// async fn list_owners(extract::Path(crate_name): extract::Path<String>) {
//     log::info!("list owners called for {}", crate_name);
// }

// async fn add_owner(extract::Path(crate_name): extract::Path<String>) {
//     log::info!("add owner called for {}", crate_name);
// }

// async fn remove_owner(extract::Path(crate_name): extract::Path<String>) {
//     log::info!("remove owner called for {}", crate_name);
// }

// async fn search_registry() {
//     log::info!("search registry called");
// }

// async fn search_mirror() {
//     log::info!("search mirror called");
// }
