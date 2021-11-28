pub async fn create_file(
    token: &str,
    project_id: usize,
    file: &str,
    request: &crate::models::create_file::Request,
) -> Result<crate::models::create_file::Response, reqwest::Error> {
    let url = format!(
        "https://gitlab.com/api/v4/projects/{}/repository/files/{}",
        project_id,
        urlencoding::encode(file)
    );
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
