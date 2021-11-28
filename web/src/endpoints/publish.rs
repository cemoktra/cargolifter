use cargolifter_core::{BackendCommand, StorageCommand, models::PublishRequest};

pub async fn publish(
    request: crate::RequestExtractor,
    headers: axum::http::HeaderMap,
    backend: axum::extract::Extension<tokio::sync::mpsc::Sender<cargolifter_core::BackendCommand>>,
    storage: axum::extract::Extension<tokio::sync::mpsc::Sender<cargolifter_core::StorageCommand>>,
) -> Result<(), axum::http::StatusCode> {
    let request = request.0;
    tracing::info!(
        "publishing '{}' in version '{}'",
        request.meta.name,
        request.meta.vers
    );
    let token = match headers.get("authorization") {
        Some(token) => token.to_str().unwrap(),
        None => {
            return Err(axum::http::StatusCode::UNAUTHORIZED);
        }
    };

    // TODO: store to storage
    publish_to_storage(storage.0, &request.meta.name, &request.meta.vers, request.data.clone()).await?;
    publish_to_backend(backend.0, request, token).await?;

    Ok(())
}

async fn publish_to_storage(
    storage: tokio::sync::mpsc::Sender<cargolifter_core::StorageCommand>,
    crate_name: &str,
    crate_version: &str,
    data: Vec<u8>
) -> Result<(), axum::http::StatusCode> {
    let (tx, rx) = tokio::sync::oneshot::channel::<bool>();
    let put_request = cargolifter_core::models::StoragePutRequest {
        crate_name: crate_name.into(),
        crate_version: crate_version.into(),
        data,
        result_sender: tx,
    };

    match storage
        .send(StorageCommand::Put(put_request))
        .await
    {
        Ok(_) => match rx.await {
            Ok(result) => {
                if result {
                    Ok(())
                } else {
                    tracing::error!("Failed store crate");
                    Err(axum::http::StatusCode::INTERNAL_SERVER_ERROR)
                }
            }
            Err(e) => {
                tracing::error!("Failed to receive storage response: {}", e);
                Err(axum::http::StatusCode::INTERNAL_SERVER_ERROR)
            }
        },
        Err(e) => {
            tracing::error!("Failed to send storage command: {}", e);
            Err(axum::http::StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}


async fn publish_to_backend(
    backend: tokio::sync::mpsc::Sender<cargolifter_core::BackendCommand>,
    request: PublishRequest,
    token: &str,
) -> Result<(), axum::http::StatusCode> {
    let (tx, rx) = tokio::sync::oneshot::channel::<bool>();
    match backend
        .send(BackendCommand::Publish(token.into(), request, tx))
        .await
    {
        Ok(_) => match rx.await {
            Ok(result) => {
                if result {
                    Ok(())
                } else {
                    tracing::error!("Failed publish crate");
                    Err(axum::http::StatusCode::INTERNAL_SERVER_ERROR)
                }
            }
            Err(e) => {
                tracing::error!("Failed to receive backend response: {}", e);
                Err(axum::http::StatusCode::INTERNAL_SERVER_ERROR)
            }
        },
        Err(e) => {
            tracing::error!("Failed to send backend command: {}", e);
            Err(axum::http::StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}