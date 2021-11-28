pub async fn merge_pull_request(
    url: &str,
    username: &str,
    token: &str,
    project_id: &str,
    pull_id: i64,
    request: crate::models::merge_pull_request::Request,
) -> Result<crate::models::merge_pull_request::Response, reqwest::Error> {
    let url = format!("{}/repos/{}/pulls/{}/merge", url, project_id, pull_id);
    let client = reqwest::Client::new();
    client
        .put(url)
        .basic_auth(username, Some(token))
        .header("Accept", "application/vnd.github.v3+json")
        .header("user-agent", "cargolifter")
        .json(&request)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await
}
