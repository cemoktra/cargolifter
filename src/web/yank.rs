use crate::web::service::RegistryGit;
use axum::{extract, Json};
use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
pub struct YankResult {
    ok: bool,
}

pub async fn yank(
    extract::Path((crate_name, crate_version)): extract::Path<(String, String)>,
    git: extract::Extension<RegistryGit>,
) -> Json<YankResult> {
    // TODO: check auth
    log::info!(
        "requested yanking of {} version {}",
        crate_name,
        crate_version
    );

    let git = git.0;
    match git.0 {
        Some(git) => match git.lock() {
            Ok(git) => match git.yank(true, &crate_name, &crate_version) {
                Ok(_) => Json(YankResult { ok: true }),
                Err(_) => {
                    log::error!("yanking failed");
                    Json(YankResult { ok: false })
                }
            },
            Err(_) => {
                log::error!("cannot get lock in git repo");
                Json(YankResult { ok: false })
            }
        },
        None => {
            log::error!("cannot access git repo");
            Json(YankResult { ok: false })
        }
    }
}

pub async fn unyank(
    extract::Path((crate_name, crate_version)): extract::Path<(String, String)>,
    git: extract::Extension<RegistryGit>,
) -> Json<YankResult> {
    // TODO: check auth
    log::info!(
        "requested unyanking of {} version {}",
        crate_name,
        crate_version
    );

    let git = git.0;
    match git.0 {
        Some(git) => match git.lock() {
            Ok(git) => match git.yank(false, &crate_name, &crate_version) {
                Ok(_) => Json(YankResult { ok: true }),
                Err(_) => {
                    log::error!("yanking failed");
                    Json(YankResult { ok: false })
                }
            },
            Err(_) => {
                log::error!("cannot get lock in git repo");
                Json(YankResult { ok: false })
            }
        },
        None => {
            log::error!("cannot access git repo");
            Json(YankResult { ok: false })
        }
    }
}
