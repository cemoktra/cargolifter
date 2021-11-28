pub async fn get_file(
    url: &str,
    username: &str,
    token: &str,
    project_id: &str,
    file: &str,
    branch: &str,
) -> Result<crate::models::get_file::Response, reqwest::Error> {
    let url = format!("{}/repos/{}/contents/{}", url, project_id, file);
    let client = reqwest::Client::new();
    client
        .get(url)
        .basic_auth(username, Some(token))
        .header("Accept", "application/vnd.github.v3+json")
        .header("user-agent", "cargolifter")
        .query(&[("ref", branch)])
        .send()
        .await?
        .error_for_status()?
        .json()
        .await
}
