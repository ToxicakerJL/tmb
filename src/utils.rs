use std::env;
use std::path::PathBuf;
use ratatui::prelude::*;

pub fn centered_rect(r: Rect, percent_x: u16, percent_y: u16) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

pub fn get_project_root_path() -> String {
    let project_root = env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".into());
    let project_root_path = PathBuf::from(project_root);
    project_root_path.to_string_lossy().into_owned()
}