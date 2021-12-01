pub async fn close_pull_request(
    url: &str,
    username: &str,
    token: &str,
    project_id: &str,
    pull_id: u64,
) -> Result<crate::models::close_pull_request::Response, reqwest::Error> {
    let url = format!("{}/repos/{}/pulls/{}", url, project_id, pull_id);
    let client = reqwest::Client::new();
    let request = crate::models::close_pull_request::Request {
        state: "closed".into(),
    };
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
