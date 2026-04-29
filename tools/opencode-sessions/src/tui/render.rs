use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph, Wrap};

use crate::format::*;
use crate::models::*;
use crate::tui::app::*;

pub(crate) fn draw_tui(frame: &mut ratatui::Frame<'_>, app: &mut TuiApp) {
    let areas = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(6),
            Constraint::Length(3),
            Constraint::Min(1),
            Constraint::Length(4),
        ])
        .split(frame.area());

    let header = Paragraph::new(vec![
        Line::from(vec![
            Span::styled("OpenCode Sessions", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw("  "),
            Span::styled(
                format!("roots={} visible={}", app.index.roots.len(), app.visible_rows.len()),
                Style::default().fg(Color::Cyan),
            ),
        ]),
        Line::from(vec![
            Span::styled("DB: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(app.db_path.display().to_string()),
        ]),
        Line::from(vec![
            Span::styled("export: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw("e selected subtree  E root conversation  o open last export"),
        ]),
        Line::from(vec![
            Span::styled("browse: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw("↑↓/jk move  enter toggle  / search  esc clear search  a expand all  z collapse all  q quit"),
        ]),
    ])
    .block(Block::default().borders(Borders::ALL));
    frame.render_widget(header, areas[0]);

    let search_title = if app.search_mode { "Search (typing)" } else { "Search (/ to edit)" };
    let search = Paragraph::new(app.search.as_str())
        .block(Block::default().borders(Borders::ALL).title(search_title))
        .wrap(Wrap { trim: false });
    frame.render_widget(search, areas[1]);

    let items = if app.visible_rows.is_empty() {
        vec![ListItem::new(Line::from("No sessions match search."))]
    } else {
        app.visible_rows
            .iter()
            .map(|row| ListItem::new(Line::from(format_row(app, row))))
            .collect()
    };

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("Recent conversations"))
        .highlight_style(Style::default().bg(Color::Rgb(30, 30, 70)).fg(Color::White))
        .highlight_symbol("▶ ");

    frame.render_stateful_widget(list, areas[2], &mut app.list_state);

    let footer = Paragraph::new(vec![
        Line::from(vec![
            Span::styled("selected: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(selected_summary(app)),
        ]),
        Line::from(vec![
            Span::styled("status: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(app.status.as_str()),
        ]),
    ])
    .block(Block::default().borders(Borders::ALL));
    frame.render_widget(footer, areas[3]);
}

pub(crate) fn format_row(app: &TuiApp, row: &VisibleRow) -> String {
    let session = match app.index.sessions.get(&row.session_id) {
        Some(session) => session,
        None => return row.session_id.clone(),
    };

    let children = app.index.children_of(&row.session_id);
    let marker = if children.is_empty() {
        "·"
    } else if !app.search.trim().is_empty() || app.expanded.contains(&row.session_id) {
        "▾"
    } else {
        "▸"
    };

    let indent = "  ".repeat(row.depth);
    let kind = if row.depth == 0 {
        String::from("root")
    } else {
        session.agent_hint().unwrap_or_else(|| String::from("subagent"))
    };

    format!(
        "{}{} [{}] {}  {}  {}  {} msgs  {}",
        indent,
        marker,
        kind,
        session.title,
        short_id(&session.id),
        format_local_timestamp(session.time_updated),
        session.message_count,
        format_duration(session.duration_ms()),
    )
}

pub(crate) fn selected_summary(app: &TuiApp) -> String {
    let Some(session_id) = app.selected_session_id() else {
        return String::from("none");
    };
    let Some(session) = app.index.sessions.get(session_id) else {
        return String::from("none");
    };

    let kind = if session.parent_id.is_some() {
        session.agent_hint().unwrap_or_else(|| String::from("subagent"))
    } else {
        String::from("root")
    };

    format!("[{}] {}  {}", kind, session.title, short_id(&session.id))
}
