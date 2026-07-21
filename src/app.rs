//! The editor's state. Milestone 2: a file tree beside a read-only editor. The tree
//! opens a file on select; the editor shows it highlighted and scrolls it. Editing and
//! the diff tab come in later milestones.

use std::path::{Path, PathBuf};

use anyhow::{Context as _, Result};

use crate::herdr::Context;
use crate::highlight::{Highlighter, Span};
use crate::tree::Tree;

/// Which panel takes keyboard input.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Focus {
    Tree,
    Editor,
}

/// One opened, read-only document: its display name, highlighted lines, and scroll offset.
#[derive(Debug)]
pub struct Document {
    pub title: String,
    pub lines: Vec<Vec<Span>>,
    pub scroll: usize,
    pub viewport_rows: usize,
}

impl Document {
    fn from_lines(title: String, lines: Vec<Vec<Span>>) -> Self {
        Self { title, lines, scroll: 0, viewport_rows: 0 }
    }

    fn max_scroll(&self) -> usize {
        self.lines.len().saturating_sub(1)
    }

    pub fn scroll_down(&mut self, n: usize) {
        self.scroll = (self.scroll + n).min(self.max_scroll());
    }

    pub fn scroll_up(&mut self, n: usize) {
        self.scroll = self.scroll.saturating_sub(n);
    }

    pub fn scroll_to_start(&mut self) {
        self.scroll = 0;
    }

    pub fn scroll_to_end(&mut self) {
        self.scroll = self.max_scroll();
    }
}

/// The whole editor: the herdr context, the file tree, the open document, and which panel
/// has focus.
#[derive(Debug)]
pub struct App {
    pub context: Context,
    pub tree: Tree,
    pub doc: Option<Document>,
    pub focus: Focus,
    pub highlighter: Highlighter,
    pub should_quit: bool,
    pub status: String,
}

impl App {
    /// Build the editor rooted at the pane's working directory.
    pub fn new(context: Context, highlighter: Highlighter) -> Self {
        let root = context.cwd.clone();
        let status = context.summary();
        Self {
            context,
            tree: Tree::new(root),
            doc: None,
            focus: Focus::Tree,
            highlighter,
            should_quit: false,
            status,
        }
    }

    /// Toggle focus between the tree and the editor.
    pub fn toggle_focus(&mut self) {
        self.focus = match self.focus {
            Focus::Tree => Focus::Editor,
            Focus::Editor => Focus::Tree,
        };
    }

    /// Activate the selected tree row: expand/collapse a directory, or open a file into the
    /// editor and move focus to it.
    pub fn activate_selection(&mut self) {
        if let Some(path) = self.tree.activate() {
            self.open_path(&path);
        }
    }

    /// Open `path` read-only, highlight it, and focus the editor. A read error stays on the
    /// tree with the reason in the status line.
    pub fn open_path(&mut self, path: &Path) {
        match read_document(path, &self.highlighter) {
            Ok(doc) => {
                self.status = format!("opened {}", path.display());
                self.doc = Some(doc);
                self.focus = Focus::Editor;
            }
            Err(e) => {
                self.status = format!("could not open {}: {e}", path.display());
            }
        }
    }
}

/// Read `path` into a highlighted [`Document`].
fn read_document(path: &Path, highlighter: &Highlighter) -> Result<Document> {
    let content =
        std::fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))?;
    let ext = path.extension().and_then(|e| e.to_str());
    let lines = highlighter.highlight(&content, ext);
    let title = path.file_name().and_then(|n| n.to_str()).unwrap_or("file").to_string();
    Ok(Document::from_lines(title, lines))
}

/// Build the editor and open an initial file: the first CLI argument if given, else nothing
/// (the tree waits for a selection).
pub fn init(context: Context, highlighter: Highlighter) -> App {
    let arg = std::env::args().nth(1);
    let mut app = App::new(context, highlighter);
    if let Some(arg) = arg {
        app.open_path(&PathBuf::from(arg));
    }
    app
}
