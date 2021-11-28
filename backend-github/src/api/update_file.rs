pub async fn update_file(
    url: &str,
    username: &str,
    token: &str,
    project_id: &str,
    file: &str,
    request: &crate::models::update_file::Request,
) -> Result<crate::models::update_file::Response, reqwest::Error> {
    let url = format!(
        "{}/repos/{}/contents/{}",
        url, project_id, file
    );
    let client = reqwest::Client::new();
    client
        .put(url)
        .basic_auth(username, Some(token))
        .header("Accept", "application/vnd.github.v3+json.raw")
        .header("user-agent", "cargolifter")
        .json(request)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await
}
