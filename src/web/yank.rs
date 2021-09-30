use std::sync::Arc;

use crate::git::service::{GitRegistryCommand, YankRequest};
use axum::{extract, Json};
use serde::Serialize;
use tokio::sync::{mpsc::Sender, oneshot};

#[derive(Clone, Debug, Serialize)]
pub struct YankResult {
    ok: bool,
}

pub async fn yank(
    extract::Path((crate_name, crate_version)): extract::Path<(String, String)>,
    git: extract::Extension<Arc<Sender<GitRegistryCommand>>>,
) -> Json<YankResult> {
    // TODO: check auth
    log::info!(
        "requested yanking of {} version {}",
        crate_name,
        crate_version
    );

    let (tx, rx) = oneshot::channel::<bool>();
    let request = YankRequest {
        crate_name,
        crate_version,
        yank: true,
        result_sender: tx,
    };
    match git.send(GitRegistryCommand::Yank(request)).await {
        Ok(_) => match rx.await {
            Ok(result) => {
                if result {
                    Json(YankResult { ok: true })
                } else {
                    Json(YankResult { ok: false })
                }
            }
            Err(e) => {
                log::error!("Failed to receive git response: {}", e);
                Json(YankResult { ok: false })
            }
        },
        Err(e) => {
            log::error!("Failed to send git command: {}", e);
            Json(YankResult { ok: false })
        }
    }
}

pub async fn unyank(
    extract::Path((crate_name, crate_version)): extract::Path<(String, String)>,
    git: extract::Extension<Arc<Sender<GitRegistryCommand>>>,
) -> Json<YankResult> {
    // TODO: check auth
    log::info!(
        "requested unyanking of {} version {}",
        crate_name,
        crate_version
    );

    let (tx, rx) = oneshot::channel::<bool>();
    let request = YankRequest {
        crate_name,
        crate_version,
        yank: false,
        result_sender: tx,
    };
    match git.send(GitRegistryCommand::Yank(request)).await {
        Ok(_) => match rx.await {
            Ok(result) => {
                if result {
                    Json(YankResult { ok: true })
                } else {
                    Json(YankResult { ok: false })
                }
            }
            Err(e) => {
                log::error!("Failed to receive git response: {}", e);
                Json(YankResult { ok: false })
            }
        },
        Err(e) => {
            log::error!("Failed to send git command: {}", e);
            Json(YankResult { ok: false })
        }
    }
}
