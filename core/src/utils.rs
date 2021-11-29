pub fn read_versions(content: &str, encoding: &str) -> Vec<crate::models::PublishedVersion> {
    let content = content.replace("\n", "");
    let content = if encoding == "base64" {
        let content_bytes = base64::decode(content).unwrap();
        String::from_utf8(content_bytes).unwrap()
    } else {
        content
    };

    content
        .lines()
        .map(|s| serde_json::from_str::<crate::models::PublishedVersion>(s).unwrap())
        .collect()
}

pub fn updated_yanked(content: &str, encoding: &str, name: &str, vers: &str, yanked: bool) -> Option<String> {
    let mut versions = read_versions(content, encoding);
    let mut version_found = false;
    versions.iter_mut().for_each(|v| {
        if v.name == name && v.vers == vers && v.yanked != yanked
        {
            version_found = true;
            v.yanked = yanked;
        }
    });

    if !version_found {
        tracing::warn!(
            "Crate {} has no version {} or yanked status won't change - skipping yank!",
            name,
            vers
        );
        return None;
    }

    let new_content = versions
        .iter()
        .map(|v| serde_json::to_string(v).unwrap())
        .collect::<Vec<String>>()
        .join("\n");

    Some(base64::encode(new_content))
}

pub fn add_version(new_version: crate::models::PublishedVersion, content: &str, encoding: &str) -> String {
    let mut versions = read_versions(content, encoding);

    if versions.iter().any(|v| v.vers == new_version.vers) {
        tracing::info!(
            "{} version '{}' already existing! Updating!",
            new_version.name,
            new_version.vers
        );
    } else {
        versions.push(new_version);
    }
    let new_content = versions
        .iter()
        .map(|v| serde_json::to_string(v).unwrap())
        .collect::<Vec<String>>()
        .join("\n");
    base64::encode(new_content)
}