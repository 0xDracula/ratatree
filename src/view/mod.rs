pub mod list;
pub mod tree;

use std::collections::HashSet;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct ListViewState {
    pub cursor: usize,
    pub scroll_offset: usize,
}

impl Default for ListViewState {
    fn default() -> Self {
        Self::new()
    }
}

impl ListViewState {
    pub fn new() -> Self {
        Self {
            cursor: 0,
            scroll_offset: 0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TreeViewState {
    pub cursor: usize,
    pub scroll_offset: usize,
    pub expanded: HashSet<PathBuf>,
}

impl Default for TreeViewState {
    fn default() -> Self {
        Self::new()
    }
}

impl TreeViewState {
    pub fn new() -> Self {
        Self {
            cursor: 0,
            scroll_offset: 0,
            expanded: HashSet::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum ViewState {
    List(ListViewState),
    Tree(TreeViewState),
}

impl ViewState {
    pub fn toggle(self) -> Self {
        match self {
            ViewState::List(_) => ViewState::Tree(TreeViewState::new()),
            ViewState::Tree(_) => ViewState::List(ListViewState::new()),
        }
    }

    pub fn cursor(&self) -> usize {
        match self {
            ViewState::List(s) => s.cursor,
            ViewState::Tree(s) => s.cursor,
        }
    }

    pub fn cursor_mut(&mut self) -> &mut usize {
        match self {
            ViewState::List(s) => &mut s.cursor,
            ViewState::Tree(s) => &mut s.cursor,
        }
    }

    pub fn scroll_offset(&self) -> usize {
        match self {
            ViewState::List(s) => s.scroll_offset,
            ViewState::Tree(s) => s.scroll_offset,
        }
    }

    pub fn scroll_offset_mut(&mut self) -> &mut usize {
        match self {
            ViewState::List(s) => &mut s.scroll_offset,
            ViewState::Tree(s) => &mut s.scroll_offset,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn list_view_state_default() {
        let state = ListViewState::new();
        assert_eq!(state.cursor, 0);
        assert_eq!(state.scroll_offset, 0);
    }

    #[test]
    fn tree_view_state_default() {
        let state = TreeViewState::new();
        assert_eq!(state.cursor, 0);
        assert_eq!(state.scroll_offset, 0);
        assert!(state.expanded.is_empty());
    }

    #[test]
    fn view_state_toggle() {
        let view = ViewState::List(ListViewState::new());
        let toggled = view.toggle();
        assert!(matches!(toggled, ViewState::Tree(_)));
        let back = toggled.toggle();
        assert!(matches!(back, ViewState::List(_)));
    }
}
