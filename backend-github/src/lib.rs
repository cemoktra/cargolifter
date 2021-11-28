mod api;
mod models;

use async_trait::async_trait;
use cargolifter_core::models::{PublishRequest, PublishedVersion};
use cargolifter_core::Backend;

pub struct Github {
    cargoliter_token: Option<String>,
    project_id: String,
    host: Option<String>
}

impl Github {
    pub fn from(config: cargolifter_core::config::GithubConfig) -> Self {
        Self {
            cargoliter_token: config.cargolifter_token,
            project_id: [config.owner, config.repo].join("/").into(),
            host: config.host.clone()
        }
    }
}

fn read_versions(content: &str) -> Vec<PublishedVersion> {
    let content = content.replace("\n", "");
    let content = base64::decode(content).unwrap(); // TODO: handle error
    let content = String::from_utf8(content).unwrap(); // TODO: handle error
    content
        .lines()
        .map(|s| serde_json::from_str::<PublishedVersion>(s).unwrap())
        .collect()
}

#[async_trait]
impl Backend for Github {
    async fn publish_crate(
        &self,
        token: &str,
        request: &PublishRequest,
    ) -> Result<(), reqwest::Error> {
        let crate_path = request.meta.crate_file_path();
        let branch_name = format!("{}-{}", request.meta.name, request.meta.vers);

        let credentials = token.split(":").collect::<Vec<_>>();
        let owned_token = token.to_owned();
        let merge_credentials = self.cargoliter_token.as_ref().unwrap_or(&owned_token).split(":").collect::<Vec<_>>();
        let default_host = String::from("https://api.github.com");
        let host = self.host.as_ref().unwrap_or(&default_host);

        // TODO: make main configurable
        match api::get_file(
            host,
            credentials[0],
            credentials[1],
            &self.project_id,
            &crate_path,
            "main",
        )
        .await
        {
            Ok(response) => {
                tracing::info!("'{}' already found! updating!", crate_path);

                if response.encoding != "base64" {
                    //TODO: error
                }

                let new_version: PublishedVersion = request.into();
                let mut versions = read_versions(&response.content);

                if let Some(_) = versions.iter().find(|v| v.vers == new_version.vers) {
                    tracing::info!(
                        "{} version '{}' already existing! Updating!",
                        new_version.name,
                        new_version.vers
                    );
                } else {
                    versions.push(new_version);
                }
                let new_content = versions
                    .iter()
                    .map(|v| serde_json::to_string(v).unwrap())
                    .collect::<Vec<String>>()
                    .join("\n");
                let encoded_content = base64::encode(new_content);

                let update_request = crate::models::update_file::Request {
                    branch: Some(branch_name.clone()),
                    content: encoded_content,
                    message: format!("Adding {} {}", request.meta.name, request.meta.vers),
                    sha: Some(response.sha),
                    ..Default::default()
                };

                let main_branch =
                    api::get_branch(host,credentials[0], credentials[1], &self.project_id, "main")
                        .await?;
                api::create_branch(
                    host,
                    credentials[0],
                    credentials[1],
                    &self.project_id,
                    crate::models::create_branch::Request {
                        r#ref: format!("refs/heads/{}", branch_name),
                        sha: main_branch.commit.sha,
                    },
                )
                .await?;

                match api::update_file(
                    host,
                    credentials[0],
                    credentials[1],
                    &self.project_id,
                    &crate_path,
                    &update_request,
                )
                .await
                {
                    Ok(_) => {}
                    Err(e) => {
                        tracing::error!("failed to udate file - deleting branch");
                        api::delete_branch(
                            host,
                            credentials[0],
                            credentials[1],
                            &self.project_id,
                            &branch_name,
                        )
                        .await?;
                        return Err(e);
                    }
                }
            }
            Err(_) => {
                tracing::info!("'{}' not found! creating!", crate_path);
                let main_branch =
                    api::get_branch(host,credentials[0], credentials[1], &self.project_id, "main")
                        .await?;
                api::create_branch(host,
                    credentials[0],
                    credentials[1],
                    &self.project_id,
                    crate::models::create_branch::Request {
                        r#ref: format!("refs/heads/{}", branch_name),
                        sha: main_branch.commit.sha,
                    },
                )
                .await?;

                let initial_version: PublishedVersion = request.into();
                let json = serde_json::to_string(&initial_version).unwrap();
                let encoded_content = base64::encode(json);
                let create_request = crate::models::update_file::Request {
                    branch: Some(branch_name.clone()),
                    content: encoded_content,
                    message: format!("Adding {} {}", request.meta.name, request.meta.vers),
                    ..Default::default()
                };

                match api::update_file(
                    host,
                    credentials[0],
                    credentials[1],
                    &self.project_id,
                    &crate_path,
                    &create_request,
                )
                .await
                {
                    Ok(_) => {}
                    Err(e) => {
                        tracing::error!("failed to udate file - deleting branch");
                        api::delete_branch(
                            host,
                            credentials[0],
                            credentials[1],
                            &self.project_id,
                            &branch_name,
                        )
                        .await?;
                        return Err(e);
                    }
                }
            }
        };

        tracing::info!("creating PR for branch '{}'!", branch_name);
        let pull_request = models::create_pull_request::Request {
            title: format!("{}-{}", request.meta.name, request.meta.vers),
            head: branch_name.clone(),
            base: "main".into(), // TODO: make configurable
            ..Default::default()
        };

        match api::create_pull_request(
            host,
            credentials[0],
            credentials[1],
            &self.project_id,
            pull_request,
        )
        .await
        {
            Ok(pull_response) => {
                tracing::info!("merging pull request #{}", pull_response.number);
                let merge_request = crate::models::merge_pull_request::Request::default();
                match api::merge_pull_request(
                    host,
                    merge_credentials[0],
                    merge_credentials[1],
                    &self.project_id,
                    pull_response.number,
                    merge_request,
                )
                .await
                {
                    Ok(_) => {
                        api::delete_branch(
                            host,
                            credentials[0],
                            credentials[1],
                            &self.project_id,
                            &branch_name,
                        )
                        .await?;
                        Ok(())
                    }
                    Err(e) => {
                        tracing::error!("failed to merhe MR - deleting branch");
                        api::close_pull_request(
                            host,
                            credentials[0],
                            credentials[1],
                            &self.project_id,
                            pull_response.number,
                        )
                        .await?;
                        api::delete_branch(
                            host,
                            credentials[0],
                            credentials[1],
                            &self.project_id,
                            &branch_name,
                        )
                        .await?;
                        Err(e)
                    }
                }
            }
            Err(e) => {
                tracing::error!("failed to create MR - deleting branch");
                api::delete_branch(
                    host,
                    credentials[0],
                    credentials[1],
                    &self.project_id,
                    &branch_name,
                )
                .await?;
                Err(e)
            }
        }
    }

    async fn yank_crate(
        &self,
        token: &str,
        request: &cargolifter_core::models::YankRequest,
    ) -> Result<(), reqwest::Error> {
        let crate_path = cargolifter_core::get_crate_file_path(&request.name);
        let branch_name = format!(
            "{}-{}-{}",
            if request.yank { "yank" } else { "unyank" },
            request.name,
            request.vers
        );

        let credentials = token.split(":").collect::<Vec<_>>();
        let owned_token = token.to_owned();
        let merge_credentials = self.cargoliter_token.as_ref().unwrap_or(&owned_token).split(":").collect::<Vec<_>>();
        let default_host = String::from("https://api.github.com");
        let host = self.host.as_ref().unwrap_or(&default_host);

        // TODO: make main configurable
        match api::get_file(
            host,
            credentials[0],
            credentials[1],
            &self.project_id,
            &crate_path,
            "main",
        )
        .await
        {
            Ok(response) => {
                if response.encoding != "base64" {
                    //TODO: error
                }

                let mut versions = read_versions(&response.content);
                let mut version_found = false;
                versions.iter_mut().for_each(|v| {
                    if v.name == request.name && v.vers == request.vers && v.yanked != request.yank
                    {
                        version_found = true;
                        v.yanked = request.yank;
                    }
                });

                if !version_found {
                    tracing::warn!(
                        "Crate {} has no version {} or yanked status won't change - skipping yank!",
                        request.name,
                        request.vers
                    );
                    return Ok(());
                }

                tracing::warn!("{:?}", versions);
                let new_content = versions
                    .iter()
                    .map(|v| serde_json::to_string(v).unwrap())
                    .collect::<Vec<String>>()
                    .join("\n");
                let encoded_content = base64::encode(new_content);

                let update_request = crate::models::update_file::Request {
                    branch: Some(branch_name.clone()),
                    content: encoded_content,
                    message: format!(
                        "{} {} {}",
                        if request.yank { "yanking" } else { "unyanking" },
                        request.name,
                        request.vers
                    ),
                    sha: Some(response.sha),
                    ..Default::default()
                };

                let main_branch =
                    api::get_branch(host,credentials[0], credentials[1], &self.project_id, "main")
                        .await?;
                api::create_branch(
                    host,
                    credentials[0],
                    credentials[1],
                    &self.project_id,
                    crate::models::create_branch::Request {
                        r#ref: format!("refs/heads/{}", branch_name),
                        sha: main_branch.commit.sha,
                    },
                )
                .await?;

                match api::update_file(
                    host,
                    credentials[0],
                    credentials[1],
                    &self.project_id,
                    &crate_path,
                    &update_request,
                )
                .await
                {
                    Ok(_) => {}
                    Err(e) => {
                        tracing::error!("failed to udate file - deleting branch");
                        api::delete_branch(
                            host,
                            credentials[0],
                            credentials[1],
                            &self.project_id,
                            &branch_name,
                        )
                        .await?;
                        return Err(e);
                    }
                }
            }
            Err(e) => {
                tracing::error!("crate {} not found", request.name);
                return Err(e);
            }
        }

        tracing::info!("creating PR for branch '{}'!", branch_name);
        let pull_request = models::create_pull_request::Request {
            title: format!(
                "{}-{}-{}",
                if request.yank { "yank" } else { "unyank" },
                request.name,
                request.vers
            ),
            head: branch_name.clone(),
            base: "main".into(), // TODO: make configurable
            ..Default::default()
        };

        match api::create_pull_request(
            host,
            credentials[0],
            credentials[1],
            &self.project_id,
            pull_request,
        )
        .await
        {
            Ok(pull_response) => {
                tracing::info!("merging pull request #{}", pull_response.number);
                let merge_request = crate::models::merge_pull_request::Request::default();
                match api::merge_pull_request(
                    host,
                    merge_credentials[0],
                    merge_credentials[1],
                    &self.project_id,
                    pull_response.number,
                    merge_request,
                )
                .await
                {
                    Ok(_) => {
                        api::delete_branch(
                            host,
                            credentials[0],
                            credentials[1],
                            &self.project_id,
                            &branch_name,
                        )
                        .await?;
                        Ok(())
                    }
                    Err(e) => {
                        tracing::error!("failed to merhe MR - deleting branch");
                        api::close_pull_request(
                            host,
                            credentials[0],
                            credentials[1],
                            &self.project_id,
                            pull_response.number,
                        )
                        .await?;
                        api::delete_branch(
                            host,
                            credentials[0],
                            credentials[1],
                            &self.project_id,
                            &branch_name,
                        )
                        .await?;
                        Err(e)
                    }
                }
            }
            Err(e) => {
                tracing::error!("failed to create MR - deleting branch");
                api::delete_branch(
                    host,
                    credentials[0],
                    credentials[1],
                    &self.project_id,
                    &branch_name,
                )
                .await?;
                Err(e)
            }
        }
    }
}
