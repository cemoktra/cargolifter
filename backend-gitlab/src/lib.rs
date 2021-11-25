mod api;
mod models;

use async_trait::async_trait;
use cargolifter_core::models::{PublishRequest, PublishedVersion};
use cargolifter_core::Backend;

pub struct Gitlab {
    // cargoliter_token: String,
    project_id: usize,
}

impl Gitlab {
    pub fn new(project_id: usize) -> Self {
        Self { project_id }
    }
}

#[async_trait]
impl Backend for Gitlab {
    async fn publish_crate(
        &self,
        token: &str,
        request: &PublishRequest,
    ) -> Result<(), reqwest::Error> {
        let crate_path = request.meta.crate_file_path();
        let branch_name = format!("{}-{}", request.meta.name, request.meta.vers);

        match api::get_file(token, self.project_id, &crate_path).await {
            Ok(response) => {
                tracing::info!("'{}' already found! updating!", crate_path);

                // TODO: handle encoding (should be text hopefully)
                let new_version: PublishedVersion = request.into();
                let json = serde_json::to_string(&new_version).unwrap();
                let mut versions = response
                    .content
                    .lines()
                    .map(|s| s.into())
                    .collect::<Vec<_>>();
                // TODO: check if version already published
                versions.push(json);

                let update_request = crate::models::update_file::Request {
                    branch: branch_name.clone(),
                    start_branch: Some("main".into()), // TODO: make configurable
                    content: versions.join("\n"),
                    commit_message: format!("Adding {} {}", request.meta.name, request.meta.vers),
                    ..Default::default()
                };
                api::update_file(token, self.project_id, &crate_path, &update_request).await?;
            }
            Err(_) => {
                tracing::info!("'{}' not found! creating!", crate_path);

                let initial_version: PublishedVersion = request.into();
                let json = serde_json::to_string(&initial_version).unwrap();
                let create_request = crate::models::create_file::Request {
                    branch: branch_name.clone(),
                    start_branch: Some("main".into()), // TODO: make configurable
                    content: json,
                    commit_message: format!("Adding {} {}", request.meta.name, request.meta.vers),
                    ..Default::default()
                };

                api::create_file(token, self.project_id, &crate_path, &create_request).await?;
            }
        }

        tracing::info!("creating MR for branch '{}'!", branch_name);
        let merge_request = models::create_merge_request::Request {
            source_branch: branch_name.clone(),
            target_branch: "main".into(), // TODO: make configurable
            title: format!("{}-{}", request.meta.name, request.meta.vers),
            remove_source_branch: Some(true),
            ..Default::default()
        };
        match api::create_merge_request(token, self.project_id, &merge_request).await {
            Ok(response) => {
                let accept_request = models::accept_merge_request::Request {
                    should_remove_source_branch: Some(true),
                    ..Default::default()
                };

                // TODO: use cargo lifter token here
                match api::accept_merge_request(token, self.project_id, response.iid, &accept_request).await {
                    Ok(_) => {
                        tracing::info!("{} publishes successfully", crate_path);
                        Ok(())
                    },
                    Err(e) => {
                        tracing::error!("failed to accept MR - deleting branch");
                        api::delete_branch(token, self.project_id, &branch_name).await?;
                        Err(e)
                    }
                }
            }
            Err(e) => {
                tracing::error!("failed to create MR - deleting branch");
                api::delete_branch(token, self.project_id, &branch_name).await?;
                Err(e)
            }
        }
    }
}
