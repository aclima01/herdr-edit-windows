//! herdr-edit — a simple text editor that runs in a herdr pane. Windows-only.
//!
//! Milestone 3: a file tree beside an editable `ropey` buffer. The tree opens a file on
//! select; the editor is modeless — printable keys insert, `Ctrl+S` saves, `Esc` returns
//! to the tree. The binary is long-lived and exits on `Ctrl+Q`.

mod app;
mod buffer;
mod herdr;
mod highlight;
mod tree;
mod ui;

use std::time::Duration;

use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

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

/// The frame loop: refresh the highlight, draw, wait for a key, mutate, repeat. Blocks on
/// input with a short poll timeout so the loop stays responsive without busy-spinning.
fn run(terminal: &mut ratatui::DefaultTerminal, app: &mut App) -> Result<()> {
    while !app.should_quit {
        app.refresh_highlight();
        terminal.draw(|f| ui::render(f, app))?;
        if event::poll(Duration::from_millis(250))?
            && let Event::Key(key) = event::read()?
            && key.kind == KeyEventKind::Press
        {
            handle_key(app, key);
        }
    }
    Ok(())
}

/// Route a keypress. `Ctrl+Q`/`Ctrl+S`/`Tab` are global; the rest go to the focused panel.
fn handle_key(app: &mut App, key: KeyEvent) {
    if key.modifiers.contains(KeyModifiers::CONTROL) {
        match key.code {
            KeyCode::Char('q') | KeyCode::Char('c') => {
                app.should_quit = true;
                return;
            }
            KeyCode::Char('s') => {
                app.save();
                return;
            }
            _ => {}
        }
    }
    if key.code == KeyCode::Tab {
        app.toggle_focus();
        return;
    }
    match app.focus {
        Focus::Tree => handle_tree_key(app, key.code),
        Focus::Editor => handle_editor_key(app, key.code),
    }
}

/// Tree navigation: move the cursor, expand/collapse, open a file, or quit.
fn handle_tree_key(app: &mut App, code: KeyCode) {
    match code {
        KeyCode::Char('q') => app.should_quit = true,
        KeyCode::Down => app.tree.move_down(),
        KeyCode::Up => app.tree.move_up(),
        KeyCode::Right => app.tree.expand(),
        KeyCode::Left => app.tree.collapse(),
        KeyCode::Enter => app.activate_selection(),
        _ => {}
    }
}

/// Modeless editing: move the cursor, insert, delete, or leave to the tree.
fn handle_editor_key(app: &mut App, code: KeyCode) {
    if code == KeyCode::Esc {
        app.focus = Focus::Tree;
        return;
    }
    let Some(doc) = app.doc.as_mut() else { return };
    let page = doc.viewport_rows.saturating_sub(1).max(1);
    match code {
        KeyCode::Left => doc.buffer.move_left(),
        KeyCode::Right => doc.buffer.move_right(),
        KeyCode::Up => doc.buffer.move_up(1),
        KeyCode::Down => doc.buffer.move_down(1),
        KeyCode::PageUp => doc.buffer.move_up(page),
        KeyCode::PageDown => doc.buffer.move_down(page),
        KeyCode::Home => doc.buffer.move_home(),
        KeyCode::End => doc.buffer.move_end(),
        KeyCode::Enter => doc.insert_newline(),
        KeyCode::Backspace => doc.backspace(),
        KeyCode::Delete => doc.delete_forward(),
        KeyCode::Char(c) => doc.insert_char(c),
        _ => {}
    }
}
