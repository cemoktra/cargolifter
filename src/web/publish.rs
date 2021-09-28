use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use axum::async_trait;
use axum::extract::{self, FromRequest, RequestParts};
use bytes::Buf;
use hyper::StatusCode;
use serde::Deserialize;
use serde_json::from_slice;

use crate::storage::Storage;
use crate::web::service::RegistryGit;

#[derive(Clone, Debug, Deserialize)]
pub struct Dependency {
    pub name: String,
    pub version_req: String,
    pub features: Vec<String>,
    pub optional: bool,
    pub default_features: bool,
    pub target: Option<String>,
    pub kind: String,
    pub registry: Option<String>,
    pub package: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct MetaData {
    pub name: String,
    pub vers: String,
    pub deps: Vec<Dependency>,
    pub features: HashMap<String, Vec<String>>,
    pub authors: Vec<String>,
    pub description: Option<String>,
    pub documentation: Option<String>,
    pub homepage: Option<String>,
    pub readme: Option<String>,
    pub readme_file: Option<String>,
    pub keywords: Vec<String>,
    pub categories: Vec<String>,
    pub license: Option<String>,
    pub license_file: Option<String>,
    pub repository: Option<String>,
    pub badges: HashMap<String, HashMap<String, String>>,
    pub links: Option<String>,
}

#[derive(Debug)]
pub struct PublishRequest {
    pub meta: MetaData,
    pub data: Vec<u8>,
}

#[async_trait]
impl FromRequest for PublishRequest {
    type Rejection = axum::http::StatusCode;

    async fn from_request(req: &mut RequestParts) -> Result<Self, Self::Rejection> {
        let mut data = hyper::body::to_bytes(req.body_mut().unwrap())
            .await
            .unwrap();
        let json_length = data.get_u32_le() as usize;
        let json_data = &data[0..json_length].to_vec();
        data.advance(json_length);
        let data_length = data.get_u32_le() as usize;

        Ok(Self {
            meta: from_slice(json_data).unwrap(),
            data: data[0..data_length].to_vec(),
        })
    }
}

pub async fn publish<T: Storage>(
    request: PublishRequest,
    storage: extract::Extension<Arc<RwLock<T>>>,
    git: extract::Extension<RegistryGit>,
) -> Result<(), StatusCode> {
    // TODO: check auth
    log::info!(
        "publishing '{}' in version '{}'",
        request.meta.name,
        request.meta.vers
    );

    let git = git.0;
    match git.0 {
        Some(git) => match git.lock() {
            Ok(git) => match git.publish(&request) {
                Ok(_) => {
                    let mut storage_lock = storage.write().map_err(|e| {
                        log::error!("Failed to get write lock on storage: {}", e);
                        StatusCode::INTERNAL_SERVER_ERROR
                    })?;
                    match storage_lock.put(
                        &request.meta.name,
                        &request.meta.vers,
                        true,
                        &request.data,
                    ) {
                        Ok(_) => Ok(()),
                        Err(_) => {
                            log::error!("failed to publish crate to storage");
                            Err(StatusCode::INTERNAL_SERVER_ERROR)
                        }
                    }
                }
                Err(_) => {
                    log::error!("failed to publish crate to git repo");
                    Err(StatusCode::INTERNAL_SERVER_ERROR)
                }
            },
            Err(_) => {
                log::error!("cannot get lock in git repo");
                Err(StatusCode::INTERNAL_SERVER_ERROR)
            }
        },
        None => {
            log::error!("cannot access git repo");
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
