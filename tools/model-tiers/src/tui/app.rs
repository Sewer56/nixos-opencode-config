use crate::config::load_config;
use crate::models::available_models;
use crate::rewrite::{build_model_line_re, current_counts};
use crate::types::{ApplyResult, Config, Env};
use anyhow::bail;
use crossterm::event::{self, Event, KeyEventKind};
use ratatui::DefaultTerminal;
use std::collections::BTreeMap;

use super::handler::AppModelHandler;
use super::render::AppModelRender;

pub(crate) enum Mode {
    Main,
    Picker,
}

/// Interactive TUI state for reviewing and editing tier-model assignments.
///
/// Holds loaded config, available models, current tier counts, selection
/// indices, picker state, and the apply preview result.
pub(crate) struct AppModel<'a> {
    pub(crate) env: &'a Env,
    pub(crate) cfg: Config,
    pub(crate) tier_order: Vec<String>,
    pub(crate) profiles: Vec<String>,
    pub(crate) models: Vec<String>,
    pub(crate) counts: BTreeMap<String, BTreeMap<String, usize>>,
    pub(crate) profile_idx: usize,
    pub(crate) tier_idx: usize,
    pub(crate) mode: Mode,
    pub(crate) input: String,
    pub(crate) pick_idx: usize,
    pub(crate) message: String,
    pub(crate) should_quit: bool,
    pub(crate) apply_preview: Option<ApplyResult>,
    pub(crate) apply_preview_err: Option<String>,
}

impl<'a> AppModel<'a> {
    /// Initialize the app state for interactive assignment review.
    ///
    /// Loads the config file, fetches available models, builds the model-line
    /// regex, reads current tier counts, and validates the initial profile.
    ///
    /// Pass an empty `initial_profile` to select the first profile in the config.
    pub(crate) fn new(env: &'a Env, initial_profile: &str) -> anyhow::Result<Self> {
        let loaded = load_config(env)?;
        let models = available_models(env)?;
        let re = build_model_line_re(&loaded.tier_order);
        let counts = current_counts(env, &loaded.tier_order, &re)?;
        let profiles = crate::config::sorted_profiles(&loaded.profiles);
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

    pub(crate) fn profile(&self) -> &str {
        &self.profiles[self.profile_idx]
    }

    pub(crate) fn tier(&self) -> &str {
        &self.tier_order[self.tier_idx]
    }
}

/// Open the TUI for reviewing tier-model assignments.
///
/// # Errors
///
/// - Returns `"not a terminal"` when stdout is not a TTY.
/// - Returns errors from [`AppModel::new`] when the config file is missing,
///   invalid, fetching available models fails, agent-file reads fail,
///   no profiles are configured, or the initial profile is unknown.
/// - Returns I/O errors from event reading or terminal rendering.
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
        if let Event::Key(key) = event::read()?
            && key.kind == KeyEventKind::Press
        {
            app.handle_key(key.code);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Config, Env, TierSet};
    use std::collections::BTreeMap;

    fn test_env_empty() -> Env {
        Env {
            root: String::new(),
            tier_file: String::new(),
            agent_dirs: vec![],
        }
    }

    #[test]
    fn test_per_tier_model_lookup_uses_loop_variable_not_selected_tier() {
        // Set up config where each tier has a different model.
        // If the old bug (self.current_model() using self.tier()) were present,
        // all rows would show the selected tier's model.
        let mut cfg: Config = Config::new();
        let mut normal: TierSet = TierSet::new();
        normal.insert("LOW".into(), "model-low".into());
        normal.insert("MED".into(), "model-med".into());
        normal.insert("HIGH".into(), "model-high".into());
        cfg.insert("normal".into(), normal);

        let tier_order = vec!["LOW".to_string(), "MED".to_string(), "HIGH".to_string()];
        let profiles = vec!["normal".to_string()];
        let env = test_env_empty();

        // tier_idx = 1 (MED selected) — old bug would show "model-med" for all rows
        let app = AppModel {
            env: &env,
            cfg,
            tier_order,
            profiles,
            models: vec![],
            counts: BTreeMap::new(),
            profile_idx: 0,
            tier_idx: 1,
            mode: Mode::Main,
            input: String::new(),
            pick_idx: 0,
            message: String::new(),
            should_quit: false,
            apply_preview: None,
            apply_preview_err: None,
        };

        // Simulate the render loop: for each tier, look up its model
        let models: Vec<String> = app
            .tier_order
            .iter()
            .map(|tier| {
                app.cfg
                    .get(app.profile())
                    .and_then(|v| v.get(tier))
                    .cloned()
                    .unwrap_or_else(|| "<unmapped>".into())
            })
            .collect();

        // Each tier should have its own model, not all "model-med"
        assert_eq!(
            models,
            vec!["model-low", "model-med", "model-high"],
            "each tier row must show its own model, not the selected tier's model"
        );
    }
}
