pub fn get_sub_directories(
    parent_directory: &std::path::Path,
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let entries = std::fs::read_dir(parent_directory)?;
    return Ok(entries
        .filter_map(|x| {
            x.map(|dir| {
                if dir.path().is_dir() {
                    dir.file_name()
                        .into_string()
                        .or::<std::ffi::OsString>(Ok(String::new()))
                        .unwrap()
                } else {
                    return String::new();
                }
            })
            .ok()
        })
        .filter(|x| !x.is_empty())
        .collect());
}
