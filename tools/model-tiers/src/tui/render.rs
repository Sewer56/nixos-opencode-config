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
}

impl<'a> AppModelRender for AppModel<'a> {
    fn render(&mut self, f: &mut Frame) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),                                // title
                Constraint::Length(1),                                // profiles
                Constraint::Length(1),                                // blank
                Constraint::Length(self.tier_order.len() as u16 + 1), // tiers
                Constraint::Length(1),                                // blank
                Constraint::Length(3),                                // preview
                Constraint::Length(4),                                // counts
                Constraint::Length(1),                                // blank
                Constraint::Length(1),                                // message
                Constraint::Length(1),                                // help
            ])
            .split(f.area());

        // Title
        let title = Line::from("opencode model tiers".bold().fg(Color::Cyan));
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
                let model = self
                    .cfg
                    .get(self.profile())
                    .and_then(|v| v.get(tier))
                    .cloned()
                    .unwrap_or_else(|| "<unmapped>".into());
                let marker = if i == self.tier_idx { "> " } else { "  " };
                if i == self.tier_idx {
                    Line::from(format!("{}{:<4} {}", marker, tier, model))
                        .fg(Color::Green)
                        .bold()
                } else {
                    Line::from(format!("{}{:<4} {}", marker, tier, model))
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
                    && let Some(model) = self.cfg.get(self.profile()).and_then(|v| v.get(tier))
                {
                    lines.push(format!("  {}: {} -> {}", tier, count, model));
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

        // Message
        if !self.message.is_empty() {
            f.render_widget(Paragraph::new(self.message.as_str()), chunks[8]);
        }

        // Help
        f.render_widget(
            Paragraph::new("←/→ profile • ↑/↓ tier • enter choose • s save • a apply • q quit")
                .fg(Color::DarkGray),
            chunks[9],
        );

        // Picker overlay
        if matches!(self.mode, Mode::Picker) {
            self.render_picker(f);
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
}

fn center_rect(r: Rect, width: u16, height: u16) -> Rect {
    let x = r.x + (r.width.saturating_sub(width)) / 2;
    let y = r.y + (r.height.saturating_sub(height)) / 2;
    Rect::new(x, y, width.min(r.width), height.min(r.height))
}
