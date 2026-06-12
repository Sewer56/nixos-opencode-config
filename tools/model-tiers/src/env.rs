use crate::types::Env;
use anyhow::Context;

/// OpenCode config directory. Matches the TypeScript core which uses `xdg-basedir`
/// (always resolves to `$XDG_CONFIG_HOME/opencode` or `~/.config/opencode` on all platforms).
pub fn opencode_config_dir() -> String {
    if let Ok(d) = std::env::var("XDG_CONFIG_HOME") {
        if !d.is_empty() {
            return format!("{d}/opencode");
        }
    }
    let home = std::env::var("HOME").unwrap_or_default();
    format!("{home}/.config/opencode")
}

/// Walk upward from CWD to find agent directories (`config/agent` or
/// `.opencode/agent`). The tier config file lives in the OpenCode config
/// directory returned by [`opencode_config_dir`].
pub fn find_env() -> anyhow::Result<Env> {
    let cwd = std::env::current_dir().context("get current directory")?;
    let tier_file = format!("{}/model-tiers.json", opencode_config_dir());
    let mut dir = cwd.as_path();

    loop {
        let mut agent_dirs = Vec::new();

        for candidate in &[
            dir.join("config").join("agent"),
            dir.join(".opencode").join("agent"),
        ] {
            if candidate.is_dir() {
                agent_dirs.push(candidate.to_string_lossy().into_owned());
            }
        }

        if !agent_dirs.is_empty() {
            return Ok(Env {
                root: dir.to_string_lossy().into_owned(),
                tier_file,
                agent_dirs,
            });
        }

        match dir.parent() {
            Some(parent) => dir = parent,
            None => break,
        }
    }

    // No agent dirs found - still usable for config editing.
    Ok(Env {
        root: cwd.to_string_lossy().into_owned(),
        tier_file,
        agent_dirs: Vec::new(),
    })
}

/// Return path relative to env.root, falling back to the full path on error.
pub fn rel(env: &Env, path: &str) -> String {
    std::path::Path::new(path)
        .strip_prefix(&env.root)
        .map(|p| p.to_string_lossy().into_owned())
        .unwrap_or_else(|_| path.to_string())
}
