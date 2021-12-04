pub async fn create_merge_request(
    host: &str,
    token: &str,
    project_id: &str,
    request: &crate::models::create_merge_request::Request,
) -> Result<crate::models::create_merge_request::Response, reqwest::Error> {
    let url = format!("{}/api/v1/repos/{}/pulls", host, project_id);
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
