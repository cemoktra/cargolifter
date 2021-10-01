use crate::storage::{GetRequest, MirrorGetRequest, StorageCommand};
use axum::extract;
use axum::http::StatusCode;
use std::sync::Arc;
use tokio::sync::mpsc::Sender;
use tokio::sync::oneshot;

pub async fn registry_download(
    extract::Path((crate_name, crate_version)): extract::Path<(String, String)>,
    storage: extract::Extension<Arc<Sender<StorageCommand>>>,
) -> Result<Vec<u8>, StatusCode> {
    log::info!(
        "requtested registry download of '{}' in version '{}'",
        crate_name,
        crate_version
    );

    let (tx, rx) = oneshot::channel::<Option<Vec<u8>>>();
    let request = GetRequest {
        crate_name: crate_name.into(),
        crate_version: crate_version.into(),
        result_sender: tx,
    };

    match storage.send(StorageCommand::Get(request)).await {
        Ok(_) => match rx.await {
            Ok(result) => match result {
                Some(data) => Ok(data),
                None => Err(StatusCode::NOT_FOUND),
            },
            Err(e) => {
                log::error!("Failed to receive storage response: {}", e);
                Err(StatusCode::INTERNAL_SERVER_ERROR)
            }
        },
        Err(e) => {
            log::error!("Failed to send storage request: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn mirror_download(
    extract::Path((crate_name, crate_version)): extract::Path<(String, String)>,
    storage: extract::Extension<Arc<Sender<StorageCommand>>>,
) -> Result<Vec<u8>, StatusCode> {
    log::info!(
        "requested mirrored download of '{}' in version '{}'",
        crate_name,
        crate_version
    );

    let (tx, rx) = oneshot::channel::<Option<Vec<u8>>>();
    let request = MirrorGetRequest {
        crate_name: crate_name.into(),
        crate_version: crate_version.into(),
        result_sender: tx,
    };

    match storage.send(StorageCommand::MirrorGet(request)).await {
        Ok(_) => match rx.await {
            Ok(result) => match result {
                Some(data) => Ok(data),
                None => Err(StatusCode::NOT_FOUND),
            },
            Err(e) => {
                log::error!("Failed to receive storage response: {}", e);
                Err(StatusCode::INTERNAL_SERVER_ERROR)
            }
        },
        Err(e) => {
            log::error!("Failed to send storage request: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
