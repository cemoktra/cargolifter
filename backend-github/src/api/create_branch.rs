pub async fn create_branch(
    url: &str,
    username: &str,
    token: &str,
    project_id: &str,
    request: crate::models::create_branch::Request,
) -> Result<crate::models::create_branch::Response, reqwest::Error> {
    let url = format!("{}/repos/{}/git/refs", url, project_id);
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
