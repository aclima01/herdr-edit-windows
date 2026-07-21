//! Rendering. A title bar (file name + herdr context), the highlighted body with a
//! left gutter of line numbers, and a footer of key hints.

use ratatui::Frame;
use ratatui::layout::{Constraint, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span as TuiSpan};
use ratatui::widgets::{Block, Borders, Paragraph};

use crate::app::App;

/// Muted color for the gutter, footer, and title chrome.
const GUTTER_FG: Color = Color::Rgb(0x6c, 0x70, 0x86);

/// Draw the whole frame from `app`. Updates `app.viewport_rows` so the caller's paging
/// keys move by a real screenful.
pub fn render(f: &mut Frame, app: &mut App) {
    let areas = Layout::vertical([
        Constraint::Length(1), // title
        Constraint::Min(1),    // body
        Constraint::Length(1), // footer
    ])
    .split(f.area());
    let (title_area, body_area, footer_area) = (areas[0], areas[1], areas[2]);

    // Title bar: file name on the left, herdr context on the right.
    let title = Line::from(vec![
        TuiSpan::styled(
            format!(" {} ", app.title),
            Style::default().fg(Color::Black).bg(Color::Rgb(0x89, 0xb4, 0xfa)).add_modifier(Modifier::BOLD),
        ),
        TuiSpan::raw("  "),
        TuiSpan::styled(app.context.summary(), Style::default().fg(GUTTER_FG)),
    ]);
    f.render_widget(Paragraph::new(title), title_area);

    // Body: the visible slice, with a line-number gutter.
    let body = Block::default().borders(Borders::NONE);
    let inner = body.inner(body_area);
    app.viewport_rows = inner.height as usize;
    f.render_widget(body, body_area);

    let total = app.lines.len();
    let gutter_width = total.to_string().len().max(2);
    let start = app.scroll.min(total.saturating_sub(1));
    let end = (start + app.viewport_rows).min(total);

    let mut rows: Vec<Line> = Vec::with_capacity(end - start);
    for (i, spans) in app.lines[start..end].iter().enumerate() {
        let lineno = start + i + 1;
        let mut cells = vec![TuiSpan::styled(
            format!("{lineno:>gutter_width$} "),
            Style::default().fg(GUTTER_FG),
        )];
        for s in spans {
            cells.push(TuiSpan::styled(
                s.text.clone(),
                Style::default().fg(Color::Rgb(s.color.0, s.color.1, s.color.2)),
            ));
        }
        rows.push(Line::from(cells));
    }
    f.render_widget(Paragraph::new(rows), inner);

    // Footer: scroll position and key hints.
    let pos = format!(" {}/{} ", end.min(total), total);
    let footer = Line::from(vec![
        TuiSpan::styled(pos, Style::default().fg(GUTTER_FG)),
        TuiSpan::styled(
            "  ↑/↓ PgUp/PgDn  g/G top/bottom  q quit",
            Style::default().fg(GUTTER_FG),
        ),
        TuiSpan::raw("   "),
        TuiSpan::styled(app.status.clone(), Style::default().fg(Color::Rgb(0xf9, 0xe2, 0xaf))),
    ]);
    f.render_widget(Paragraph::new(footer), footer_area);
}
