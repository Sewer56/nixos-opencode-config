use super::app::{AppModel, Mode};
use crate::config::validate_work;
use crate::env::rel;
use crate::models::filter_models;
use crate::rewrite::{apply_profile, build_model_line_re, current_counts};
use crossterm::event::KeyCode;

pub(crate) trait AppModelHandler {
    fn handle_key(&mut self, code: KeyCode);
    fn handle_main_key(&mut self, code: KeyCode);
    fn handle_picker_key(&mut self, code: KeyCode);
    fn filtered_models(&self) -> Vec<String>;
    fn update_preview(&mut self);
}

impl<'a> AppModelHandler for AppModel<'a> {
    fn handle_key(&mut self, code: KeyCode) {
        match self.mode {
            Mode::Main => self.handle_main_key(code),
            Mode::Picker => self.handle_picker_key(code),
        }
    }

    fn handle_main_key(&mut self, code: KeyCode) {
        match code {
            KeyCode::Char('q') | KeyCode::Esc => self.should_quit = true,
            KeyCode::Left => {
                self.profile_idx =
                    (self.profile_idx + self.profiles.len() - 1) % self.profiles.len();
                self.update_preview();
            }
            KeyCode::Right => {
                self.profile_idx = (self.profile_idx + 1) % self.profiles.len();
                self.update_preview();
            }
            KeyCode::Up => {
                self.tier_idx = (self.tier_idx + self.tier_order.len() - 1) % self.tier_order.len();
            }
            KeyCode::Down => {
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
                if profile == "work"
                    && let Some(values) = self.cfg.get("work")
                    && let Err(e) = validate_work(values, &self.tier_order)
                {
                    self.message = format!("save failed: {}", e);
                    return;
                }
                match crate::config::save_config(
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
                if profile == "work"
                    && let Err(e) = validate_work(&values, &self.tier_order)
                {
                    self.message = format!("apply failed: {}", e);
                    return;
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
            KeyCode::Up => {
                if !filtered.is_empty() {
                    self.pick_idx = (self.pick_idx + filtered.len() - 1) % filtered.len();
                }
            }
            KeyCode::Down => {
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
}
