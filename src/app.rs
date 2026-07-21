//! The editor's state. Milestone 3: the editor is an editable `ropey` buffer. The tree
//! opens a file into it; typing inserts, `Ctrl+S` saves, and the syntax highlight is
//! recomputed from the buffer after each edit. The diff tab comes in Milestone 4.

use std::path::{Path, PathBuf};

use anyhow::{Context as _, Result};

use crate::buffer::Buffer;
use crate::herdr::Context;
use crate::highlight::{Highlighter, Span};
use crate::tree::Tree;

/// Which panel takes keyboard input.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Focus {
    Tree,
    Editor,
}

/// One opened document: the file path, the editable buffer, the scroll offset, and a cached
/// syntax highlight rebuilt lazily after edits.
#[derive(Debug)]
pub struct Document {
    pub path: Option<PathBuf>,
    pub title: String,
    ext: Option<String>,
    pub buffer: Buffer,
    pub scroll: usize,
    pub viewport_rows: usize,
    highlight: Vec<Vec<Span>>,
    highlight_dirty: bool,
}

impl Document {
    fn new(path: Option<PathBuf>, title: String, text: &str) -> Self {
        let ext = path.as_ref().and_then(|p| p.extension()).and_then(|e| e.to_str()).map(str::to_owned);
        Self {
            path,
            title,
            ext,
            buffer: Buffer::from_str(text),
            scroll: 0,
            viewport_rows: 0,
            highlight: Vec::new(),
            highlight_dirty: true,
        }
    }

    /// The cached per-line highlight spans. Valid only after [`ensure_highlight`].
    pub fn highlight(&self) -> &[Vec<Span>] {
        &self.highlight
    }

    /// Rebuild the highlight from the buffer if an edit invalidated it. Cheap when clean.
    pub fn ensure_highlight(&mut self, highlighter: &Highlighter) {
        if self.highlight_dirty {
            self.highlight = highlighter.highlight(&self.buffer.text(), self.ext.as_deref());
            self.highlight_dirty = false;
        }
    }

    // --- editing (invalidates the highlight) -------------------------------

    pub fn insert_char(&mut self, c: char) {
        self.buffer.insert_char(c);
        self.highlight_dirty = true;
    }

    pub fn insert_newline(&mut self) {
        self.buffer.insert_newline();
        self.highlight_dirty = true;
    }

    pub fn backspace(&mut self) {
        self.buffer.backspace();
        self.highlight_dirty = true;
    }

    pub fn delete_forward(&mut self) {
        self.buffer.delete_forward();
        self.highlight_dirty = true;
    }

    /// Write the buffer to its path. Returns the number of bytes written.
    pub fn save(&mut self) -> Result<usize> {
        let path = self.path.as_ref().context("no file path to save to")?;
        let text = self.buffer.text();
        std::fs::write(path, &text).with_context(|| format!("writing {}", path.display()))?;
        self.buffer.clear_modified();
        Ok(text.len())
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

    /// Rebuild the open document's highlight if an edit dirtied it. Called once per frame
    /// before drawing, so rendering borrows an up-to-date cache immutably.
    pub fn refresh_highlight(&mut self) {
        if let Some(doc) = self.doc.as_mut() {
            doc.ensure_highlight(&self.highlighter);
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

    /// Open `path` into the editor and focus it. Line endings are normalized to `\n` (the
    /// buffer works in `\n`; a save writes `\n`). A read error stays on the tree with the
    /// reason in the status line.
    pub fn open_path(&mut self, path: &Path) {
        match std::fs::read_to_string(path) {
            Ok(raw) => {
                let text = raw.replace("\r\n", "\n");
                let title = path.file_name().and_then(|n| n.to_str()).unwrap_or("file").to_string();
                self.status = format!("opened {}", path.display());
                self.doc = Some(Document::new(Some(path.to_path_buf()), title, &text));
                self.focus = Focus::Editor;
            }
            Err(e) => {
                self.status = format!("could not open {}: {e}", path.display());
            }
        }
    }

    /// Save the open document, reporting the outcome in the status line.
    pub fn save(&mut self) {
        let Some(doc) = self.doc.as_mut() else {
            self.status = "nothing to save".to_string();
            return;
        };
        match doc.save() {
            Ok(bytes) => self.status = format!("saved {} ({bytes} bytes)", doc.title),
            Err(e) => self.status = format!("save failed: {e}"),
        }
    }
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
