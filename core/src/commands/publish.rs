pub async fn execute(
    backend: &impl crate::Backend,
    token: &str,
    request: &crate::models::PublishRequest,
) -> Result<(), reqwest::Error> {
    let crate_path = crate::get_crate_file_path(&request.meta.name);
    let branch_name = format!("{}-{}", request.meta.name, request.meta.vers);

    match backend.get_file(token, &crate_path).await {
        Ok((content, encoding, sha)) => {
            tracing::info!("'{}' already found! updating!", crate_path);
            let mut versions = super::utils::read_versions(&content, &encoding);

            let new_version: crate::models::PublishedVersion = request.into();
            if versions.iter().any(|v| v.vers == new_version.vers) {
                tracing::warn!(
                    "{} version '{}' already existing! !",
                    new_version.name,
                    new_version.vers
                );
            } else {
                versions.push(new_version);
            }

            if let Err(e) = backend
                .update_file(token, &crate_path, &branch_name, &versions, &sha)
                .await
            {
                tracing::error!(
                    "Failed to update file'{}' - deleting branch if exists!",
                    crate_path
                );
                let _ = backend.delete_branch(token, &branch_name).await;
                return Err(e);
            }
        }
        Err(_) => {
            tracing::info!("'{}' not found! creating!", crate_path);
            let initial_version: crate::models::PublishedVersion = request.into();
            if let Err(e) = backend
                .create_file(token, &crate_path, &branch_name, &initial_version)
                .await
            {
                let _ = backend.delete_branch(token, &branch_name).await;
                return Err(e);
            }
        }
    }

    super::utils::merge_branch(backend, token, &branch_name).await
}
