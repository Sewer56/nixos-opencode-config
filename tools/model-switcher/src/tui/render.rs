use super::app::{AppModel, Mode};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph},
};

pub(crate) trait AppModelRender {
    fn render(&mut self, f: &mut Frame);
    fn render_picker(&self, f: &mut Frame);
    fn render_variant_picker(&self, f: &mut Frame);
}

const MAIN_HELP: &str = "←/→ profile • ↑/↓ tier • PgUp/PgDn agents • enter model • v variant • s save • a apply • q quit";

impl AppModel<'_> {
    fn agent_text(&self, height: u16) -> String {
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
        let rows = self.tier_order.len() as u16 + 1;
        if f.area().height < rows.saturating_mul(3).saturating_add(11) {
            let assignment = &self.cfg[self.profile()][self.tier()];
            let selection = match self.mode {
                Mode::Main => format!("{} [{}]", assignment.model, assignment.variant),
                Mode::ModelPicker => {
                    use super::handler::AppModelHandler;
                    self.filtered_models()
                        .get(self.pick_idx)
                        .cloned()
                        .unwrap_or_else(|| "no models match".into())
                }
                Mode::VariantPicker => crate::types::VARIANTS[self.pick_idx].into(),
            };
            let help = match self.mode {
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
                Paragraph::new(format!(
                    "{} > {}: {}",
                    self.profile(),
                    self.tier(),
                    selection
                )),
                chunks[0],
            );
            f.render_widget(Paragraph::new(help), chunks[1]);
            f.render_widget(Paragraph::new(self.agent_text(chunks[2].height)), chunks[2]);
            f.render_widget(
                Paragraph::new(format!(
                    "Resize for all tiers/preview/counts. {}",
                    self.message
                )),
                chunks[3],
            );
            return;
        }
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),    // title
                Constraint::Length(1),    // profiles
                Constraint::Length(1),    // blank
                Constraint::Length(rows), // tiers
                Constraint::Length(1),    // blank
                Constraint::Length(rows), // preview
                Constraint::Length(rows), // counts
                Constraint::Min(4),       // affected agents
                Constraint::Length(1),    // message
                Constraint::Length(1),    // help
            ])
            .split(f.area());

        // Title
        let title = Line::from("opencode model switcher".bold().fg(Color::Cyan));
        f.render_widget(Paragraph::new(title), chunks[0]);

        // Profiles
        let profile_spans: Vec<Span> = self
            .profiles
            .iter()
            .enumerate()
            .flat_map(|(i, p)| {
                let s = format!(" {} ", p);
                if i == self.profile_idx {
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
        let tier_text = self
            .tier_order
            .iter()
            .enumerate()
            .map(|(i, tier)| {
                let assignment = self
                    .cfg
                    .get(self.profile())
                    .and_then(|v| v.get(tier))
                    .cloned()
                    .expect("validated tier assignment");
                let marker = if i == self.tier_idx { "> " } else { "  " };
                if i == self.tier_idx {
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
        if let Some(ref err) = self.apply_preview_err {
            f.render_widget(
                Paragraph::new(format!("preview failed: {}", err)).fg(Color::Red),
                chunks[5],
            );
        } else if let Some(ref result) = self.apply_preview {
            let mut lines = vec![format!(
                "preview: {} line(s), {} file(s) would change",
                result.lines,
                result.files.len()
            )];
            for tier in &self.tier_order {
                if let Some(&count) = result.tiers.get(tier)
                    && count > 0
                    && let Some(assignment) = self.cfg.get(self.profile()).and_then(|v| v.get(tier))
                {
                    lines.push(format!(
                        "  {}: {} -> {} [{}]",
                        tier, count, assignment.model, assignment.variant
                    ));
                }
            }
            f.render_widget(Paragraph::new(lines.join("\n")), chunks[5]);
        }

        // Counts
        let mut count_lines = vec!["current marked assignments:".to_string()];
        for tier in &self.tier_order {
            let total: usize = self.counts.get(tier).map(|m| m.values().sum()).unwrap_or(0);
            let mut line = format!("  {:<4} {}", tier, total);
            if let Some(models) = self.counts.get(tier) {
                let mut items: Vec<(usize, &String)> =
                    models.iter().map(|(k, v)| (*v, k)).collect();
                items.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| a.1.cmp(b.1)));
                if !items.is_empty() {
                    line.push_str("  ");
                    let parts: Vec<String> = items
                        .iter()
                        .take(2)
                        .map(|(count, model)| format!("{}×{}", count, model))
                        .collect();
                    line.push_str(&parts.join(", "));
                }
            }
            count_lines.push(line);
        }
        f.render_widget(Paragraph::new(count_lines.join("\n")), chunks[6]);

        f.render_widget(Paragraph::new(self.agent_text(chunks[7].height)), chunks[7]);

        // Message
        if !self.message.is_empty() {
            f.render_widget(Paragraph::new(self.message.as_str()), chunks[8]);
        }

        // Help
        f.render_widget(Paragraph::new(MAIN_HELP).fg(Color::DarkGray), chunks[9]);

        // Picker overlay
        if matches!(self.mode, Mode::ModelPicker) {
            self.render_picker(f);
        }
        if matches!(self.mode, Mode::VariantPicker) {
            self.render_variant_picker(f);
        }
    }

    fn render_picker(&self, f: &mut Frame) {
        use super::handler::AppModelHandler;
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tui::handler::AppModelHandler;
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
            counts: tiers
                .iter()
                .map(|t| (t.clone(), BTreeMap::from([("old".into(), 2)])))
                .collect(),
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
            assert!(full.contains(&format!("{tier}")));
            assert!(full.contains(&format!("model-{i} [low]")));
            assert!(full.contains(&format!("{tier}: 1 -> model-{i}")));
            assert!(full.contains(&format!("{tier:<4} 2")));
        }
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
