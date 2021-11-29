/// get the path to store the crate in
pub fn get_crate_path(name: &str) -> String {
    match name.len() {
        1 => "1".into(),
        2 => "2".into(),
        3 => format!("3/{}", name[0..1].to_string()),
        _ => {
            format!("{}/{}", name[0..2].to_string(), name[2..4].to_string())
        }
    }
}

/// get the path and filename to store the crate in
pub fn get_crate_file_path(name: &str) -> String {
    format!("{}/{}", get_crate_path(name), name)
}

// read existing versions
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

// update yank status 
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

// add a version
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


#[cfg(test)]
mod tests {
    #[test]
    fn test_crate_file_path_1() {
        assert_eq!("1/A", super::get_crate_file_path("A"));
    }

    #[test]
    fn test_crate_file_path_2() {
        assert_eq!("2/AB", super::get_crate_file_path("AB"));
    }

    #[test]
    fn test_crate_file_path_3() {
        assert_eq!("3/A/ABC", super::get_crate_file_path("ABC"));
    }

    #[test]
    fn test_crate_file_path_more() {
        assert_eq!("AB/CD/ABCDE", super::get_crate_file_path("ABCDE"));
    }
}
