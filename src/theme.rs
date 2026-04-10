use ratatui::style::{Color, Modifier, Style};

#[derive(Debug, Clone)]
pub struct FilePickerTheme {
    pub normal: Style,
    pub cursor: Style,
    pub selected: Style,
    pub directory: Style,
    pub symlink: Style,
    pub path_bar: Style,
    pub status_bar: Style,
    pub search_input: Style,
    pub error: Style,
}

impl Default for FilePickerTheme {
    fn default() -> Self {
        Self {
            normal: Style::default(),
            cursor: Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD),
            selected: Style::default().fg(Color::Green),
            directory: Style::default().fg(Color::Blue).add_modifier(Modifier::BOLD),
            symlink: Style::default().fg(Color::Cyan),
            path_bar: Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
            status_bar: Style::default().fg(Color::DarkGray),
            search_input: Style::default().fg(Color::Yellow),
            error: Style::default().fg(Color::Red),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_theme_has_distinct_styles() {
        let theme = FilePickerTheme::default();
        assert_ne!(theme.cursor, Style::default());
        assert_ne!(theme.selected, Style::default());
    }
}
