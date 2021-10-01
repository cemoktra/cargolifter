pub fn crate_name_to_path(crate_name: &str) -> String {
    match crate_name.len() {
        1 => "1".into(),
        2 => "2".into(),
        3 => format!("3/{}", crate_name[0..1].to_string()),
        _ => {
            format!(
                "{}/{}",
                crate_name[0..2].to_string(),
                crate_name[2..4].to_string()
            )
        }
    }
}
