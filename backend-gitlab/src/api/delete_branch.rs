// DELETE /projects/:id/repository/branches/:branch

pub async fn delete_branch(
    token: &str,
    project_id: usize,
    branch: &str,
) -> Result<(), reqwest::Error> {
    let url = format!(
        "https://gitlab.com/api/v4/projects/{}/repository/branches/{}",
        project_id,
        branch
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
