// DELETE /projects/:id/repository/branches/:branch

pub async fn delete_branch(
    host: &str,
    token: &str,
    project_id: usize,
    branch: &str,
) -> Result<(), reqwest::Error> {
    let url = format!(
        "{}/api/v4/projects/{}/repository/branches/{}",
        host, project_id, branch
    );
    let client = reqwest::Client::new();
    client
        .delete(url)
        .header("PRIVATE-TOKEN", token)
        .send()
        .await?
        .error_for_status()?;
    Ok(())
}
