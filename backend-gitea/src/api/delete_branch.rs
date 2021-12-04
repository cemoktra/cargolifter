// DELETE /projects/:id/repository/branches/:branch

pub async fn delete_branch(
    host: &str,
    token: &str,
    project_id: &str,
    branch: &str,
) -> Result<(), reqwest::Error> {
    let url = format!(
        "{}/api/v1/repos/{}/branches/{}",
        host, project_id, branch
    );
    let client = reqwest::Client::new();
    client
        .delete(url)
        .header("Authorization", format!("token {}", token))
        .header("user-agent", "cargolifter")
        .send()
        .await?
        .error_for_status()?;
    Ok(())
}
