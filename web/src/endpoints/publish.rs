use cargolifter_core::{models::PublishRequest, BackendCommand};

pub async fn publish(
    request: crate::RequestExtractor,
    headers: axum::http::HeaderMap,
    backend: axum::extract::Extension<tokio::sync::mpsc::Sender<cargolifter_core::BackendCommand>>,
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

    // let data = request.data.clone();
    // let crate_name = request.meta.name.clone();
    // let crate_vers = request.meta.vers.clone();

    publish_to_backend(backend.0, request, token).await?;

    Err(axum::http::StatusCode::NOT_IMPLEMENTED)
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
                    tracing::error!("Failed store crate");
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
