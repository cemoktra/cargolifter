// DELETE /projects/:id/repository/branches/:branch

pub async fn get_branch(
    url: &str,
    username: &str,
    token: &str,
    project_id: &str,
    branch: &str,
) -> Result<crate::models::get_branch::Response, reqwest::Error> {
    let url = format!("{}/repos/{}/branches/{}", url, project_id, branch);
    let client = reqwest::Client::new();
    client
        .get(url)
        .basic_auth(username, Some(token))
        .header("Accept", "application/vnd.github.v3+json")
        .header("user-agent", "cargolifter")
        .send()
        .await?
        .error_for_status()?
        .json()
        .await
}
