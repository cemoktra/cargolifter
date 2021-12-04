pub async fn get_file(
    host: &str,
    token: &str,
    project_id: &str,
    file: &str,
    branch: &str,
) -> Result<crate::models::get_file::Response, reqwest::Error> {
    let url = format!("{}/api/v1/repos/{}/contents/{}", host, project_id, file);
    let client = reqwest::Client::new();
    client
        .get(url)
        .header("Authorization", format!("token {}", token))
        .header("user-agent", "cargolifter")
        .query(&[("ref", branch)])
        .send()
        .await?
        .error_for_status()?
        .json()
        .await
}
