//! The editor's state. Milestone 1 is read-only: one opened file, highlighted, with a
//! vertical scroll offset. Editing, the file tree, and the diff tab come in later milestones.

use std::path::{Path, PathBuf};

use anyhow::{Context as _, Result};

use crate::herdr::Context;
use crate::highlight::{Highlighter, Span};

/// One opened, read-only document: its display name, highlighted lines, and scroll offset.
#[derive(Debug)]
pub struct App {
    pub context: Context,
    /// Display name shown in the title bar.
    pub title: String,
    /// The file's lines, pre-highlighted into per-line spans.
    pub lines: Vec<Vec<Span>>,
    /// Index of the first visible line.
    pub scroll: usize,
    /// Height of the text viewport in rows, updated each draw for clamped scrolling.
    pub viewport_rows: usize,
    pub should_quit: bool,
    pub status: String,
}

impl App {
    /// Open `path` read-only and highlight it by its extension.
    pub fn open(context: Context, path: &Path, highlighter: &Highlighter) -> Result<Self> {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("reading {}", path.display()))?;
        let ext = path.extension().and_then(|e| e.to_str());
        let lines = highlighter.highlight(&content, ext);
        let title = path.file_name().and_then(|n| n.to_str()).unwrap_or("file").to_string();
        Ok(Self::from_lines(context, title, lines))
    }

    /// Open the embedded sample, used when no file argument is given so the pane always
    /// shows highlighted content regardless of the working directory.
    pub fn sample(context: Context, highlighter: &Highlighter) -> Self {
        let lines = highlighter.highlight(SAMPLE, Some("rs"));
        Self::from_lines(context, "sample.rs".to_string(), lines)
    }

    fn from_lines(context: Context, title: String, lines: Vec<Vec<Span>>) -> Self {
        let status = context.summary();
        Self { context, title, lines, scroll: 0, viewport_rows: 0, should_quit: false, status }
    }

    /// The largest valid scroll offset given the current viewport, so the last line can
    /// reach the top but the view never scrolls past the end.
    pub fn max_scroll(&self) -> usize {
        self.lines.len().saturating_sub(1)
    }

    pub fn scroll_down(&mut self, n: usize) {
        self.scroll = (self.scroll + n).min(self.max_scroll());
    }

    pub fn scroll_up(&mut self, n: usize) {
        self.scroll = self.scroll.saturating_sub(n);
    }

    pub fn scroll_to_end(&mut self) {
        self.scroll = self.max_scroll();
    }
}

/// Resolve which file to open: the first CLI argument, else the embedded sample.
pub fn open_from_args(context: Context, highlighter: &Highlighter) -> App {
    if let Some(arg) = std::env::args().nth(1) {
        let path = PathBuf::from(&arg);
        match App::open(context.clone(), &path, highlighter) {
            Ok(app) => return app,
            Err(e) => {
                let mut app = App::sample(context, highlighter);
                app.status = format!("could not open {arg}: {e}");
                return app;
            }
        }
    }
    App::sample(context, highlighter)
}

/// A small Rust sample so the pane always shows highlighting, even outside a repo.
const SAMPLE: &str = r#"// herdr-edit — Milestone 1: read-only, syntax-highlighted view in a herdr pane.
use std::collections::HashMap;

/// Count word frequencies in a piece of text.
fn word_counts(text: &str) -> HashMap<String, usize> {
    let mut counts = HashMap::new();
    for word in text.split_whitespace() {
        *counts.entry(word.to_lowercase()).or_insert(0) += 1;
    }
    counts
}

fn main() {
    let text = "the quick brown fox the lazy dog the end";
    let counts = word_counts(text);
    let mut pairs: Vec<_> = counts.into_iter().collect();
    pairs.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
    for (word, n) in pairs {
        println!("{n:>3}  {word}");
    }
}
"#;
