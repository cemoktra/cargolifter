mod api;
mod models;

use async_trait::async_trait;
use cargolifter_core::models::PublishedVersion;
use cargolifter_core::Backend;

pub struct Github {
    cargoliter_token: Option<String>,
    project_id: String,
    host: Option<String>,
    default_branch: String,
}

impl Github {
    pub fn from(config: cargolifter_core::config::GithubConfig) -> Self {
        Self {
            cargoliter_token: config.cargolifter_token,
            project_id: [config.owner, config.repo].join("/"),
            host: config.host.clone(),
            default_branch: config
                .default_branch
                .unwrap_or_else(|| String::from("main")),
        }
    }

    fn host(&self) -> String {
        let default_host = String::from("https://api.github.com");
        self.host.as_ref().unwrap_or(&default_host).into()
    }

    fn config(&self, token: &str) -> (String, String, String) {
        let credentials = token.split(':').collect::<Vec<_>>();
        (credentials[0].into(), credentials[1].into(), self.host())
    }

    fn merge_config(&self, token: &str) -> (String, String, String) {
        let owned_token = token.to_owned();
        let merge_credentials = self
            .cargoliter_token
            .as_ref()
            .unwrap_or(&owned_token)
            .split(':')
            .collect::<Vec<_>>();
        (merge_credentials[0].into(), merge_credentials[1].into(), self.host())
    }
}

#[async_trait]
impl Backend for Github {
    async fn get_file(
        &self,
        token: &str,
        crate_path: &str,
    ) -> Result<(String, String, String), reqwest::Error> {
        let (username, password, host) = self.config(token);

        match api::get_file(
            &host,
            &username,
            &password,
            &self.project_id,
            &crate_path,
            &self.default_branch,
        )
        .await
        {
            Ok(response) => Ok((
                response.content, 
                response.encoding,
                response.sha,
            )),
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
        let (username, token, host) = self.config(token);

        let main_branch = api::get_branch(
            &host,
            &username,
            &token,
            &self.project_id,
            &self.default_branch,
        )
        .await?;
        api::create_branch(
            &host,
            &username,
            &token,
            &self.project_id,
            crate::models::create_branch::Request {
                r#ref: format!("refs/heads/{}", branch_name),
                sha: main_branch.commit.sha,
            },
        )
        .await?;

        let json = serde_json::to_string(&initial_version).unwrap();
        let encoded_content = base64::encode(json);
        let create_request = crate::models::update_file::Request {
            branch: Some(branch_name.into()),
            content: encoded_content,
            message: format!("Adding {} {}", initial_version.name, initial_version.vers),
            ..Default::default()
        };

        match api::update_file(
            &host,
            &username,
            &token,
            &self.project_id,
            &crate_path,
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
        let (username, token, host) = self.config(token);

        let new_content = versions
            .iter()
            .map(|v| serde_json::to_string(v).unwrap())
            .collect::<Vec<String>>()
            .join("\n");

        let update_request = crate::models::update_file::Request {
            branch: Some(branch_name.into()),
            content: base64::encode(new_content),
            message: format!("Adding {} {}", versions[0].name, versions[0].vers),
            sha: Some(current_sha.into()),
            ..Default::default()
        };

        let main_branch = api::get_branch(
            &host,
            &username,
            &token,
            &self.project_id,
            &self.default_branch,
        )
        .await?;
        api::create_branch(
            &host,
            &username,
            &token,
            &self.project_id,
            crate::models::create_branch::Request {
                r#ref: format!("refs/heads/{}", branch_name),
                sha: main_branch.commit.sha,
            },
        )
        .await?;

        match api::update_file(
            &host,
            &username,
            &token,
            &self.project_id,
            &crate_path,
            &update_request,
        )
        .await
        {
            Ok(_) => Ok(()),
            Err(e) => Err(e),
        }
    }

    async fn delete_branch(
        &self,
        token: &str,
        branch_name: &str,
    ) -> Result<(), reqwest::Error>
    {
        let (username, token, host) = self.config(token);

        match api::delete_branch(
            &host,
            &username,
            &token,
            &self.project_id,
            &branch_name,
        ).await {
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
        let (username, token, host) = self.config(token);

        let pull_request = models::create_pull_request::Request {
            title: title.into(),
            head: branch_name.into(),
            base: self.default_branch.clone(),
            ..Default::default()
        };

        match api::create_pull_request(
            &host,
            &username,
            &token,
            &self.project_id,
            pull_request,
        )
        .await {
            Ok(response) => Ok(response.number),
            Err(e) => Err(e),
        }
    }

    async fn merge_pull_request(
        &self,
        token: &str,
        id: u64,
    ) -> Result<(), reqwest::Error> {
        let (username, token, host) = self.merge_config(token);

        let merge_request = crate::models::merge_pull_request::Request::default();

        match api::merge_pull_request(
            &host,
            &username,
            &token,
            &self.project_id,
            id,
            merge_request,
        ).await {
            Ok(_) => Ok(()),
            Err(e) => Err(e),
        }
    }

    async fn delete_pull_request(
        &self,
        token: &str,
        id: u64,
    ) -> Result<(), reqwest::Error> {
        let (username, token, host) = self.config(token);

        api::close_pull_request(
            &host,
            &username,
            &token,
            &self.project_id,
            id,
        )
        .await?;

        todo!()
    }
}
