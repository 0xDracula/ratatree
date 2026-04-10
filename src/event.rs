use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
use std::time::{Duration, Instant};

use crate::search::fuzzy_score;
use crate::state::{FilePickerState, InputMode, PickerResult};

/// Main event dispatcher. Does nothing if the picker has already finished.
pub fn handle_event(state: &mut FilePickerState, event: Event) {
    if state.common.result != PickerResult::Pending {
        return;
    }

    match event {
        Event::Key(key) => handle_key(state, key),
        Event::Mouse(mouse) => handle_mouse(state, mouse),
        _ => {}
    }
}

fn handle_key(state: &mut FilePickerState, key: KeyEvent) {
    match state.common.input_mode {
        InputMode::Normal => handle_normal_key(state, key),
        InputMode::Search => handle_search_key(state, key),
    }
}

fn handle_normal_key(state: &mut FilePickerState, key: KeyEvent) {
    // gg sequence: check if pending_key is 'g' and not expired (500ms)
    if let Some((pending_char, pending_time)) = state.common.pending_key.take() {
        if pending_char == 'g'
            && pending_time.elapsed() < Duration::from_millis(500)
            && key.code == KeyCode::Char('g')
        {
            state.move_to_top();
            return;
        }
        // Pending key expired or different key - fall through with pending_key cleared (already taken)
    }

    match key.code {
        KeyCode::Char('j') | KeyCode::Down => {
            state.move_cursor_down();
        }
        KeyCode::Char('k') | KeyCode::Up => {
            state.move_cursor_up();
        }
        KeyCode::Char('l') | KeyCode::Right => {
            state.enter_directory();
        }
        KeyCode::Char('h') | KeyCode::Left | KeyCode::Backspace => {
            state.go_parent();
        }
        KeyCode::Char('G') => {
            state.move_to_bottom();
        }
        KeyCode::Char('g') => {
            state.common.pending_key = Some(('g', Instant::now()));
        }
        KeyCode::Char('d') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            state.move_half_page_down(20);
        }
        KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            state.move_half_page_up(20);
        }
        KeyCode::Char(' ') => {
            state.toggle_select();
        }
        KeyCode::Enter => {
            state.confirm();
        }
        KeyCode::Esc | KeyCode::Char('q') => {
            state.cancel();
        }
        KeyCode::Tab => {
            state.toggle_view();
        }
        KeyCode::Char('.') => {
            state.toggle_hidden();
        }
        KeyCode::Char('/') => {
            state.common.input_mode = InputMode::Search;
            state.common.search_query.clear();
        }
        KeyCode::Char('f') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            state.common.input_mode = InputMode::Search;
            state.common.search_query.clear();
        }
        KeyCode::Char('~') => {
            state.go_home();
        }
        _ => {}
    }
}

fn handle_search_key(state: &mut FilePickerState, key: KeyEvent) {
    match key.code {
        KeyCode::Esc => {
            state.common.input_mode = InputMode::Normal;
            state.common.search_query.clear();
            state.common.filtered_indices = None;
            state.clamp_cursor_pub();
        }
        KeyCode::Enter => {
            // Keep filter active, just return to Normal mode
            state.common.input_mode = InputMode::Normal;
        }
        KeyCode::Backspace => {
            state.common.search_query.pop();
            update_search_filter(state);
        }
        KeyCode::Char('j') | KeyCode::Down => {
            state.move_cursor_down();
        }
        KeyCode::Char('k') | KeyCode::Up => {
            state.move_cursor_up();
        }
        KeyCode::Char(c) if !key.modifiers.contains(KeyModifiers::CONTROL) => {
            state.common.search_query.push(c);
            update_search_filter(state);
        }
        _ => {}
    }
}

fn update_search_filter(state: &mut FilePickerState) {
    let query = state.common.search_query.clone();
    if query.is_empty() {
        state.common.filtered_indices = None;
    } else {
        let mut scored: Vec<(usize, i32)> = state
            .common
            .entries
            .iter()
            .enumerate()
            .filter_map(|(i, entry)| {
                fuzzy_score(&entry.name, &query).map(|score| (i, score))
            })
            .collect();
        scored.sort_by(|a, b| b.1.cmp(&a.1));
        let indices: Vec<usize> = scored.into_iter().map(|(i, _)| i).collect();
        state.common.filtered_indices = Some(indices);
    }
    *state.view.cursor_mut() = 0;
    *state.view.scroll_offset_mut() = 0;
}

fn handle_mouse(state: &mut FilePickerState, mouse: MouseEvent) {
    match mouse.kind {
        MouseEventKind::Down(MouseButton::Left) => {
            // Offset by 2 rows for path bar at top
            let row = mouse.row as usize;
            if row >= 2 {
                let entry_idx = row - 2;
                let count = state.visible_count();
                if entry_idx < count {
                    *state.view.cursor_mut() = entry_idx;
                }
            }
        }
        MouseEventKind::ScrollDown => {
            state.move_cursor_down();
        }
        MouseEventKind::ScrollUp => {
            state.move_cursor_up();
        }
        _ => {}
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    use std::fs;
    use tempfile::TempDir;

    fn make_state() -> (TempDir, FilePickerState) {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("alpha.txt"), b"").unwrap();
        fs::write(dir.path().join("beta.rs"), b"").unwrap();
        fs::create_dir(dir.path().join("subdir")).unwrap();
        let state = FilePickerState::builder()
            .start_dir(dir.path())
            .build();
        (dir, state)
    }

    fn key(code: KeyCode) -> Event {
        Event::Key(KeyEvent::new(code, KeyModifiers::NONE))
    }

    fn shift_key(code: KeyCode) -> Event {
        Event::Key(KeyEvent::new(code, KeyModifiers::SHIFT))
    }

    #[test]
    fn j_moves_down() {
        let (_dir, mut state) = make_state();
        let initial = state.view.cursor();
        handle_event(&mut state, key(KeyCode::Char('j')));
        assert_eq!(state.view.cursor(), initial + 1);
    }

    #[test]
    fn k_moves_up() {
        let (_dir, mut state) = make_state();
        // Move down first
        handle_event(&mut state, key(KeyCode::Char('j')));
        let after_down = state.view.cursor();
        handle_event(&mut state, key(KeyCode::Char('k')));
        assert_eq!(state.view.cursor(), after_down - 1);
    }

    #[test]
    fn arrow_keys_navigate() {
        let (_dir, mut state) = make_state();
        let initial = state.view.cursor();
        handle_event(&mut state, key(KeyCode::Down));
        assert_eq!(state.view.cursor(), initial + 1);
        handle_event(&mut state, key(KeyCode::Up));
        assert_eq!(state.view.cursor(), initial);
    }

    #[test]
    fn shift_g_moves_to_bottom() {
        let (_dir, mut state) = make_state();
        let count = state.visible_count();
        // 'G' is sent as Shift+g
        handle_event(&mut state, shift_key(KeyCode::Char('G')));
        assert_eq!(state.view.cursor(), count - 1);
    }

    #[test]
    fn space_toggles_selection() {
        let (_dir, mut state) = make_state();
        // Find a file entry (files come after directories)
        let file_idx = state
            .common
            .entries
            .iter()
            .position(|e| e.kind == crate::entry::EntryKind::File)
            .expect("should have a file");
        *state.view.cursor_mut() = file_idx;

        assert!(state.common.selected.is_empty());
        handle_event(&mut state, key(KeyCode::Char(' ')));
        assert_eq!(state.common.selected.len(), 1);
        handle_event(&mut state, key(KeyCode::Char(' ')));
        assert!(state.common.selected.is_empty());
    }

    #[test]
    fn dot_toggles_hidden() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("visible.txt"), b"").unwrap();
        fs::write(dir.path().join(".hidden.txt"), b"").unwrap();
        let mut state = FilePickerState::builder()
            .start_dir(dir.path())
            .build();
        assert_eq!(state.visible_count(), 1);
        handle_event(&mut state, key(KeyCode::Char('.')));
        assert_eq!(state.visible_count(), 2);
    }

    #[test]
    fn esc_cancels() {
        let (_dir, mut state) = make_state();
        assert_eq!(state.common.result, PickerResult::Pending);
        handle_event(&mut state, key(KeyCode::Esc));
        assert_eq!(state.common.result, PickerResult::Cancelled);
    }

    #[test]
    fn q_cancels() {
        let (_dir, mut state) = make_state();
        handle_event(&mut state, key(KeyCode::Char('q')));
        assert_eq!(state.common.result, PickerResult::Cancelled);
    }

    #[test]
    fn tab_toggles_view() {
        let (_dir, mut state) = make_state();
        assert!(matches!(state.view, crate::view::ViewState::List(_)));
        handle_event(&mut state, key(KeyCode::Tab));
        assert!(matches!(state.view, crate::view::ViewState::Tree(_)));
        handle_event(&mut state, key(KeyCode::Tab));
        assert!(matches!(state.view, crate::view::ViewState::List(_)));
    }

    #[test]
    fn slash_enters_search_mode() {
        let (_dir, mut state) = make_state();
        assert_eq!(state.common.input_mode, InputMode::Normal);
        handle_event(&mut state, key(KeyCode::Char('/')));
        assert_eq!(state.common.input_mode, InputMode::Search);
        assert!(state.common.search_query.is_empty());
    }

    #[test]
    fn search_mode_typing_and_esc_clears() {
        let (_dir, mut state) = make_state();
        // Enter search mode
        handle_event(&mut state, key(KeyCode::Char('/')));
        // Type a character
        handle_event(&mut state, key(KeyCode::Char('a')));
        assert_eq!(state.common.search_query, "a");
        assert!(state.common.filtered_indices.is_some());
        // Esc clears query and filter
        handle_event(&mut state, key(KeyCode::Esc));
        assert_eq!(state.common.input_mode, InputMode::Normal);
        assert!(state.common.search_query.is_empty());
        assert!(state.common.filtered_indices.is_none());
    }

    #[test]
    fn search_mode_enter_keeps_filter() {
        let (_dir, mut state) = make_state();
        // Enter search mode and type a query
        handle_event(&mut state, key(KeyCode::Char('/')));
        handle_event(&mut state, key(KeyCode::Char('a')));
        let filter_before = state.common.filtered_indices.clone();
        assert!(filter_before.is_some());
        // Press Enter - should exit search but keep filter
        handle_event(&mut state, key(KeyCode::Enter));
        assert_eq!(state.common.input_mode, InputMode::Normal);
        assert_eq!(state.common.filtered_indices, filter_before);
    }

    #[test]
    fn gg_sequence_moves_to_top() {
        let (_dir, mut state) = make_state();
        // Move to bottom first
        let count = state.visible_count();
        *state.view.cursor_mut() = count - 1;
        // Press g once - sets pending key
        handle_event(&mut state, key(KeyCode::Char('g')));
        assert!(state.common.pending_key.is_some());
        // Press g again quickly - should move to top
        handle_event(&mut state, key(KeyCode::Char('g')));
        assert_eq!(state.view.cursor(), 0);
        assert!(state.common.pending_key.is_none());
    }
}
