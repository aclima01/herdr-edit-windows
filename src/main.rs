//! herdr-edit — a simple text editor that runs in a herdr pane. Windows-only.
//!
//! Milestone 1: open one file (a CLI argument, else an embedded sample), highlight it
//! with syntect, and scroll it read-only. The binary is long-lived: it paints a TUI in
//! the pane herdr opens for it and exits on `q`.

mod app;
mod herdr;
mod highlight;
mod ui;

use std::time::Duration;

use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};

use crate::app::{App, open_from_args};
use crate::herdr::Context;
use crate::highlight::Highlighter;

fn main() -> Result<()> {
    let context = Context::from_env();
    let highlighter = Highlighter::new();
    let mut app = open_from_args(context, &highlighter);

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

/// Map a keypress onto a scroll or quit. Paging moves by one viewport height.
fn handle_key(app: &mut App, code: KeyCode) {
    let page = app.viewport_rows.saturating_sub(1).max(1);
    match code {
        KeyCode::Char('q') | KeyCode::Esc => app.should_quit = true,
        KeyCode::Down | KeyCode::Char('j') => app.scroll_down(1),
        KeyCode::Up | KeyCode::Char('k') => app.scroll_up(1),
        KeyCode::PageDown | KeyCode::Char(' ') => app.scroll_down(page),
        KeyCode::PageUp => app.scroll_up(page),
        KeyCode::Char('g') | KeyCode::Home => app.scroll = 0,
        KeyCode::Char('G') | KeyCode::End => app.scroll_to_end(),
        _ => {}
    }
}
