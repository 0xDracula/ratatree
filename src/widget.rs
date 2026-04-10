use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Paragraph, StatefulWidget, Widget};

use crate::entry::EntryKind;
use crate::state::{FilePickerState, InputMode};
use crate::view::ViewState;

#[derive(Default)]
pub struct FilePicker {
    block: Option<Block<'static>>,
}

impl FilePicker {
    pub fn block(mut self, block: Block<'static>) -> Self {
        self.block = Some(block);
        self
    }
}

impl StatefulWidget for FilePicker {
    type State = FilePickerState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        // Apply outer block if provided, get inner area
        let inner = if let Some(block) = self.block {
            let inner = block.inner(area);
            block.render(area, buf);
            inner
        } else {
            area
        };

        // Need at least 3 rows: path bar (1) + list (>=1) + status bar (1)
        if inner.height < 3 {
            return;
        }

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),
                Constraint::Min(1),
                Constraint::Length(1),
            ])
            .split(inner);

        render_path_bar(chunks[0], buf, state);
        render_file_list(chunks[1], buf, state);
        render_status_bar(chunks[2], buf, state);
    }
}

fn render_path_bar(area: Rect, buf: &mut Buffer, state: &FilePickerState) {
    let path_str = state.common.current_dir.to_string_lossy().to_string();
    let style = state.common.theme.path_bar;
    let para = Paragraph::new(Line::from(Span::styled(path_str, style)));
    para.render(area, buf);
}

fn render_file_list(area: Rect, buf: &mut Buffer, state: &mut FilePickerState) {
    let entries = state.visible_entries();

    if entries.is_empty() {
        let style = state.common.theme.status_bar;
        let para = Paragraph::new(Line::from(Span::styled("(empty)", style)));
        para.render(area, buf);
        return;
    }

    let visible_height = area.height as usize;
    let cursor = state.view.cursor();

    // Update scroll offset so cursor stays visible
    {
        let scroll = state.view.scroll_offset_mut();
        if cursor < *scroll {
            *scroll = cursor;
        } else if cursor >= *scroll + visible_height {
            *scroll = cursor + 1 - visible_height;
        }
    }

    let scroll_offset = state.view.scroll_offset();

    let entries = state.visible_entries(); // re-borrow after mut borrow ends

    let theme = &state.common.theme;
    let selected_paths = &state.common.selected;

    for (row, entry) in entries.iter().enumerate().skip(scroll_offset).take(visible_height) {
        let y = area.y + (row - scroll_offset) as u16;
        if y >= area.y + area.height {
            break;
        }

        let is_cursor = row == cursor;
        let is_selected = selected_paths.contains(&entry.path);

        // Prefix
        let prefix = if is_selected { " * " } else { "   " };

        // Name style based on kind
        let name_style = match entry.kind {
            EntryKind::Directory => theme.directory,
            EntryKind::Symlink => theme.symlink,
            EntryKind::File => theme.normal,
        };

        // Apply cursor or selected overlay
        let effective_name_style = if is_cursor {
            name_style.patch(theme.cursor)
        } else if is_selected {
            name_style.patch(theme.selected)
        } else {
            name_style
        };

        let prefix_style = if is_cursor {
            theme.cursor
        } else if is_selected {
            theme.selected
        } else {
            Style::default()
        };

        // Suffix
        let suffix = match entry.kind {
            EntryKind::Directory => "/",
            EntryKind::Symlink => " ->",
            EntryKind::File => "",
        };

        // Render the line into buffer manually for correct styling
        let x = area.x;
        let width = area.width as usize;

        let mut col = 0usize;

        // Prefix
        for ch in prefix.chars() {
            if col >= width {
                break;
            }
            let cell = buf.cell_mut((x + col as u16, y));
            if let Some(cell) = cell {
                cell.set_char(ch);
                cell.set_style(prefix_style);
            }
            col += 1;
        }

        // Name
        for ch in entry.name.chars() {
            if col >= width {
                break;
            }
            let cell = buf.cell_mut((x + col as u16, y));
            if let Some(cell) = cell {
                cell.set_char(ch);
                cell.set_style(effective_name_style);
            }
            col += 1;
        }

        // Suffix
        for ch in suffix.chars() {
            if col >= width {
                break;
            }
            let cell = buf.cell_mut((x + col as u16, y));
            if let Some(cell) = cell {
                cell.set_char(ch);
                cell.set_style(effective_name_style);
            }
            col += 1;
        }

        // Fill remaining width with cursor background if at cursor position
        if is_cursor {
            let bg = theme.cursor.bg.unwrap_or(ratatui::style::Color::Reset);
            while col < width {
                let cell = buf.cell_mut((x + col as u16, y));
                if let Some(cell) = cell {
                    cell.set_char(' ');
                    cell.set_style(Style::default().bg(bg));
                }
                col += 1;
            }
        }
    }
}

fn render_status_bar(area: Rect, buf: &mut Buffer, state: &FilePickerState) {
    let theme = &state.common.theme;

    // Error message takes priority
    if let Some(err) = &state.common.error_message {
        let para = Paragraph::new(Line::from(Span::styled(err.clone(), theme.error)));
        para.render(area, buf);
        return;
    }

    match state.common.input_mode {
        InputMode::Search => {
            let query = &state.common.search_query;
            let count = state.visible_count();
            let text = format!("/ {}  ({} matches)", query, count);
            let para = Paragraph::new(Line::from(Span::styled(text, theme.search_input)));
            para.render(area, buf);
        }
        InputMode::Normal => {
            let selected_count = state.common.selected.len();
            let hidden_str = if state.common.show_hidden { "on" } else { "off" };
            let view_str = match &state.view {
                ViewState::List(_) => "list",
                ViewState::Tree(_) => "tree",
            };
            let text = format!(
                "{} selected | hidden: {} | view: {}",
                selected_count, hidden_str, view_str
            );
            let para = Paragraph::new(Line::from(Span::styled(text, theme.status_bar)));
            para.render(area, buf);
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::backend::TestBackend;
    use ratatui::widgets::Borders;
    use ratatui::Terminal;
    use std::fs;
    use tempfile::TempDir;

    fn make_dir_with_files() -> TempDir {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("alpha.txt"), b"").unwrap();
        fs::write(dir.path().join("beta.rs"), b"").unwrap();
        fs::create_dir(dir.path().join("subdir")).unwrap();
        dir
    }

    #[test]
    fn renders_without_panic() {
        let dir = make_dir_with_files();
        let mut state = FilePickerState::builder()
            .start_dir(dir.path())
            .build();

        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal.draw(|frame| {
            let widget = FilePicker::default().block(Block::default().borders(Borders::ALL));
            frame.render_stateful_widget(widget, frame.area(), &mut state);
        }).unwrap();
    }

    #[test]
    fn renders_empty_directory() {
        let dir = TempDir::new().unwrap();
        let mut state = FilePickerState::builder()
            .start_dir(dir.path())
            .build();

        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal.draw(|frame| {
            let widget = FilePicker::default();
            frame.render_stateful_widget(widget, frame.area(), &mut state);
        }).unwrap();
    }

    #[test]
    fn renders_with_selection() {
        let dir = make_dir_with_files();
        let mut state = FilePickerState::builder()
            .start_dir(dir.path())
            .build();

        // Find a file entry and select it
        let file_idx = state
            .common
            .entries
            .iter()
            .position(|e| e.kind == EntryKind::File)
            .expect("should have a file entry");
        *state.view.cursor_mut() = file_idx;
        state.toggle_select();

        assert_eq!(state.common.selected.len(), 1);

        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal.draw(|frame| {
            let widget = FilePicker::default();
            frame.render_stateful_widget(widget, frame.area(), &mut state);
        }).unwrap();
    }
}
