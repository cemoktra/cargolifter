pub async fn create_merge_request(
    host: &str,
    token: &str,
    project_id: usize,
    request: &crate::models::create_merge_request::Request,
) -> Result<crate::models::create_merge_request::Response, reqwest::Error> {
    let url = format!("{}/api/v4/projects/{}/merge_requests", host, project_id);
    let client = reqwest::Client::new();
    client
        .post(url)
        .header("PRIVATE-TOKEN", token)
        .json(request)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await
}
