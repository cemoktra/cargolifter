pub async fn execute(
    backend: &impl crate::Backend,
    token: &str,
    request: &crate::models::YankRequest,
) -> Result<(), reqwest::Error> {
    let crate_path = crate::get_crate_file_path(&request.name);
    let branch_name = format!("{}-{}", request.name, request.vers);

    match backend.get_file(token, &crate_path).await {
        Ok((content, encoding, sha)) => {
            let mut versions = super::utils::read_versions(&content, &encoding);
            let mut version_found = false;
            versions.iter_mut().for_each(|v| {
                if v.name == request.name && v.vers == request.vers && v.yanked != request.yank {
                    version_found = true;
                    v.yanked = request.yank;
                }
            });

            if version_found {
                tracing::warn!(
                    "Crate {} has no version {} or yanked status won't change - skipping yank!",
                    request.name,
                    request.vers
                );
                return Ok(());
            }

            if backend
                .update_file(token, &crate_path, &branch_name, &versions, &sha)
                .await
                .is_err()
            {
                let _ = backend.delete_branch(token, &branch_name).await;
            }
        }
        Err(e) => {
            tracing::error!("crate {} not found", request.name);
            return Err(e);
        }
    }

    super::utils::merge_branch(backend, token, &branch_name).await
}
