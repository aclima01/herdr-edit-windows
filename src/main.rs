//! herdr-edit — a simple text editor that runs in a herdr pane. Windows-only.
//!
//! Milestone 2: a file tree beside a read-only, syntax-highlighted editor. The tree opens
//! a file on select. The binary is long-lived: it paints a TUI in the pane herdr opens for
//! it and exits on `q`.

mod app;
mod herdr;
mod highlight;
mod tree;
mod ui;

use std::time::Duration;

use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};

use crate::app::{App, Focus, init};
use crate::herdr::Context;
use crate::highlight::Highlighter;

fn main() -> Result<()> {
    let context = Context::from_env();
    let mut app = init(context, Highlighter::new());

    let mut terminal = ratatui::init();
    let result = run(&mut terminal, &mut app);
    ratatui::restore();
    result
}

/// The frame loop: draw, wait for a key, mutate, repeat. Blocks on input with a short
/// poll timeout so the loop stays responsive without busy-spinning.
fn run(terminal: &mut ratatui::DefaultTerminal, app: &mut App) -> Result<()> {
    while !app.should_quit {
        terminal.draw(|f| ui::render(f, app))?;
        if event::poll(Duration::from_millis(250))?
            && let Event::Key(key) = event::read()?
            && key.kind == KeyEventKind::Press
        {
            handle_key(app, key.code);
        }
    }
    Ok(())
}

/// Route a keypress. `q`/`Tab` are global; the rest go to the focused panel.
fn handle_key(app: &mut App, code: KeyCode) {
    match code {
        KeyCode::Char('q') => {
            app.should_quit = true;
            return;
        }
        KeyCode::Tab => {
            app.toggle_focus();
            return;
        }
        _ => {}
    }
    match app.focus {
        Focus::Tree => handle_tree_key(app, code),
        Focus::Editor => handle_editor_key(app, code),
    }
}

/// Tree navigation: move the cursor, expand/collapse, or open a file.
fn handle_tree_key(app: &mut App, code: KeyCode) {
    match code {
        KeyCode::Down | KeyCode::Char('j') => app.tree.move_down(),
        KeyCode::Up | KeyCode::Char('k') => app.tree.move_up(),
        KeyCode::Right | KeyCode::Char('l') => app.tree.expand(),
        KeyCode::Left | KeyCode::Char('h') => app.tree.collapse(),
        KeyCode::Enter => app.activate_selection(),
        _ => {}
    }
}

/// Editor scrolling. Paging moves by one viewport height; `Esc` returns focus to the tree.
fn handle_editor_key(app: &mut App, code: KeyCode) {
    let Some(doc) = app.doc.as_mut() else { return };
    let page = doc.viewport_rows.saturating_sub(1).max(1);
    match code {
        KeyCode::Down | KeyCode::Char('j') => doc.scroll_down(1),
        KeyCode::Up | KeyCode::Char('k') => doc.scroll_up(1),
        KeyCode::PageDown | KeyCode::Char(' ') => doc.scroll_down(page),
        KeyCode::PageUp => doc.scroll_up(page),
        KeyCode::Char('g') | KeyCode::Home => doc.scroll_to_start(),
        KeyCode::Char('G') | KeyCode::End => doc.scroll_to_end(),
        KeyCode::Esc => app.focus = Focus::Tree,
        _ => {}
    }
}
