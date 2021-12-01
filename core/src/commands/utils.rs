pub async fn merge_branch(backend: &impl crate::Backend, token: &str, branch_name: &str) -> Result<(), reqwest::Error> {
    tracing::info!("creating pull request for branch '{}'!", branch_name);
    match backend.create_pull_request(token, &branch_name, &branch_name).await {
        Ok(pull_id) => {
            match backend.merge_pull_request(token, pull_id).await {
                Ok(_) => {
                    backend.delete_branch(token, & branch_name).await
                },
                Err(e) => {
                    tracing::error!("failed to create pull request - deleting pull request and branch");
                    let _ = backend.delete_pull_request(token, pull_id).await;
                    let _ = backend.delete_branch(token, & branch_name).await;
                    Err(e)
                }
            }
        },
        Err(e) => {
            tracing::error!("failed to create pull request - deleting branch");
            let _ = backend.delete_branch(token, & branch_name).await;
            Err(e)
        },
    }
}

pub fn read_versions(content: &str, encoding: &str) -> Vec<crate::models::PublishedVersion> {
    let content = content.replace("\n", "");
    let content = if encoding == "base64" {
        let content_bytes = base64::decode(content).unwrap();
        String::from_utf8(content_bytes).unwrap()
    } else {
        content
    };

    content
        .lines()
        .map(|s| serde_json::from_str::<crate::models::PublishedVersion>(s).unwrap())
        .collect()
}