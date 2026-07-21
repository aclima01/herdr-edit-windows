//! The file tree: a filesystem tree rooted at the pane's working directory. Directories
//! expand lazily; the visible rows are a flat sequence over the expanded tree, which the
//! selection and rendering both walk.
//!
//! `.git` is the one exclusion. Directories sort first, then files, both case-insensitive.
//! The tree is rebuilt from disk on every expand/collapse; a simple editor's tree is small
//! enough that reading the expanded directories each time stays cheap.

use std::collections::HashSet;
use std::path::{Path, PathBuf};

/// One visible row over the tree.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Node {
    pub path: PathBuf,
    /// The segment shown: a directory or file name.
    pub name: String,
    /// Nesting level, for indentation.
    pub depth: usize,
    pub is_dir: bool,
    /// For a directory, whether its children are shown.
    pub expanded: bool,
}

/// The tree and its view state: the flat visible rows, which directories are expanded, and
/// the selection cursor with its scroll offset.
#[derive(Debug)]
pub struct Tree {
    root: PathBuf,
    expanded: HashSet<PathBuf>,
    nodes: Vec<Node>,
    pub selected: usize,
    pub scroll: usize,
    pub viewport_rows: usize,
}

impl Tree {
    /// Build a collapsed tree rooted at `root`.
    pub fn new(root: PathBuf) -> Self {
        let mut tree = Self {
            root,
            expanded: HashSet::new(),
            nodes: Vec::new(),
            selected: 0,
            scroll: 0,
            viewport_rows: 0,
        };
        tree.rebuild();
        tree
    }

    pub fn nodes(&self) -> &[Node] {
        &self.nodes
    }

    pub fn selected_node(&self) -> Option<&Node> {
        self.nodes.get(self.selected)
    }

    /// Re-read the expanded directories from disk into the flat row list, keeping the
    /// selection within bounds.
    fn rebuild(&mut self) {
        let mut nodes = Vec::new();
        let root = self.root.clone();
        self.walk(&root, 0, &mut nodes);
        self.nodes = nodes;
        if self.selected >= self.nodes.len() {
            self.selected = self.nodes.len().saturating_sub(1);
        }
    }

    /// Append `dir`'s children as rows, recursing into expanded subdirectories.
    fn walk(&self, dir: &Path, depth: usize, out: &mut Vec<Node>) {
        let Ok(entries) = std::fs::read_dir(dir) else {
            return;
        };
        let mut items: Vec<(PathBuf, String, bool)> = entries
            .flatten()
            .filter_map(|e| {
                let name = e.file_name().to_string_lossy().into_owned();
                if name == ".git" {
                    return None;
                }
                let is_dir = e.file_type().map(|t| t.is_dir()).unwrap_or(false);
                Some((e.path(), name, is_dir))
            })
            .collect();
        // Directories first, then files, both case-insensitive by name.
        items.sort_by(|a, b| {
            b.2.cmp(&a.2)
                .then_with(|| a.1.to_lowercase().cmp(&b.1.to_lowercase()))
        });
        for (path, name, is_dir) in items {
            let expanded = is_dir && self.expanded.contains(&path);
            out.push(Node {
                path: path.clone(),
                name,
                depth,
                is_dir,
                expanded,
            });
            if expanded {
                self.walk(&path, depth + 1, out);
            }
        }
    }

    pub fn move_down(&mut self) {
        if self.selected + 1 < self.nodes.len() {
            self.selected += 1;
        }
    }

    pub fn move_up(&mut self) {
        self.selected = self.selected.saturating_sub(1);
    }

    /// Expand the selected directory. No-op on a file or an already-expanded directory.
    pub fn expand(&mut self) {
        if let Some(node) = self.selected_node()
            && node.is_dir
            && !node.expanded
        {
            self.expanded.insert(node.path.clone());
            self.rebuild();
        }
    }

    /// Collapse the selected directory if expanded, else move the selection to its parent
    /// row, so `←` walks out of a subtree.
    pub fn collapse(&mut self) {
        let Some(node) = self.selected_node() else {
            return;
        };
        if node.is_dir && node.expanded {
            let path = node.path.clone();
            self.expanded.remove(&path);
            self.rebuild();
        } else if node.depth > 0 {
            let target_depth = node.depth - 1;
            for i in (0..self.selected).rev() {
                if self.nodes[i].depth == target_depth && self.nodes[i].is_dir {
                    self.selected = i;
                    break;
                }
            }
        }
    }

    /// Toggle the selected directory's expansion. Returns the file path when the selection
    /// is a file, so the caller opens it.
    pub fn activate(&mut self) -> Option<PathBuf> {
        let node = self.selected_node()?;
        if node.is_dir {
            if node.expanded {
                self.collapse();
            } else {
                self.expand();
            }
            None
        } else {
            Some(node.path.clone())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Tree;
    use std::fs;

    /// A small on-disk fixture: a dir with a file and a nested dir, plus a `.git` to skip.
    fn fixture() -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        fs::create_dir(root.join("src")).unwrap();
        fs::write(root.join("src").join("main.rs"), "fn main() {}").unwrap();
        fs::write(root.join("README.md"), "# hi").unwrap();
        fs::create_dir(root.join(".git")).unwrap();
        fs::write(root.join(".git").join("HEAD"), "ref: x").unwrap();
        dir
    }

    #[test]
    fn lists_dirs_first_and_skips_git() {
        let dir = fixture();
        let tree = Tree::new(dir.path().to_path_buf());
        let names: Vec<_> = tree.nodes().iter().map(|n| n.name.as_str()).collect();
        assert_eq!(names, vec!["src", "README.md"]);
    }

    #[test]
    fn expanding_a_dir_reveals_its_children() {
        let dir = fixture();
        let mut tree = Tree::new(dir.path().to_path_buf());
        tree.expand(); // src is selected (row 0)
        let names: Vec<_> = tree.nodes().iter().map(|n| n.name.as_str()).collect();
        assert_eq!(names, vec!["src", "main.rs", "README.md"]);
    }

    #[test]
    fn activate_returns_a_file_path() {
        let dir = fixture();
        let mut tree = Tree::new(dir.path().to_path_buf());
        tree.move_down(); // README.md
        let opened = tree.activate();
        assert_eq!(opened.unwrap().file_name().unwrap(), "README.md");
    }
}
