pub async fn accept_merge_request(
    host: &str,
    token: &str,
    project_id: &str,
    merge_request_iid: u64,
    request: &crate::models::accept_merge_request::Request,
) -> Result<(), reqwest::Error> {
    let url = format!(
        "{}/api/v1/repos/{}/pulls/{}/merge",
        host, project_id, merge_request_iid
    );
    let client = reqwest::Client::new();
    client
        .post(url)
        .header("Authorization", format!("token {}", token))
        .header("user-agent", "cargolifter")
        .json(request)
        .send()
        .await?
        .error_for_status()?;
    Ok(())
}
