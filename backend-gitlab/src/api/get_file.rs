pub async fn get_file(
    token: &str,
    project_id: usize,
    file: &str,
) -> Result<crate::models::get_file::Response, reqwest::Error> {
    let url = format!(
        "https://gitlab.com/api/v4/projects/{}/files/{}",
        project_id, file
    );
    let client = reqwest::Client::new();
    client
        .get(url)
        .header("PRIVATE-TOKEN", token)
        .send()
        .await?
        .json::<crate::models::get_file::Response>()
        .await
}
