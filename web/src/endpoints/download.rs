pub async fn download(
    axum::extract::Path((crate_name, crate_version)): axum::extract::Path<(String, String)>,
    storage: axum::extract::Extension<tokio::sync::mpsc::Sender<cargolifter_core::StorageCommand>>,
) -> Result<Vec<u8>, axum::http::StatusCode> {
    tracing::info!(
        "requtested download of '{}' in version '{}'",
        crate_name,
        crate_version
    );

    let (tx, rx) = tokio::sync::oneshot::channel::<Option<Vec<u8>>>();
    let request = cargolifter_core::models::StorageGetRequest {
        crate_name: crate_name.into(),
        crate_version: crate_version.into(),
        result_sender: tx,
    };

    match storage.send(cargolifter_core::StorageCommand::Get(request)).await {
        Ok(_) => match rx.await {
            Ok(result) => match result {
                Some(data) => Ok(data),
                None => Err(axum::http::StatusCode::NOT_FOUND),
            },
            Err(e) => {
                tracing::error!("Failed to receive storage response: {}", e);
                Err(axum::http::StatusCode::INTERNAL_SERVER_ERROR)
            }
        },
        Err(e) => {
            tracing::error!("Failed to send storage request: {}", e);
            Err(axum::http::StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}