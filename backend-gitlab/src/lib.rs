mod api;
mod models;

use async_trait::async_trait;
use cargolifter_core::models::{PublishRequest, PublishedVersion, YankRequest};
use cargolifter_core::Backend;

pub struct Gitlab {
    cargolifter_token: Option<String>,
    project_id: usize,
    host: Option<String>,
    default_branch: String,
}

impl Gitlab {
    pub fn from(config: cargolifter_core::config::GitlabConfig) -> Self {
        Self {
            cargolifter_token: config.cargolifter_token,
            project_id: config.project_id,
            host: config.host.clone(),
            default_branch: config.default_branch.unwrap_or_else(|| String::from("main")),
        }
    }

    fn host(&self) -> String {
        let default_host = String::from("https://gitlab.com");
        self.host.as_ref().unwrap_or(&default_host).into()
    }

    async fn merge_change(
        &self,
        token: &str,
        name: &str,
        vers: &str,
        title: &str,
    ) -> Result<(), reqwest::Error> {
        let branch_name = format!("{}-{}", name, vers);

        let owned_token = token.to_owned();
        let merge_token = self.cargolifter_token.as_ref().unwrap_or(&owned_token);
        let host = self.host();

        tracing::info!("creating MR for branch '{}'!", branch_name);
        let merge_request = models::create_merge_request::Request {
            source_branch: branch_name.clone(),
            target_branch: self.default_branch.clone(),
            title: title.into(),
            remove_source_branch: Some(true),
            ..Default::default()
        };
        match api::create_merge_request(&host, token, self.project_id, &merge_request).await {
            Ok(response) => {
                let accept_request = models::accept_merge_request::Request {
                    should_remove_source_branch: Some(true),
                    ..Default::default()
                };

                match api::accept_merge_request(
                    &host,
                    merge_token,
                    self.project_id,
                    response.iid,
                    &accept_request,
                )
                .await
                {
                    Ok(_) => Ok(()),
                    Err(e) => {
                        tracing::error!("failed to accept MR - deleting branch");
                        api::delete_branch(&host, token, self.project_id, &branch_name.clone())
                            .await?;
                        Err(e)
                    }
                }
            }
            Err(e) => {
                tracing::error!("failed to create MR - deleting branch");
                api::delete_branch(&host, token, self.project_id, &branch_name).await?;
                Err(e)
            }
        }
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
        let host = self.host();

        match api::get_file(
            &host,
            token,
            self.project_id,
            &crate_path,
            &self.default_branch,
        )
        .await
        {
            Ok(response) => {
                tracing::info!("'{}' already found! updating!", crate_path);
                let update_request = crate::models::update_file::Request {
                    branch: branch_name.clone(),
                    start_branch: Some(self.default_branch.clone()),
                    content: cargolifter_core::utils::add_version(
                        request.into(),
                        &response.content,
                        &response.encoding,
                    ),
                    encoding: Some("base64".into()),
                    commit_message: format!("Adding {} {}", request.meta.name, request.meta.vers),
                    ..Default::default()
                };
                api::update_file(&host, token, self.project_id, &crate_path, &update_request)
                    .await?;
            }
            Err(_) => {
                let initial_version: PublishedVersion = request.into();
                let json = serde_json::to_string(&initial_version).unwrap();
                let create_request = crate::models::create_file::Request {
                    branch: branch_name.clone(),
                    start_branch: Some(self.default_branch.clone()),
                    content: json,
                    commit_message: format!("Adding {} {}", request.meta.name, request.meta.vers),
                    ..Default::default()
                };

                api::create_file(&host, token, self.project_id, &crate_path, &create_request)
                    .await?;
            }
        }

        self.merge_change(
            token,
            &request.meta.name,
            &request.meta.vers,
            &format!("{}-{}", request.meta.name, request.meta.vers),
        )
        .await?;
        tracing::info!("{} publishes successfully", crate_path);
        Ok(())
    }

    async fn yank_crate(&self, token: &str, request: &YankRequest) -> Result<(), reqwest::Error> {
        let crate_path = cargolifter_core::get_crate_file_path(&request.name);
        let branch_name = format!(
            "{}-{}-{}",
            if request.yank { "yank" } else { "unyank" },
            request.name,
            request.vers
        );
        let host = self.host();

        match api::get_file(
            &host,
            token,
            self.project_id,
            &crate_path,
            &self.default_branch,
        )
        .await
        {
            Ok(response) => {
                match cargolifter_core::utils::updated_yanked(
                    &response.content,
                    &response.encoding,
                    &request.name,
                    &request.vers,
                    request.yank,
                ) {
                    Some(encoded_content) => {
                        let update_request = crate::models::update_file::Request {
                            branch: branch_name.clone(),
                            start_branch: Some(self.default_branch.clone()),
                            content: encoded_content,
                            encoding: Some("base64".into()),
                            commit_message: format!(
                                "{} {} {}",
                                if request.yank { "yanking" } else { "unyanking" },
                                request.name,
                                request.vers
                            ),
                            ..Default::default()
                        };
                        match api::update_file(
                            &host,
                            token,
                            self.project_id,
                            &crate_path,
                            &update_request,
                        )
                        .await
                        {
                            Ok(_) => {}
                            Err(e) => {
                                tracing::error!("failed to udate file - deleting branch");
                                api::delete_branch(&host, token, self.project_id, &branch_name)
                                    .await?;
                                return Err(e);
                            }
                        }
                    }
                    None => return Ok(()),
                }
            }
            Err(e) => {
                tracing::error!("crate {} not found", request.name);
                return Err(e);
            }
        }

        self.merge_change(
            token,
            &request.name,
            &request.vers,
            &format!(
                "{}-{}-{}",
                if request.yank { "yank" } else { "unyank" },
                request.name,
                request.vers
            ),
        )
        .await?;

        tracing::info!(
            "{} {} successfully",
            crate_path,
            if request.yank { "yanked" } else { "unyanked" }
        );

        Ok(())
    }

    async fn is_version_published(
        &self,
        token: &str,
        crate_name: &str,
        crate_version: &str,
    ) -> Result<bool, reqwest::Error> {
        let crate_path = cargolifter_core::get_crate_file_path(crate_name);
        let host = self.host();

        tracing::info!(
            "checking if version {} of {} has been published",
            crate_version,
            crate_name
        );

        match api::get_file(
            &host,
            token,
            self.project_id,
            &crate_path,
            &self.default_branch,
        )
        .await
        {
            Ok(response) => {
                let versions =
                    cargolifter_core::utils::read_versions(&response.content, &response.encoding);
                Ok(versions
                    .iter()
                    .any(|v| v.name == crate_name && v.vers == crate_version))
            }
            Err(e) => {
                tracing::error!("crate {} not found", crate_name);
                Err(e)
            }
        }
    }
}
