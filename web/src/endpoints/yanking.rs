use cargolifter_core::{models::YankRequest, BackendCommand};

pub async fn yank(
    axum::extract::Path((crate_name, crate_version)): axum::extract::Path<(String, String)>,
    headers: axum::http::HeaderMap,
    backend: axum::extract::Extension<tokio::sync::mpsc::Sender<cargolifter_core::BackendCommand>>,
) -> Result<(), axum::http::StatusCode> {
    tracing::info!("yanking '{}' in version '{}'", crate_name, crate_version);
    let token = match headers.get("authorization") {
        Some(token) => token.to_str().unwrap(),
        None => {
            return Err(axum::http::StatusCode::UNAUTHORIZED);
        }
    };

    let request = YankRequest {
        name: crate_name.clone(),
        vers: crate_version.clone(),
        yank: true,
    };

    yank_at_backend(backend.0, request, token).await?;
    Ok(())
}

pub async fn unyank(
    axum::extract::Path((crate_name, crate_version)): axum::extract::Path<(String, String)>,
    headers: axum::http::HeaderMap,
    backend: axum::extract::Extension<tokio::sync::mpsc::Sender<cargolifter_core::BackendCommand>>,
) -> Result<(), axum::http::StatusCode> {
    tracing::info!("unyanking '{}' in version '{}'", crate_name, crate_version);
    let token = match headers.get("authorization") {
        Some(token) => token.to_str().unwrap(),
        None => {
            return Err(axum::http::StatusCode::UNAUTHORIZED);
        }
    };

    let request = YankRequest {
        name: crate_name.clone(),
        vers: crate_version.clone(),
        yank: false,
    };

    yank_at_backend(backend.0, request, token).await?;
    Ok(())
}

async fn yank_at_backend(
    backend: tokio::sync::mpsc::Sender<cargolifter_core::BackendCommand>,
    request: YankRequest,
    token: &str,
) -> Result<(), axum::http::StatusCode> {
    let (tx, rx) = tokio::sync::oneshot::channel::<bool>();
    match backend
        .send(BackendCommand::Yank(token.into(), request, tx))
        .await
    {
        Ok(_) => match rx.await {
            Ok(result) => {
                if result {
                    Ok(())
                } else {
                    tracing::error!("Failed yank/unyank crate");
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
