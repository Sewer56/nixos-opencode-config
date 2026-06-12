use crate::types::{Env, WORK_PROVIDER};
use anyhow::{Context, bail};
use std::process::Command;

/// Shell out to `opencode models` to discover available models.
pub fn available_models(env: &Env) -> anyhow::Result<Vec<String>> {
    let output = Command::new("opencode")
        .arg("models")
        .current_dir(&env.root)
        .output()
        .context("run 'opencode models'")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("opencode models failed: {}", stderr.trim());
    }

    parse_models_output(&String::from_utf8_lossy(&output.stdout))
}

/// Parse `opencode models` output: one `provider/model` per line.
pub fn parse_models_output(output: &str) -> anyhow::Result<Vec<String>> {
    let mut seen = std::collections::HashSet::new();
    let mut models = Vec::new();
    for line in output.lines() {
        let model = line.trim();
        if model.is_empty() || !model.contains('/') || seen.contains(model) {
            continue;
        }
        seen.insert(model.to_string());
        models.push(model.to_string());
    }
    if models.is_empty() {
        bail!("opencode models returned no models");
    }
    Ok(models)
}

/// Filter models by case-insensitive space-separated token query.
/// Work profile additionally requires `sewer-axonhub-work/` prefix.
pub fn filter_models(profile: &str, models: &[String], query: &str) -> Vec<String> {
    let query = query.trim().to_lowercase();
    let tokens: Vec<&str> = query.split_whitespace().collect();
    let mut out = Vec::new();
    for model in models {
        if profile == "work" && !model.starts_with(WORK_PROVIDER) {
            continue;
        }
        let lower = model.to_lowercase();
        if tokens.iter().all(|t| lower.contains(t)) {
            out.push(model.clone());
        }
    }
    out
}
