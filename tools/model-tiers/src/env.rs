use crate::types::Env;
use anyhow::{Context, bail};
use std::path::Path;

/// Walk upward from CWD until we find a directory containing both
/// `config/model-tiers.json` and at least one agent directory
/// (`config/agent` or `.opencode/agent`).
pub fn find_env() -> anyhow::Result<Env> {
    let cwd = std::env::current_dir().context("get current directory")?;
    let mut dir = cwd.as_path();

    loop {
        let tier_file = dir.join("config").join("model-tiers.json");
        let mut agent_dirs = Vec::new();

        for candidate in &[
            dir.join("config").join("agent"),
            dir.join(".opencode").join("agent"),
        ] {
            if candidate.is_dir() {
                agent_dirs.push(candidate.to_string_lossy().into_owned());
            }
        }

        if tier_file.is_file() && !agent_dirs.is_empty() {
            return Ok(Env {
                root: dir.to_string_lossy().into_owned(),
                tier_file: tier_file.to_string_lossy().into_owned(),
                agent_dirs,
            });
        }

        match dir.parent() {
            Some(parent) => dir = parent,
            None => break,
        }
    }

    bail!(
        "could not find repo root from {} (need config/model-tiers.json and config/agent or .opencode/agent)",
        cwd.display()
    )
}

/// Return path relative to env.root, falling back to the full path on error.
pub fn rel(env: &Env, path: &str) -> String {
    Path::new(path)
        .strip_prefix(&env.root)
        .map(|p| p.to_string_lossy().into_owned())
        .unwrap_or_else(|_| path.to_string())
}
