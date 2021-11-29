pub async fn get_file(
    host: &str,
    token: &str,
    project_id: usize,
    file: &str,
    branch: &str,
) -> Result<crate::models::get_file::Response, reqwest::Error> {
    let url = format!(
        "{}/api/v4/projects/{}/repository/files/{}",
        host,
        project_id,
        urlencoding::encode(file)
    );
    tracing::info!("calling {}", url);
    let client = reqwest::Client::new();
    client
        .get(url)
        .header("PRIVATE-TOKEN", token)
        .header("user-agent", "cargolifter")
        .query(&[("ref", branch)])
        .send()
        .await?
        .error_for_status()?
        .json()
        .await
}
