// Tree view rendering - implemented in Task 9

use std::path::{Path, PathBuf};

use crate::entry::{read_entries, Entry, EntryKind};
use super::TreeViewState;

#[derive(Debug, Clone)]
pub struct TreeEntry {
    pub entry: Entry,
    pub depth: usize,
}

impl TreeViewState {
    pub fn toggle_expand(&mut self, path: &Path) {
        if !self.expanded.remove(path) {
            self.expanded.insert(path.to_path_buf());
        }
    }

    pub fn is_expanded(&self, path: &Path) -> bool {
        self.expanded.contains(path)
    }

    /// Builds a flat list of tree entries by walking expanded directories.
    pub fn build_tree_entries(
        &self,
        root: &Path,
        show_hidden: bool,
        filter: Option<&dyn Fn(&Path) -> bool>,
    ) -> Vec<TreeEntry> {
        let mut result = Vec::new();
        self.collect_entries(root, 0, show_hidden, filter, &mut result);
        result
    }

    fn collect_entries(
        &self,
        dir: &Path,
        depth: usize,
        show_hidden: bool,
        filter: Option<&dyn Fn(&Path) -> bool>,
        result: &mut Vec<TreeEntry>,
    ) {
        let entries = read_entries(dir, show_hidden, filter);
        for entry in entries {
            let is_dir = entry.kind == EntryKind::Directory;
            let path = entry.path.clone();
            result.push(TreeEntry { entry, depth });
            if is_dir && self.is_expanded(&path) {
                self.collect_entries(&path, depth + 1, show_hidden, filter, result);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn toggle_expand() {
        let mut state = TreeViewState::new();
        let path = PathBuf::from("/some/dir");
        assert!(!state.is_expanded(&path));
        state.toggle_expand(&path);
        assert!(state.is_expanded(&path));
        state.toggle_expand(&path);
        assert!(!state.is_expanded(&path));
    }

    #[test]
    fn build_tree_flat_when_nothing_expanded() {
        let tmp = TempDir::new().unwrap();
        fs::write(tmp.path().join("a.txt"), "").unwrap();
        fs::create_dir(tmp.path().join("subdir")).unwrap();
        fs::write(tmp.path().join("subdir").join("b.txt"), "").unwrap();
        let state = TreeViewState::new();
        let tree = state.build_tree_entries(tmp.path(), false, None);
        assert_eq!(tree.len(), 2); // subdir + a.txt
        assert!(tree.iter().all(|e| e.depth == 0));
    }

    #[test]
    fn build_tree_with_expanded_dir() {
        let tmp = TempDir::new().unwrap();
        fs::write(tmp.path().join("a.txt"), "").unwrap();
        let subdir = tmp.path().join("subdir");
        fs::create_dir(&subdir).unwrap();
        fs::write(subdir.join("b.txt"), "").unwrap();
        let mut state = TreeViewState::new();
        state.toggle_expand(&subdir);
        let tree = state.build_tree_entries(tmp.path(), false, None);
        assert_eq!(tree.len(), 3); // subdir, b.txt (inside), a.txt
        let sub_entry = tree.iter().find(|e| e.entry.name == "b.txt").unwrap();
        assert_eq!(sub_entry.depth, 1);
    }
}
