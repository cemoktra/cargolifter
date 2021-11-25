pub async fn update_file(
    token: &str,
    project_id: usize,
    file: &str,
    request: &crate::models::update_file::Request,
) -> Result<crate::models::update_file::Response, reqwest::Error> {
    let url = format!(
        "https://gitlab.com/api/v4/projects/{}/files/{}",
        project_id, file
    );
    let client = reqwest::Client::new();
    client
        .put(url)
        .header("PRIVATE-TOKEN", token)
        .json(request)
        .send()
        .await?
        .error_for_status()?
        .json::<crate::models::update_file::Response>()
        .await
}
