mod api;
mod models;

use async_trait::async_trait;
use cargolifter_core::models::PublishedVersion;
use cargolifter_core::Backend;

pub struct Gitea {
    cargolifter_token: Option<String>,
    project_id: String,
    host: String,
    default_branch: String,
}

impl Gitea {
    pub fn from(config: cargolifter_core::config::GiteaConfig) -> Self {
        Self {
            cargolifter_token: config.cargolifter_token,
            project_id: [config.owner, config.repo].join("/"),
            host: config.host.clone(),
            default_branch: config
                .default_branch
                .unwrap_or_else(|| String::from("main")),
        }
    }
}

#[async_trait]
impl Backend for Gitea {
    async fn get_file(
        &self,
        token: &str,
        crate_path: &str,
    ) -> Result<(String, String, String), reqwest::Error> {
        match api::get_file(
            &self.host,
            token,
            &self.project_id,
            crate_path,
            &self.default_branch,
        )
        .await
        {
            Ok(response) => Ok((response.content, response.encoding, response.sha)),
            Err(e) => Err(e),
        }
    }

    async fn create_file(
        &self,
        token: &str,
        crate_path: &str,
        branch_name: &str,
        initial_version: &PublishedVersion,
    ) -> Result<(), reqwest::Error> {
        let json = serde_json::to_string(&initial_version).unwrap();
        let encoded_content = base64::encode(json);
        let create_request = crate::models::create_file::Request {
            branch: Some(self.default_branch.clone()),
            new_branch: Some(branch_name.into()),
            content: encoded_content,
            message: Some(format!(
                "Adding {} {}",
                initial_version.name, initial_version.vers
            )),
        };

        match api::create_file(
            &self.host,
            token,
            &self.project_id,
            crate_path,
            &create_request,
        )
        .await
        {
            Ok(_) => Ok(()),
            Err(e) => Err(e),
        }
    }

    async fn update_file(
        &self,
        token: &str,
        crate_path: &str,
        branch_name: &str,
        versions: &[PublishedVersion],
        current_sha: &str,
    ) -> Result<(), reqwest::Error> {
        let new_content = versions
            .iter()
            .map(|v| serde_json::to_string(v).unwrap())
            .collect::<Vec<String>>()
            .join("\n");

        let update_request = crate::models::update_file::Request {
            new_branch: Some(branch_name.into()),
            branch: Some(self.default_branch.clone()),
            content: base64::encode(new_content),
            message: Some(format!("Adding {} {}", versions[0].name, versions[0].vers)),
            sha: current_sha.into(),
        };
        match api::update_file(
            &self.host,
            token,
            &self.project_id,
            crate_path,
            &update_request,
        )
        .await
        {
            Ok(_) => Ok(()),
            Err(e) => Err(e),
        }
    }

    async fn delete_branch(&self, token: &str, branch_name: &str) -> Result<(), reqwest::Error> {
        match api::delete_branch(&self.host, token, &self.project_id, branch_name).await {
            Ok(_) => Ok(()),
            Err(e) => Err(e),
        }
    }

    async fn create_pull_request(
        &self,
        token: &str,
        title: &str,
        branch_name: &str,
    ) -> Result<u64, reqwest::Error> {
        let merge_request = models::create_merge_request::Request {
            title: title.into(),
            head: branch_name.into(),
            base: self.default_branch.clone(),
        };
        match api::create_merge_request(&self.host, token, &self.project_id, &merge_request).await {
            Ok(response) => Ok(response.number),
            Err(e) => Err(e),
        }
    }

    async fn merge_pull_request(&self, token: &str, id: u64) -> Result<(), reqwest::Error> {
        let owned_token = token.to_owned();
        let merge_token = self.cargolifter_token.as_ref().unwrap_or(&owned_token);

        let accept_request = models::accept_merge_request::Request {
            r#do: "merge".into(),
            delete_branch_after_merge: Some(false),
        };

        match api::accept_merge_request(
            &self.host,
            merge_token,
            &self.project_id,
            id,
            &accept_request,
        )
        .await
        {
            Ok(_) => Ok(()),
            Err(e) => Err(e),
        }
    }

    async fn delete_pull_request(&self, _token: &str, _id: u64) -> Result<(), reqwest::Error> {
        Ok(())
    }
}
