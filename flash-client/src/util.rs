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
//If direction is True, then we add the offset to the state. If direction is False, we remove the offset from the state.
pub fn offset_state(
    initial_state: &tui::widgets::ListState,
    offset: usize,
    direction: bool,
    max: usize,
) -> tui::widgets::ListState {
    let mut new_state = initial_state.to_owned();
    new_state.select(
        initial_state
            .selected()
            .zip(Some(offset))
            .map(|(x, y)| {
                if direction {
                    return x.checked_add(y).unwrap_or_default();
                } else {
                    return x.checked_sub(y).unwrap_or_default();
                }
            })
            .map(|x| std::cmp::min(x, max)),
    );
    return new_state;
}
