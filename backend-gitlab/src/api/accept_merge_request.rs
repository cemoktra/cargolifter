pub async fn accept_merge_request(
    token: &str,
    project_id: usize,
    merge_request_iid: i32,
    request: &crate::models::accept_merge_request::Request, 
) -> Result<crate::models::accept_merge_request::Response, reqwest::Error> {
    let url = format!(
        "https://gitlab.com/api/v4/projects/{}/merge_requests/{}/merge",
        project_id,
        merge_request_iid
    );
    let client = reqwest::Client::new();
    client
        .put(url)
        .header("PRIVATE-TOKEN", token)
        .json(request)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await
}