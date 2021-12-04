mod api;
mod models;

use async_trait::async_trait;
use cargolifter_core::models::PublishedVersion;
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
            default_branch: config
                .default_branch
                .unwrap_or_else(|| String::from("main")),
        }
    }

    fn host(&self) -> String {
        let default_host = String::from("https://gitlab.com");
        self.host.as_ref().unwrap_or(&default_host).into()
    }
}

#[async_trait]
impl Backend for Gitlab {
    async fn get_file(
        &self,
        token: &str,
        crate_path: &str,
    ) -> Result<(String, String, String), reqwest::Error> {
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
            Ok(response) => Ok((response.content, response.encoding, response.content_sha256)),
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
        let host = self.host();

        let json = serde_json::to_string(&initial_version).unwrap();
        let encoded_content = base64::encode(json);
        let create_request = crate::models::create_file::Request {
            branch: branch_name.into(),
            start_branch: Some(self.default_branch.clone()),
            content: encoded_content,
            encoding: Some("base64".into()),
            commit_message: format!("Adding {} {}", initial_version.name, initial_version.vers),
            ..Default::default()
        };

        match api::create_file(&host, token, self.project_id, &crate_path, &create_request).await {
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
        _current_sha: &str,
    ) -> Result<(), reqwest::Error> {
        let host = self.host();

        let new_content = versions
            .iter()
            .map(|v| serde_json::to_string(v).unwrap())
            .collect::<Vec<String>>()
            .join("\n");

        let update_request = crate::models::update_file::Request {
            branch: branch_name.into(),
            start_branch: Some(self.default_branch.clone()),
            content: base64::encode(new_content),
            encoding: Some("base64".into()),
            commit_message: format!("Adding {} {}", versions[0].name, versions[0].vers),
            ..Default::default()
        };
        match api::update_file(&host, token, self.project_id, &crate_path, &update_request).await {
            Ok(_) => Ok(()),
            Err(e) => Err(e),
        }
    }

    async fn delete_branch(&self, token: &str, branch_name: &str) -> Result<(), reqwest::Error> {
        let host = self.host();

        match api::delete_branch(&host, token, self.project_id, &branch_name.clone()).await {
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
        let host = self.host();

        let merge_request = models::create_merge_request::Request {
            source_branch: branch_name.into(),
            target_branch: self.default_branch.clone(),
            title: title.into(),
            remove_source_branch: Some(false),
            ..Default::default()
        };
        match api::create_merge_request(&host, token, self.project_id, &merge_request).await {
            Ok(response) => Ok(response.iid),
            Err(e) => Err(e),
        }
    }

    async fn merge_pull_request(&self, token: &str, id: u64) -> Result<(), reqwest::Error> {
        let host = self.host();

        let owned_token = token.to_owned();
        let merge_token = self.cargolifter_token.as_ref().unwrap_or(&owned_token);

        let accept_request = models::accept_merge_request::Request {
            should_remove_source_branch: Some(false),
            ..Default::default()
        };

        match api::accept_merge_request(&host, merge_token, self.project_id, id, &accept_request)
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
