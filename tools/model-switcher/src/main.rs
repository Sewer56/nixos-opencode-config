mod config;
mod env;
mod models;
mod rewrite;

mod tui;
mod types;

use anyhow::{bail, Result};

fn main() -> Result<()> {
    let env = env::find_env()?;
    let args: Vec<String> = std::env::args().skip(1).collect();
    let profile = parse_profile(&env, &args)?;
    tui::run_tui(&env, &profile)
}

/// Parse and validate the optional profile argument.
///
/// Returns the profile name, or an empty string when no argument is given.
/// Errors if more than one argument is provided, or if the given profile name
/// does not match any profile in the loaded config. The error message lists
/// available profiles on a second line for display in the terminal.
fn parse_profile(env: &types::Env, args: &[String]) -> anyhow::Result<String> {
    if args.len() > 1 {
        bail!("usage: opencode-model-switcher [profile]");
    }
    let name = args.first().map(|s| s.as_str()).unwrap_or("");
    if name.is_empty() {
        return Ok(String::new());
    }
    let loaded = config::load_config(env)?;
    let profiles = config::sorted_profiles(&loaded.profiles);
    if !profiles.contains(&name.to_string()) {
        bail!("unknown profile: {}\navailable: {}", name, profiles.join(", "));
    }
    Ok(name.to_string())
}
