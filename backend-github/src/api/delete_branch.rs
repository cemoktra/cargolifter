pub async fn delete_branch(
    url: &str,
    username: &str,
    token: &str,
    project_id: &str,
    branch: &str,
) -> Result<String, reqwest::Error> {
    let url = format!("{}/repos/{}/git/refs/heads/{}", url, project_id, branch);
    let client = reqwest::Client::new();
    client
        .delete(url)
        .basic_auth(username, Some(token))
        .header("Accept", "application/vnd.github.v3+json")
        .header("user-agent", "cargolifter")
        .send()
        .await?
        .error_for_status()?
        .text()
        .await
    // Ok(())
}
