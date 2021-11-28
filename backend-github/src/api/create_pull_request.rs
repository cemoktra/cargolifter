pub async fn create_pull_request(
    url: &str,
    username: &str,
    token: &str,
    project_id: &str,
    request: crate::models::create_pull_request::Request,
) -> Result<crate::models::create_pull_request::Response, reqwest::Error> {
    let url = format!("{}/repos/{}/pulls", url, project_id);
    let client = reqwest::Client::new();
    client
        .post(url)
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
