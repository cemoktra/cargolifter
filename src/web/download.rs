use crate::storage::Storage;
use axum::extract;
use axum::http::StatusCode;
use std::io::Read;
use std::sync::{Arc, RwLock};

pub async fn registry_download<T: Storage>(
    extract::Path((crate_name, crate_version)): extract::Path<(String, String)>,
    storage: extract::Extension<Arc<RwLock<T>>>,
) -> Result<Vec<u8>, StatusCode> {
    log::info!(
        "requtested registry download of '{}' in version '{}'",
        crate_name,
        crate_version
    );

    let storage = storage.read().map_err(|e| {
        log::error!("Failed to get read lock on storage: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    match storage.get(&crate_name, &crate_version, false) {
        Ok(data) => Ok(data),
        Err(_) => {
            log::error!(
                "failed to read '{}' version '{}' from storage!",
                crate_name,
                crate_version
            );
            Err(StatusCode::NOT_FOUND)
        }
    }
}

pub async fn mirror_download<T: Storage>(
    extract::Path((crate_name, crate_version)): extract::Path<(String, String)>,
    storage: extract::Extension<Arc<RwLock<T>>>,
) -> Result<Vec<u8>, StatusCode> {
    log::info!(
        "requtested mirrored download of '{}' in version '{}'",
        crate_name,
        crate_version
    );

    let storage_lock = storage.read().map_err(|e| {
        log::error!("Failed to get read lock on storage: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    match storage_lock.get(&crate_name, &crate_version, true) {
        Ok(data) => Ok(data),
        Err(_) => {
            log::warn!(
                "failed to read '{}' version '{}' from storage! Trying crates.io ...",
                crate_name,
                crate_version
            );

            let response = ureq::get(&format!(
                "https://crates.io/api/v1/crates/{}/{}/download",
                crate_name, crate_version
            ))
            .call()
            .map_err(|e| {
                log::error!("Failed to get crate from crates.io: {}", e);
                StatusCode::NOT_FOUND
            })?;
            if response.has("Content-Length") {
                let content_length = response
                    .header("Content-Length")
                    .and_then(|s| s.parse::<usize>().ok())
                    .unwrap_or_default();
                let mut response_bytes: Vec<u8> = Vec::with_capacity(content_length);
                response
                    .into_reader()
                    .read_to_end(&mut response_bytes)
                    .unwrap();

                drop(storage_lock);
                let mut storage_lock = storage.write().map_err(|e| {
                    log::error!("Failed to get write lock on storage: {}", e);
                    StatusCode::INTERNAL_SERVER_ERROR
                })?;
                storage_lock
                    .put(&crate_name, &crate_version, true, &response_bytes)
                    .unwrap();

                Ok(response_bytes)
            } else {
                log::error!("crates.io response has no content-length!");
                Err(StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
    }
}
