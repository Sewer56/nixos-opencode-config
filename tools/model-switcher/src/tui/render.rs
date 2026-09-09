//! Rendering for the main view, compact fallback and picker overlays.

use super::app::{AppModel, Mode};
use super::handler::AppModelHandler;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph},
};

const MAIN_HELP: &str =
    "←/→ profile • ↑/↓ tier • enter model • v variant • s save • a apply • q quit";

/// Render the interactive views.
pub(crate) trait AppModelRender {
    fn render(&mut self, f: &mut Frame);
    fn render_picker(&self, f: &mut Frame);
    fn render_variant_picker(&self, f: &mut Frame);
}

impl AppModel<'_> {
    /// Affected-agent lines for the highlighted tier, capped to `height` rows.
    ///
    /// Returns empty text when `height` is zero so callers can hide the panel
    /// instead of reserving layout space for it.
    fn agent_text(&self, height: u16) -> String {
        if height == 0 {
            return String::new();
        }
        if let Some(error) = &self.agents_error {
            return format!("{} agents unavailable: {error}", self.tier());
        }
        let agents = self.selected_agents();
        if agents.is_empty() {
            return format!("{}: no tagged agents", self.tier());
        }
        let mut lines = vec![format!(
            "{} agents ({}/{}): PgUp/PgDn, Home/End",
            self.tier(),
            self.agent_offset + 1,
            agents.len()
        )];
        lines.extend(
            agents
                .iter()
                .skip(self.agent_offset)
                .take(height.saturating_sub(1) as usize)
                .cloned(),
        );
        lines.join("\n")
    }
}

impl<'a> AppModelRender for AppModel<'a> {
    fn render(&mut self, f: &mut Frame) {
        let tiers = self.tier_order.len() as u16;
        // Chrome that always renders: title, profiles, two blanks, tiers,
        // message and help. Preview and affected agents share the rest.
        if f.area().height < 6 + tiers {
            render_compact(self, f);
        } else {
            render_full(self, f, tiers);
        }
        if matches!(self.mode, Mode::ModelPicker) {
            self.render_picker(f);
        }
        if matches!(self.mode, Mode::VariantPicker) {
            self.render_variant_picker(f);
        }
    }

    fn render_picker(&self, f: &mut Frame) {
        let filtered = self.filtered_models();
        let area = center_rect(f.area(), 60, 20);

        let mut lines = vec![
            Line::from(
                format!("choose {} {}", self.profile(), self.tier())
                    .bold()
                    .fg(Color::Cyan),
            ),
            Line::from(""),
            Line::from(format!("> {}", self.input)),
            Line::from(""),
        ];

        if filtered.is_empty() {
            lines.push(Line::from("no models match").fg(Color::Red));
        } else {
            let start = self.pick_idx.saturating_sub(8);
            let end = (start + 18).min(filtered.len());
            for (i, model) in filtered.iter().enumerate().take(end).skip(start) {
                if i == self.pick_idx {
                    lines.push(
                        Line::from(format!(" {} ", model))
                            .style(Style::default().bg(Color::Cyan).fg(Color::Black)),
                    );
                } else {
                    lines.push(Line::from(format!(" {}", model)));
                }
            }
            lines.push(Line::from(format!(
                "{}/{}",
                self.pick_idx + 1,
                filtered.len()
            )));
        }
        lines.push(Line::from(""));
        lines.push(
            Line::from("type to filter • ↑/↓ move • enter select • esc back").fg(Color::DarkGray),
        );

        let block = Block::default()
            .borders(Borders::ALL)
            .style(Style::default());
        let inner = block.inner(area);
        f.render_widget(block, area);
        f.render_widget(Paragraph::new(lines), inner);
    }

    fn render_variant_picker(&self, f: &mut Frame) {
        let area = center_rect(f.area(), 40, 12);
        let mut lines = vec![
            Line::from(format!("choose {} {} variant", self.profile(), self.tier()))
                .bold()
                .fg(Color::Cyan),
            Line::from(""),
        ];
        for (i, variant) in crate::types::VARIANTS.iter().enumerate() {
            if i == self.pick_idx {
                lines.push(
                    Line::from(format!(" {} ", variant))
                        .style(Style::default().bg(Color::Cyan).fg(Color::Black)),
                );
            } else {
                lines.push(Line::from(format!(" {}", variant)));
            }
        }
        lines.push(Line::from(""));
        lines.push(Line::from("↑/↓ move • enter select • esc back").fg(Color::DarkGray));
        let block = Block::default().borders(Borders::ALL);
        let inner = block.inner(area);
        f.render_widget(block, area);
        f.render_widget(Paragraph::new(lines), inner);
    }
}

fn center_rect(r: Rect, width: u16, height: u16) -> Rect {
    let x = r.x + (r.width.saturating_sub(width)) / 2;
    let y = r.y + (r.height.saturating_sub(height)) / 2;
    Rect::new(x, y, width.min(r.width), height.min(r.height))
}

/// Fallback for terminals too short for the full layout.
///
/// Shows the selected assignment, help and the affected agents, and leaves
/// picker overlays to [`AppModelRender::render`].
fn render_compact(app: &AppModel<'_>, f: &mut Frame) {
    let assignment = &app.cfg[app.profile()][app.tier()];
    let selection = match app.mode {
        Mode::Main => format!("{} [{}]", assignment.model, assignment.variant),
        Mode::ModelPicker => app
            .filtered_models()
            .get(app.pick_idx)
            .cloned()
            .unwrap_or_else(|| "no models match".into()),
        Mode::VariantPicker => crate::types::VARIANTS[app.pick_idx].into(),
    };
    let help = match app.mode {
        Mode::Main => MAIN_HELP,
        Mode::ModelPicker => "type to filter • ↑/↓ move • enter select • esc back",
        Mode::VariantPicker => "↑/↓ move • enter select • esc back",
    };
    let chunks = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Min(0),
        Constraint::Length(1),
    ])
    .split(f.area());
    f.render_widget(
        Paragraph::new(format!("{} > {}: {}", app.profile(), app.tier(), selection)),
        chunks[0],
    );
    f.render_widget(Paragraph::new(help), chunks[1]);
    if chunks[2].height > 0 {
        f.render_widget(Paragraph::new(app.agent_text(chunks[2].height)), chunks[2]);
    }
    f.render_widget(
        Paragraph::new(format!("Resize for the full layout. {}", app.message)),
        chunks[3],
    );
}

fn render_full(app: &AppModel<'_>, f: &mut Frame, tiers: u16) {
    let preview = preview_lines(app);
    let available = f.area().height.saturating_sub(6 + tiers);
    // Keep room for the agent header and one agent when any are affected, so
    // the preview never crowds the list out entirely.
    let has_agent_panel = !app.selected_agents().is_empty() || app.agents_error.is_some();
    let agent_min = if has_agent_panel { 2.min(available) } else { 0 };
    let preview_h = preview
        .len()
        .min(available.saturating_sub(agent_min) as usize) as u16;
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),         // title
            Constraint::Length(1),         // profiles
            Constraint::Length(1),         // blank
            Constraint::Length(tiers),     // tiers
            Constraint::Length(1),         // blank
            Constraint::Length(preview_h), // preview (content-sized)
            Constraint::Min(0),            // affected agents (leftover space)
            Constraint::Length(1),         // message
            Constraint::Length(1),         // help
        ])
        .split(f.area());

    // Title
    let title = Line::from("opencode model switcher".bold().fg(Color::Cyan));
    f.render_widget(Paragraph::new(title), chunks[0]);

    // Profiles
    let profile_spans: Vec<Span> = app
        .profiles
        .iter()
        .enumerate()
        .flat_map(|(i, p)| {
            let s = format!(" {} ", p);
            if i == app.profile_idx {
                vec![
                    Span::raw(" "),
                    Span::styled(s, Style::default().bg(Color::Cyan).fg(Color::Black)),
                ]
            } else {
                vec![Span::raw(s)]
            }
        })
        .collect();
    f.render_widget(Paragraph::new(Line::from(profile_spans)), chunks[1]);

    // Tiers
    let tier_text = app
        .tier_order
        .iter()
        .enumerate()
        .map(|(i, tier)| {
            let assignment = app
                .cfg
                .get(app.profile())
                .and_then(|v| v.get(tier))
                .cloned()
                .expect("validated tier assignment");
            let marker = if i == app.tier_idx { "> " } else { "  " };
            if i == app.tier_idx {
                Line::from(format!(
                    "{}{:<7} {} [{}]",
                    marker, tier, assignment.model, assignment.variant
                ))
                .fg(Color::Green)
                .bold()
            } else {
                Line::from(format!(
                    "{}{:<7} {} [{}]",
                    marker, tier, assignment.model, assignment.variant
                ))
            }
        })
        .collect::<Vec<_>>();
    f.render_widget(Paragraph::new(Text::from(tier_text)), chunks[3]);

    // Preview
    f.render_widget(Paragraph::new(preview), chunks[5]);

    // Affected agents take leftover space only. One row shows the header as a
    // hint; from three rows the list gets a blank separator line above it.
    match chunks[6].height {
        0 => {}
        1 => f.render_widget(Paragraph::new(app.agent_text(1)), chunks[6]),
        2 => f.render_widget(Paragraph::new(app.agent_text(2)), chunks[6]),
        height => {
            let body = app.agent_text(height - 1);
            f.render_widget(Paragraph::new(format!("\n{body}")), chunks[6]);
        }
    }

    // Message
    if !app.message.is_empty() {
        f.render_widget(Paragraph::new(app.message.as_str()), chunks[7]);
    }

    // Help
    f.render_widget(Paragraph::new(MAIN_HELP).fg(Color::DarkGray), chunks[8]);
}

/// Preview lines for the pending apply, empty when nothing would change.
fn preview_lines(app: &AppModel<'_>) -> Vec<Line<'static>> {
    if let Some(err) = &app.apply_preview_err {
        return vec![Line::from(format!("preview failed: {err}")).fg(Color::Red)];
    }
    let Some(result) = &app.apply_preview else {
        return Vec::new();
    };
    let mut lines = vec![Line::from(format!(
        "preview: {} line(s), {} file(s) would change",
        result.lines,
        result.files.len()
    ))];
    for tier in &app.tier_order {
        if let Some(&count) = result.tiers.get(tier)
            && count > 0
            && let Some(assignment) = app.cfg.get(app.profile()).and_then(|v| v.get(tier))
        {
            lines.push(Line::from(format!(
                "  {}: {} -> {} [{}]",
                tier, count, assignment.model, assignment.variant
            )));
        }
    }
    lines
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{ApplyResult, Assignment, Env, TierSet};
    use crossterm::event::KeyCode;
    use ratatui::{Terminal, backend::TestBackend};
    use std::collections::BTreeMap;

    #[test]
    fn seven_tiers_render_and_select_in_full_and_short_terminals() {
        let env = Env {
            root: String::new(),
            tier_file: String::new(),
            agent_dirs: vec![],
        };
        let tiers = [
            "EASY",
            "MEDIUM",
            "HARD",
            "STYLE-REVIEW",
            "CORRECTNESS-REVIEW",
            "CODER",
            "WRITER",
        ]
        .map(String::from)
        .to_vec();
        let values: TierSet = tiers
            .iter()
            .enumerate()
            .map(|(i, t)| {
                (
                    t.clone(),
                    Assignment {
                        model: format!("model-{i}"),
                        variant: "low".into(),
                    },
                )
            })
            .collect();
        let mut app = AppModel {
            env: &env,
            cfg: BTreeMap::from([("normal".into(), values.clone()), ("work".into(), values)]),
            tier_order: tiers.clone(),
            profiles: vec!["normal".into(), "work".into()],
            models: vec!["chosen/model".into()],
            agents: tiers
                .iter()
                .map(|tier| (tier.clone(), vec![format!("config/agent/{tier}.md")]))
                .collect(),
            agents_error: None,
            agent_offset: 0,
            profile_idx: 0,
            tier_idx: 0,
            mode: Mode::Main,
            input: String::new(),
            pick_idx: 0,
            message: String::new(),
            should_quit: false,
            apply_preview: Some(ApplyResult {
                files: BTreeMap::from([("file".into(), 6)]),
                lines: 6,
                tiers: tiers.iter().map(|t| (t.clone(), 1)).collect(),
            }),
            apply_preview_err: None,
        };
        let render = |app: &mut AppModel, height| {
            let mut terminal = Terminal::new(TestBackend::new(120, height)).unwrap();
            terminal.draw(|f| app.render(f)).unwrap();
            let buffer = terminal.backend().buffer();
            (0..height)
                .map(|y| {
                    (0..120)
                        .map(|x| buffer[(x, y)].symbol())
                        .collect::<String>()
                })
                .collect::<Vec<_>>()
                .join("\n")
        };
        let full = render(&mut app, 40);
        for (i, tier) in tiers.iter().enumerate() {
            assert!(full.contains(tier));
            assert!(full.contains(&format!("model-{i} [low]")));
            assert!(full.contains(&format!("{tier}: 1 -> model-{i}")));
        }
        assert!(!full.contains("current marked assignments"));
        for tier in &tiers {
            let short = render(&mut app, 6);
            assert!(short.contains(&format!("> {tier}:")));
            assert!(short.contains("a apply • q quit"));
            assert!(short.contains("Resize"));
            assert!(short.contains(&format!("config/agent/{tier}.md")));
            app.handle_key(KeyCode::Down);
        }
        assert_eq!(app.tier_idx, 0);
        app.handle_key(KeyCode::Up);
        assert_eq!(app.tier(), "WRITER");

        // Leftover-space behavior: full list on an ordinary 24-row terminal,
        // header hint with one row, hidden with none.
        let ordinary = render(&mut app, 24);
        assert!(ordinary.contains("config/agent/WRITER.md"));
        assert!(ordinary.contains("a apply • q quit"));
        let hint_only = render(&mut app, 14);
        assert!(hint_only.contains("WRITER agents (1/1)"));
        assert!(!hint_only.contains("config/agent/WRITER.md"));
        let hidden = render(&mut app, 13);
        assert!(!hidden.contains("WRITER agents"));
        assert!(hidden.contains("a apply • q quit"));

        // Ordinary 80x24 terminal keeps the full layout and the agent list.
        let mut terminal = Terminal::new(TestBackend::new(80, 24)).unwrap();
        terminal.draw(|f| app.render(f)).unwrap();
        let buffer = terminal.backend().buffer();
        let standard: String = (0..24)
            .map(|y| (0..80).map(|x| buffer[(x, y)].symbol()).collect::<String>())
            .collect::<Vec<_>>()
            .join("\n");
        assert!(standard.contains("CORRECTNESS-REVIEW"));
        assert!(standard.contains("config/agent/WRITER.md"));
        assert!(standard.contains("a apply • q quit"));

        app.agents.insert(
            "WRITER".into(),
            (0..30)
                .map(|i| format!(".opencode/agent/writer-{i}.md"))
                .collect(),
        );
        app.handle_key(KeyCode::PageDown);
        assert!(render(&mut app, 6).contains("writer-1.md"));
        app.handle_key(KeyCode::End);
        assert!(render(&mut app, 6).contains("writer-29.md"));
        app.handle_key(KeyCode::PageUp);
        assert!(render(&mut app, 6).contains("writer-28.md"));
        app.handle_key(KeyCode::Home);
        assert_eq!(app.agent_offset, 0);
        app.handle_key(KeyCode::Up);
        app.handle_key(KeyCode::Down);
        assert!(render(&mut app, 6).contains("writer-0.md"));
        app.agents.clear();
        assert!(render(&mut app, 6).contains("no tagged agents"));
        app.agents_error = Some("fixture inaccessible".into());
        assert!(render(&mut app, 6).contains("agents unavailable"));
        app.handle_key(KeyCode::Enter);
        assert!(render(&mut app, 6).contains("chosen/model"));
        app.handle_key(KeyCode::Enter);
        app.handle_key(KeyCode::Char('v'));
        app.handle_key(KeyCode::Up);
        assert!(render(&mut app, 6).contains("max"));
        app.handle_key(KeyCode::Enter);
        assert_eq!(app.cfg["normal"]["WRITER"].model, "chosen/model");
        assert_eq!(app.cfg["normal"]["WRITER"].variant, "max");
        assert_eq!(app.cfg["work"]["WRITER"].model, "model-6");
        assert_eq!(app.cfg["normal"]["HARD"].model, "model-2");
    }
}
