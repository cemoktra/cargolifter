use crate::web::download::*;
use crate::web::publish::*;
use crate::web::yank::*;
use crate::{git::service::GitService, storage::Storage};
use axum::{
    handler::{delete, get, put},
    AddExtensionLayer, Router,
};
use std::sync::{Arc, Mutex, RwLock};

#[derive(Clone)]
pub struct MirrorGit(pub Option<Arc<Mutex<GitService>>>);

#[derive(Clone)]
pub struct RegistryGit(pub Option<Arc<Mutex<GitService>>>);

pub struct WebService<T> {
    // mirror: MirrorGit,
    registry: RegistryGit,
    storage: Arc<RwLock<T>>,
    port: i32,
}

impl<T: 'static + Storage + Send + Sync> WebService<T> {
    pub fn new(
        // mirror: Option<Arc<Mutex<GitService>>>,
        registry: Option<Arc<Mutex<GitService>>>,
        storage: Arc<RwLock<T>>,
        port: i32,
    ) -> Self {
        Self {
            // mirror: MirrorGit { 0: mirror },
            registry: RegistryGit { 0: registry },
            storage,
            port,
        }
    }

    pub async fn run(&self) {
        let app = Router::new()
            .route(
                "/api/v1/crates/:crate_name/:crate_version/download",
                get(registry_download::<T>),
            )
            .route(
                "/api/v1/mirror/:crate_name/:crate_version/download",
                get(mirror_download::<T>),
            )
            .route("/api/v1/crates/new", put(publish::<T>))
            .route("/api/v1/crates/:name/:version/yank", delete(yank))
            .route("/api/v1/crates/:name/:version/unyank", put(unyank))
            // .route("/api/v1/crates/:name/owners", get(list_owners).put(add_owner).delete(remove_owner))
            // .route("/api/v1/crates", get(search_registry))
            // .route("/mirror/api/v1/crates", get(search_mirror))
            .layer(AddExtensionLayer::new(self.storage.clone()))
            .layer(AddExtensionLayer::new(self.registry.clone()));

        let host = format!("0.0.0.0:{}", self.port);

        log::info!("starting web service at: {}", host);

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
