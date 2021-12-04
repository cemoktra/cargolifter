pub async fn execute(
    backend: &impl crate::Backend,
    token: &str,
    crate_name: &str,
    crate_version: &str,
) -> Result<bool, reqwest::Error> {
    let crate_path = crate::get_crate_file_path(crate_name);

    match backend.get_file(token, &crate_path).await {
        Ok((content, encoding, _)) => {
            let versions = super::utils::read_versions(&content, &encoding);
            Ok(versions
                .iter()
                .any(|v| v.name == crate_name && v.vers == crate_version))
        }
        Err(e) => {
            tracing::info!("crate {} not found => not published", crate_name);
            Err(e)
        }
    }
}
