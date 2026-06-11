use crate::config::{load_config, save_config, sorted_profiles, validate_work};
use crate::env::rel;
use crate::models::{available_models, filter_models};
use crate::rewrite::{apply_profile, build_model_line_re, current_counts};
use crate::types::{ApplyResult, Config, Env};
use anyhow::bail;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph},
};
use std::collections::BTreeMap;

pub fn run_tui(env: &Env, initial_profile: &str) -> anyhow::Result<()> {
    if !std::io::IsTerminal::is_terminal(&std::io::stdout()) {
        bail!("not a terminal");
    }
    let mut terminal = ratatui::init();
    let result = run_app(&mut terminal, env, initial_profile);
    ratatui::restore();
    result
}

fn run_app(terminal: &mut DefaultTerminal, env: &Env, initial_profile: &str) -> anyhow::Result<()> {
    let mut app = AppModel::new(env, initial_profile)?;
    while !app.should_quit {
        terminal.draw(|f| app.render(f))?;
        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                app.handle_key(key.code);
            }
        }
    }
    Ok(())
}

enum Mode {
    Main,
    Picker,
}

struct AppModel<'a> {
    env: &'a Env,
    cfg: Config,
    tier_order: Vec<String>,
    profiles: Vec<String>,
    models: Vec<String>,
    counts: BTreeMap<String, BTreeMap<String, usize>>,
    profile_idx: usize,
    tier_idx: usize,
    mode: Mode,
    input: String,
    pick_idx: usize,
    message: String,
    should_quit: bool,
    apply_preview: Option<ApplyResult>,
    apply_preview_err: Option<String>,
}

impl<'a> AppModel<'a> {
    fn new(env: &'a Env, initial_profile: &str) -> anyhow::Result<Self> {
        let loaded = load_config(env)?;
        let models = available_models(env)?;
        let re = build_model_line_re(&loaded.tier_order);
        let counts = current_counts(env, &loaded.tier_order, &re)?;
        let profiles = sorted_profiles(&loaded.profiles);
        if profiles.is_empty() {
            bail!("no profiles configured");
        }

        let profile_idx = if initial_profile.is_empty() {
            0
        } else {
            profiles
                .iter()
                .position(|p| p == initial_profile)
                .ok_or_else(|| anyhow::anyhow!("unknown profile: {}", initial_profile))?
        };

        Ok(Self {
            env,
            cfg: loaded.profiles,
            tier_order: loaded.tier_order,
            profiles,
            models,
            counts,
            profile_idx,
            tier_idx: 0,
            mode: Mode::Main,
            input: String::new(),
            pick_idx: 0,
            message: String::new(),
            should_quit: false,
            apply_preview: None,
            apply_preview_err: None,
        })
    }

    fn profile(&self) -> &str {
        &self.profiles[self.profile_idx]
    }

    fn tier(&self) -> &str {
        &self.tier_order[self.tier_idx]
    }

    fn current_model(&self) -> String {
        self.cfg
            .get(self.profile())
            .and_then(|v| v.get(self.tier()))
            .cloned()
            .unwrap_or_else(|| "<unmapped>".to_string())
    }

    fn handle_key(&mut self, code: KeyCode) {
        match self.mode {
            Mode::Main => self.handle_main_key(code),
            Mode::Picker => self.handle_picker_key(code),
        }
    }

    fn handle_main_key(&mut self, code: KeyCode) {
        match code {
            KeyCode::Char('q') | KeyCode::Esc => self.should_quit = true,
            KeyCode::Char('h') | KeyCode::Left => {
                self.profile_idx =
                    (self.profile_idx + self.profiles.len() - 1) % self.profiles.len();
                self.update_preview();
            }
            KeyCode::Char('l') | KeyCode::Right => {
                self.profile_idx = (self.profile_idx + 1) % self.profiles.len();
                self.update_preview();
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.tier_idx = (self.tier_idx + self.tier_order.len() - 1) % self.tier_order.len();
            }
            KeyCode::Char('j') | KeyCode::Down => {
                self.tier_idx = (self.tier_idx + 1) % self.tier_order.len();
            }
            KeyCode::Enter | KeyCode::Char(' ') => {
                self.mode = Mode::Picker;
                self.pick_idx = 0;
                self.input.clear();
                self.message.clear();
            }
            KeyCode::Char('s') => {
                let profile = self.profile().to_string();
                if profile == "work" {
                    if let Some(values) = self.cfg.get("work") {
                        if let Err(e) = validate_work(values, &self.tier_order) {
                            self.message = format!("save failed: {}", e);
                            return;
                        }
                    }
                }
                match save_config(
                    self.env,
                    &crate::types::LoadedConfig {
                        tier_order: self.tier_order.clone(),
                        profiles: self.cfg.clone(),
                    },
                ) {
                    Ok(()) => {
                        self.message = format!("saved {}", rel(self.env, &self.env.tier_file))
                    }
                    Err(e) => self.message = format!("save failed: {}", e),
                }
            }
            KeyCode::Char('a') => {
                let profile = self.profile().to_string();
                let values = match self.cfg.get(&profile).cloned() {
                    Some(v) => v,
                    None => {
                        self.message = format!("no config for profile: {}", profile);
                        return;
                    }
                };
                if profile == "work" {
                    if let Err(e) = validate_work(&values, &self.tier_order) {
                        self.message = format!("apply failed: {}", e);
                        return;
                    }
                }
                let re = build_model_line_re(&self.tier_order);
                match apply_profile(self.env, &values, false, &self.tier_order, &re) {
                    Ok(result) => {
                        self.message = format!(
                            "applied {}: {} line(s), {} file(s)",
                            profile,
                            result.lines,
                            result.files.len()
                        );
                        // Refresh counts after apply
                        if let Ok(c) = current_counts(self.env, &self.tier_order, &re) {
                            self.counts = c;
                        }
                        self.update_preview();
                    }
                    Err(e) => self.message = format!("apply failed: {}", e),
                }
            }
            _ => {}
        }
    }

    fn handle_picker_key(&mut self, code: KeyCode) {
        let filtered = self.filtered_models();
        match code {
            KeyCode::Esc => {
                self.mode = Mode::Main;
            }
            KeyCode::Enter => {
                if let Some(model) = filtered.get(self.pick_idx) {
                    let profile = self.profile().to_string();
                    let tier = self.tier().to_string();
                    self.cfg
                        .entry(profile)
                        .or_default()
                        .insert(tier, model.clone());
                    self.update_preview();
                }
                self.mode = Mode::Main;
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if !filtered.is_empty() {
                    self.pick_idx = (self.pick_idx + filtered.len() - 1) % filtered.len();
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if !filtered.is_empty() {
                    self.pick_idx = (self.pick_idx + 1) % filtered.len();
                }
            }
            KeyCode::Char(c) => {
                self.input.push(c);
                self.pick_idx = 0;
            }
            KeyCode::Backspace => {
                self.input.pop();
                self.pick_idx = 0;
            }
            _ => {}
        }
        // Clamp pick index after filtering
        let filtered = self.filtered_models();
        if self.pick_idx >= filtered.len() {
            self.pick_idx = filtered.len().saturating_sub(1);
        }
    }

    fn filtered_models(&self) -> Vec<String> {
        filter_models(self.profile(), &self.models, &self.input)
    }

    fn update_preview(&mut self) {
        let re = build_model_line_re(&self.tier_order);
        let values = match self.cfg.get(self.profile()).cloned() {
            Some(v) => v,
            None => {
                self.apply_preview = None;
                self.apply_preview_err = Some("no config".into());
                return;
            }
        };
        match apply_profile(self.env, &values, true, &self.tier_order, &re) {
            Ok(result) => {
                self.apply_preview = Some(result);
                self.apply_preview_err = None;
            }
            Err(e) => {
                self.apply_preview = None;
                self.apply_preview_err = Some(e.to_string());
            }
        }
    }

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
                let model = self.current_model();
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
                if let Some(&count) = result.tiers.get(tier) {
                    if count > 0 {
                        if let Some(model) = self.cfg.get(self.profile()).and_then(|v| v.get(tier))
                        {
                            lines.push(format!("  {}: {} -> {}", tier, count, model));
                        }
                    }
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
            Paragraph::new("h/l profile • j/k tier • enter choose • s save • a apply • q quit")
                .fg(Color::DarkGray),
            chunks[9],
        );

        // Picker overlay
        if matches!(self.mode, Mode::Picker) {
            self.render_picker(f);
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
            for i in start..end {
                let model = &filtered[i];
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
            Line::from("type filter • ↑/↓ move • enter select • esc back").fg(Color::DarkGray),
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
