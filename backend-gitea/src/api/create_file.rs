pub async fn create_file(
    host: &str,
    token: &str,
    project_id: &str,
    file: &str,
    request: &crate::models::create_file::Request,
) -> Result<crate::models::create_file::Response, reqwest::Error> {
    let url = format!(
        "{}/api/v1/repos/{}/contents/{}",
        host,
        project_id,
        file
    );
    let client = reqwest::Client::new();
    client
        .post(url)
        .header("Authorization", format!("token {}", token))
        .header("user-agent", "cargolifter")
        .json(request)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await
}
