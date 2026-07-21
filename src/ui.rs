//! Rendering. A title bar, then a split body: the file tree on the left and the editor on
//! the right, the focused panel titled brightly. A footer shows position and key hints.

use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span as TuiSpan};
use ratatui::widgets::{Block, Borders, Paragraph};
use unicode_width::UnicodeWidthStr;

use crate::app::{App, EditorTab, Focus};
use crate::diff::DiffKind;
use crate::tree::Node;

const ACCENT: Color = Color::Rgb(0x89, 0xb4, 0xfa);
const MUTED: Color = Color::Rgb(0x6c, 0x70, 0x86);
const DIR_FG: Color = Color::Rgb(0x89, 0xb4, 0xfa);
const STATUS_FG: Color = Color::Rgb(0xf9, 0xe2, 0xaf);
const SELECT_BG: Color = Color::Rgb(0x31, 0x32, 0x44);

/// Draw the whole frame from `app`, updating the tree and editor viewport heights so paging
/// keys move by a real screenful.
pub fn render(f: &mut Frame, app: &mut App) {
    let areas = Layout::vertical([
        Constraint::Length(1), // title
        Constraint::Min(1),    // body
        Constraint::Length(1), // footer
    ])
    .split(f.area());
    let (title_area, body_area, footer_area) = (areas[0], areas[1], areas[2]);

    render_title(f, app, title_area);

    // A fixed-width tree beside the editor, the tree never wider than a third of the pane.
    let tree_width = 32.min(body_area.width / 3).max(16);
    let split = Layout::horizontal([Constraint::Length(tree_width), Constraint::Min(1)])
        .split(body_area);
    render_tree(f, app, split[0]);
    render_editor(f, app, split[1]);

    render_footer(f, app, footer_area);
}

fn render_title(f: &mut Frame, app: &App, area: Rect) {
    let (name, modified) = match &app.doc {
        Some(d) => (d.title.as_str(), d.buffer.modified),
        None => ("herdr-edit", false),
    };
    let marker = if modified { "● " } else { "" };
    let title = Line::from(vec![
        TuiSpan::styled(
            format!(" {marker}{name} "),
            Style::default().fg(Color::Black).bg(ACCENT).add_modifier(Modifier::BOLD),
        ),
        TuiSpan::raw("  "),
        TuiSpan::styled(app.context.summary(), Style::default().fg(MUTED)),
    ]);
    f.render_widget(Paragraph::new(title), area);
}

fn render_tree(f: &mut Frame, app: &mut App, area: Rect) {
    let focused = app.focus == Focus::Tree;
    let block = panel_block("Files", focused);
    let inner = block.inner(area);
    f.render_widget(block, area);

    app.tree.viewport_rows = inner.height as usize;
    let rows = keep_visible(app.tree.selected, &mut app.tree.scroll, inner.height as usize);

    let nodes = app.tree.nodes();
    let start = app.tree.scroll.min(nodes.len().saturating_sub(1));
    let end = (start + rows).min(nodes.len());
    let mut lines: Vec<Line> = Vec::with_capacity(end.saturating_sub(start));
    for (i, node) in nodes[start..end].iter().enumerate() {
        let idx = start + i;
        lines.push(tree_row(node, idx == app.tree.selected, focused));
    }
    f.render_widget(Paragraph::new(lines), inner);
}

/// One tree row: indent, an expansion caret for a directory, and the name colored by kind.
fn tree_row(node: &Node, selected: bool, focused: bool) -> Line<'static> {
    let indent = "  ".repeat(node.depth);
    let marker = if node.is_dir {
        if node.expanded { "▾ " } else { "▸ " }
    } else {
        "  "
    };
    let name_style = if node.is_dir {
        Style::default().fg(DIR_FG).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::Rgb(0xcd, 0xd6, 0xf4))
    };
    let name = if node.is_dir { format!("{}/", node.name) } else { node.name.clone() };
    let mut line = Line::from(vec![
        TuiSpan::styled(format!("{indent}{marker}"), Style::default().fg(MUTED)),
        TuiSpan::styled(name, name_style),
    ]);
    if selected {
        let bg = if focused { SELECT_BG } else { Color::Rgb(0x25, 0x26, 0x36) };
        line = line.style(Style::default().bg(bg));
    }
    line
}

fn render_editor(f: &mut Frame, app: &mut App, area: Rect) {
    let focused = app.focus == Focus::Editor;
    let active_tab = app.doc.as_ref().map_or(EditorTab::Editor, |d| d.tab);
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(if focused { ACCENT } else { MUTED }))
        .title(tab_strip(active_tab, focused));
    let inner = block.inner(area);
    f.render_widget(block, area);

    let Some(doc) = app.doc.as_mut() else {
        let hint = Paragraph::new(vec![
            Line::from(""),
            Line::from(TuiSpan::styled(
                "  Select a file in the tree and press Enter.",
                Style::default().fg(MUTED),
            )),
            Line::from(TuiSpan::styled(
                "  Tab switches focus; Ctrl+D shows the uncommitted diff.",
                Style::default().fg(MUTED),
            )),
        ]);
        f.render_widget(hint, inner);
        return;
    };

    if doc.tab == EditorTab::Diff {
        render_diff_body(f, doc, inner);
        return;
    }

    doc.viewport_rows = inner.height as usize;
    let total = doc.buffer.line_count();
    let gutter_width = total.to_string().len().max(2);
    // The view follows the cursor: scroll just enough to keep it on screen.
    keep_visible(doc.buffer.cursor_line, &mut doc.scroll, inner.height as usize);
    let start = doc.scroll.min(total.saturating_sub(1));
    let end = (start + inner.height as usize).min(total);

    let highlight = doc.highlight();
    let mut rows: Vec<Line> = Vec::with_capacity(end.saturating_sub(start));
    for lineno in start..end {
        let mut cells = vec![TuiSpan::styled(
            format!("{:>gutter_width$} ", lineno + 1),
            Style::default().fg(MUTED),
        )];
        // A line past the cached highlight (e.g. a trailing empty line) renders blank.
        if let Some(spans) = highlight.get(lineno) {
            for s in spans {
                cells.push(TuiSpan::styled(
                    s.text.clone(),
                    Style::default().fg(Color::Rgb(s.color.0, s.color.1, s.color.2)),
                ));
            }
        }
        rows.push(Line::from(cells));
    }
    f.render_widget(Paragraph::new(rows), inner);

    // Place the real terminal cursor when the editor has focus. Its column is the display
    // width of the line up to the cursor, past the line-number gutter.
    if focused {
        let cur_line = doc.buffer.cursor_line;
        let prefix: String =
            doc.buffer.line_text(cur_line).chars().take(doc.buffer.cursor_col).collect();
        let x = inner.x + (gutter_width as u16) + 1 + UnicodeWidthStr::width(prefix.as_str()) as u16;
        let y = inner.y + (cur_line.saturating_sub(doc.scroll)) as u16;
        if y < inner.y + inner.height && x < inner.x + inner.width {
            f.set_cursor_position((x, y));
        }
    }
}

/// The editor panel's title: an " Editor │ Diff " strip, the active tab bright, the rest
/// dimmed. When the panel is unfocused every chip dims.
fn tab_strip(active: EditorTab, focused: bool) -> Line<'static> {
    let chip = |label: &str, on: bool| {
        let style = if on && focused {
            Style::default().fg(ACCENT).add_modifier(Modifier::BOLD)
        } else if on {
            Style::default().fg(Color::Rgb(0xcd, 0xd6, 0xf4))
        } else {
            Style::default().fg(MUTED)
        };
        TuiSpan::styled(format!(" {label} "), style)
    };
    Line::from(vec![
        chip("Editor", active == EditorTab::Editor),
        TuiSpan::styled("│", Style::default().fg(MUTED)),
        chip("Diff", active == EditorTab::Diff),
    ])
}

/// Render the uncommitted diff: each line colored by its role, or a centered note when there
/// is nothing to show.
fn render_diff_body(f: &mut Frame, doc: &mut crate::app::Document, inner: Rect) {
    if let Some(note) = &doc.diff_note {
        let msg = Paragraph::new(vec![
            Line::from(""),
            Line::from(TuiSpan::styled(format!("  {note}"), Style::default().fg(MUTED))),
        ]);
        f.render_widget(msg, inner);
        return;
    }
    doc.diff_viewport_rows = inner.height as usize;
    let total = doc.diff_lines.len();
    let start = doc.diff_scroll.min(total.saturating_sub(1));
    let end = (start + inner.height as usize).min(total);
    let mut rows: Vec<Line> = Vec::with_capacity(end.saturating_sub(start));
    for line in &doc.diff_lines[start..end] {
        let color = match line.kind {
            DiffKind::Add => Color::Rgb(0xa6, 0xe3, 0xa1),
            DiffKind::Remove => Color::Rgb(0xf3, 0x8b, 0xa8),
            DiffKind::Hunk => Color::Rgb(0x89, 0xdc, 0xeb),
            DiffKind::Meta => MUTED,
            DiffKind::Context => Color::Rgb(0xcd, 0xd6, 0xf4),
        };
        rows.push(Line::from(TuiSpan::styled(line.text.clone(), Style::default().fg(color))));
    }
    f.render_widget(Paragraph::new(rows), inner);
}

fn render_footer(f: &mut Frame, app: &App, area: Rect) {
    let diff_active = app.doc.as_ref().is_some_and(|d| d.tab == EditorTab::Diff);
    let pos = match &app.doc {
        Some(doc) if diff_active => format!(" diff · {} lines ", doc.diff_lines.len()),
        Some(doc) => format!(
            " {}:{} / {} ",
            doc.buffer.cursor_line + 1,
            doc.buffer.cursor_col + 1,
            doc.buffer.line_count()
        ),
        None => format!(" {} files ", app.tree.nodes().len()),
    };
    let hints = match app.focus {
        Focus::Tree => "  ↑/↓ move  →/Enter open  ← collapse  Tab editor  q quit",
        Focus::Editor if diff_active => "  ↑/↓ scroll  Ctrl+D editor  Esc tree  Ctrl+Q quit",
        Focus::Editor => "  type to edit  Ctrl+S save  Ctrl+D diff  Esc tree",
    };
    let footer = Line::from(vec![
        TuiSpan::styled(pos, Style::default().fg(MUTED)),
        TuiSpan::styled(hints, Style::default().fg(MUTED)),
        TuiSpan::raw("   "),
        TuiSpan::styled(app.status.clone(), Style::default().fg(STATUS_FG)),
    ]);
    f.render_widget(Paragraph::new(footer), area);
}

/// A bordered panel whose title brightens when focused.
fn panel_block(title: &str, focused: bool) -> Block<'static> {
    let (border, title_style) = if focused {
        (Style::default().fg(ACCENT), Style::default().fg(ACCENT).add_modifier(Modifier::BOLD))
    } else {
        (Style::default().fg(MUTED), Style::default().fg(MUTED))
    };
    Block::default()
        .borders(Borders::ALL)
        .border_style(border)
        .title(TuiSpan::styled(format!(" {title} "), title_style))
}

/// Clamp `scroll` so `selected` stays within a `height`-row viewport, and return `height`
/// (the number of rows to draw). Scrolls just enough to reveal the selection at either edge.
fn keep_visible(selected: usize, scroll: &mut usize, height: usize) -> usize {
    if height == 0 {
        return 0;
    }
    if selected < *scroll {
        *scroll = selected;
    } else if selected >= *scroll + height {
        *scroll = selected + 1 - height;
    }
    height
}
