use std::collections::HashMap;
use std::sync::Arc;

use axum::async_trait;
use axum::extract::{self, FromRequest, RequestParts};
use bytes::Buf;
use hyper::StatusCode;
use serde::Deserialize;
use serde_json::from_slice;
use tokio::sync::mpsc::Sender;
use tokio::sync::oneshot;

use crate::git::service::GitRegistryCommand;
use crate::storage::{PutRequest, StorageCommand};

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

pub async fn publish(
    request: PublishRequest,
    storage: extract::Extension<Arc<Sender<StorageCommand>>>,
    git: extract::Extension<Arc<Sender<GitRegistryCommand>>>,
) -> Result<(), StatusCode> {
    // TODO: check auth
    log::info!(
        "publishing '{}' in version '{}'",
        request.meta.name,
        request.meta.vers
    );

    let data = request.data.clone();
    let crate_name = request.meta.name.clone();
    let crate_vers = request.meta.vers.clone();

    let (tx, rx) = oneshot::channel::<bool>();
    match git.send(GitRegistryCommand::Publish(request, tx)).await {
        Ok(_) => match rx.await {
            Ok(result) => {
                if result {
                    let (tx, rx) = oneshot::channel::<bool>();
                    let put_request = PutRequest {
                        crate_name: crate_name,
                        crate_version: crate_vers,
                        data,
                        result_sender: tx,
                    };
                    match storage.send(StorageCommand::Put(put_request)).await {
                        Ok(_) => match rx.await {
                            Ok(result) => {
                                if result {
                                    Ok(())
                                } else {
                                    log::error!("Failed store crate");
                                    Err(StatusCode::INTERNAL_SERVER_ERROR)
                                }
                            }
                            Err(e) => {
                                log::error!("Failed to receive storage response: {}", e);
                                Err(StatusCode::INTERNAL_SERVER_ERROR)
                            }
                        },
                        Err(e) => {
                            log::error!("Failed to send storage command: {}", e);
                            Err(StatusCode::INTERNAL_SERVER_ERROR)
                        }
                    }
                } else {
                    Err(StatusCode::INTERNAL_SERVER_ERROR)
                }
            }
            Err(e) => {
                log::error!("Failed to receive git response: {}", e);
                Err(StatusCode::INTERNAL_SERVER_ERROR)
            }
        },
        Err(e) => {
            log::error!("Failed to send git command: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
