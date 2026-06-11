use crate::config::{clone_tier_set, load_config, save_config, sorted_profiles, validate_work};
use crate::env::rel;
use crate::models::{available_models, validate_known};
use crate::rewrite::{apply_profile, build_model_line_re, current_counts};
use crate::tui::run_tui;
use crate::types::{Env, WORK_PROVIDER};
use anyhow::bail;

/// Dispatch CLI args. If no args, launch TUI.
pub fn run_cli(env: &Env, args: &[String]) -> anyhow::Result<()> {
    if args.is_empty() {
        return run_tui(env, "");
    }

    match args[0].as_str() {
        "tui" => {
            let profile = parse_optional_profile_flag("tui", &args[1..])?;
            run_tui(env, &profile)
        }
        "configure" => {
            let profile = parse_configure_args(&args[1..])?;
            run_tui(env, &profile)
        }
        "status" => cmd_status(env),
        "models" => cmd_models(env, &args[1..]),
        "apply" => cmd_apply(env, &args[1..]),
        "work" => cmd_apply(env, &[&["work".to_string()], &args[1..]].concat()),
        "set" => cmd_set(env, &args[1..]),
        "help" | "-h" | "--help" => {
            print_help(env);
            Ok(())
        }
        unknown => bail!("unknown command: {}", unknown),
    }
}

/// Accept `--profile <name>` flag or pass through empty string.
fn parse_optional_profile_flag(subcmd: &str, args: &[String]) -> anyhow::Result<String> {
    let mut profile = String::new();
    let mut i = 0;
    while i < args.len() {
        if args[i] == "--profile" {
            i += 1;
            if i >= args.len() {
                bail!("--profile requires a value");
            }
            profile = args[i].clone();
        } else if args[i].starts_with("--profile=") {
            profile = args[i].splitn(2, '=').nth(1).unwrap().to_string();
        } else {
            bail!(
                "usage: opencode-model-tiers {} [--profile normal|work]",
                subcmd
            );
        }
        i += 1;
    }
    Ok(profile)
}

fn parse_configure_args(args: &[String]) -> anyhow::Result<String> {
    let mut profile = String::new();
    let mut positional = Vec::new();
    let mut i = 0;
    while i < args.len() {
        if args[i] == "--profile" {
            i += 1;
            if i >= args.len() {
                bail!("--profile requires a value");
            }
            profile = args[i].clone();
        } else if args[i].starts_with("--profile=") {
            profile = args[i].splitn(2, '=').nth(1).unwrap().to_string();
        } else if !args[i].starts_with('-') {
            positional.push(args[i].clone());
        } else {
            bail!("unknown flag: {}", args[i]);
        }
        i += 1;
    }
    if positional.len() > 1 {
        bail!("usage: opencode-model-tiers configure [profile]");
    }
    if positional.len() == 1 {
        return Ok(positional[0].clone());
    }
    Ok(profile)
}

fn print_help(env: &Env) {
    let mut profiles = "normal, work".to_string();
    let mut tiers = "LOW, MED, HIGH".to_string();
    if let Ok(loaded) = load_config(env) {
        let mut pl: Vec<_> = loaded
            .profiles
            .keys()
            .filter(|k| !k.starts_with('$'))
            .cloned()
            .collect();
        pl.sort();
        profiles = pl.join(", ");
        tiers = loaded.tier_order.join(", ");
    }
    println!(
        "opencode-model-tiers\n\
         \n\
         Usage:\n  opencode-model-tiers                         open TUI\n  \
         opencode-model-tiers tui [--profile work]    open TUI\n  \
         opencode-model-tiers configure [profile]     open TUI\n  \
         opencode-model-tiers status                  show config and current \
         assignments\n  opencode-model-tiers models [--work]         list \
         opencode models\n  opencode-model-tiers apply <profile> [--dry-run]\n  \
         opencode-model-tiers work [--dry-run]        shortcut for apply work\n  \
         opencode-model-tiers set <profile> <tier> <model> [--no-validate]\n\
         \n\
         Profiles: {}\nTiers: {}\n\
         \n\
         TUI keys:\n  h/l or ←/→   switch profile\n  j/k or ↑/↓   switch tier/model\n  \
         enter         choose tier model\n  type          filter models in picker\n  \
         s             save tier config\n  a             apply selected profile to agent files\n  \
         q             quit\n",
        profiles, tiers
    );
}

fn cmd_status(env: &Env) -> anyhow::Result<()> {
    let loaded = load_config(env)?;
    println!("tier file: {}", rel(env, &env.tier_file));
    for profile in &sorted_profiles(&loaded.profiles) {
        println!("\n[{}]", profile);
        if let Some(values) = loaded.profiles.get(profile) {
            for tier in &loaded.tier_order {
                if let Some(model) = values.get(tier) {
                    if !model.is_empty() {
                        println!("{:<4} {}", tier, model);
                    }
                }
            }
        }
    }

    let re = build_model_line_re(&loaded.tier_order);
    let counts = current_counts(env, &loaded.tier_order, &re)?;
    for dir in &env.agent_dirs {
        println!("\ncurrent marked agent models: {}", rel(env, dir));
    }
    for tier in &loaded.tier_order {
        let total: usize = counts.get(tier).map(|m| m.values().sum()).unwrap_or(0);
        println!("{:<4} {}", tier, total);
        if let Some(models) = counts.get(tier) {
            let mut items: Vec<(usize, &String)> = models.iter().map(|(k, v)| (*v, k)).collect();
            items.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| a.1.cmp(b.1)));
            for (count, model) in items.iter().take(3) {
                println!("     {:3} {}", count, model);
            }
        }
    }
    Ok(())
}

fn cmd_models(env: &Env, args: &[String]) -> anyhow::Result<()> {
    let mut work_only = false;
    for arg in args {
        match arg.as_str() {
            "--work" => work_only = true,
            other if other.starts_with('-') => bail!("unknown flag: {}", other),
            _ => bail!("usage: opencode-model-tiers models [--work]"),
        }
    }
    let models = available_models(env)?;
    for model in &models {
        if work_only && !model.starts_with(WORK_PROVIDER) {
            continue;
        }
        println!("{}", model);
    }
    Ok(())
}

/// Accept `--dry-run` flag before or after positional args (Go `takeBoolFlag`).
/// Returns (value, positionals, unknown_flags).
fn take_bool_flag(args: &[String], name: &str) -> (bool, Vec<String>) {
    let flag = format!("--{}", name);
    let mut value = false;
    let mut positional = Vec::new();
    for arg in args {
        if arg == &flag {
            value = true;
        } else if arg.starts_with(&format!("{}=", flag)) {
            // equals form not allowed
            continue;
        } else if arg.starts_with('-') {
            // unknown flag — keep as positional (caller must reject)
            positional.push(arg.clone());
        } else {
            positional.push(arg.clone());
        }
    }
    (value, positional)
}

fn cmd_apply(env: &Env, args: &[String]) -> anyhow::Result<()> {
    let (dry_run, positional) = take_bool_flag(args, "dry-run");
    if positional.is_empty() {
        bail!("usage: opencode-model-tiers apply <profile> [--dry-run]");
    }
    // Check for unknown flags in remaining positional args
    for arg in &positional[1..] {
        if arg.starts_with('-') {
            bail!("usage: opencode-model-tiers apply <profile> [--dry-run]");
        }
    }
    let profile = &positional[0];
    let loaded = load_config(env)?;
    let values = loaded
        .profiles
        .get(profile)
        .ok_or_else(|| anyhow::anyhow!("unknown profile: {}", profile))?;

    if profile == "work" {
        validate_work(values, &loaded.tier_order)?;
    }

    let re = build_model_line_re(&loaded.tier_order);
    let result = apply_profile(env, values, dry_run, &loaded.tier_order, &re)?;

    let mut file_keys: Vec<_> = result.files.keys().collect();
    file_keys.sort();
    for path in &file_keys {
        println!(
            "{}: {}",
            rel(env, path),
            result.files.get(*path).unwrap_or(&0)
        );
    }
    let action = if dry_run { "would update" } else { "updated" };
    println!(
        "{}: {} line(s), {} file(s)",
        action,
        result.lines,
        result.files.len()
    );
    for tier in &loaded.tier_order {
        if let Some(&count) = result.tiers.get(tier) {
            if count > 0 {
                if let Some(model) = values.get(tier) {
                    println!("  {}: {} -> {}", tier, count, model);
                }
            }
        }
    }
    Ok(())
}

fn cmd_set(env: &Env, args: &[String]) -> anyhow::Result<()> {
    let (no_validate, positional) = take_bool_flag(args, "no-validate");
    if positional.len() < 3 {
        bail!("usage: opencode-model-tiers set <profile> <tier> <model> [--no-validate]");
    }
    let profile = &positional[0];
    let tier = positional[1].to_uppercase();
    let model = &positional[2];

    let mut loaded = load_config(env)?;
    if !loaded.tier_order.contains(&tier) {
        bail!("unknown tier: {}", tier);
    }
    let values = loaded
        .profiles
        .get_mut(profile)
        .ok_or_else(|| anyhow::anyhow!("unknown profile: {}", profile))?;

    if !no_validate {
        validate_known(env, &[model.clone()])?;
    }

    let mut next = clone_tier_set(values, &loaded.tier_order);
    next.insert(tier.clone(), model.clone());
    if profile == "work" {
        validate_work(&next, &loaded.tier_order)?;
    }
    loaded.profiles.insert(profile.clone(), next);
    save_config(env, &loaded)?;

    println!("updated: {}", rel(env, &env.tier_file));
    if let Some(final_values) = loaded.profiles.get(profile) {
        for t in &loaded.tier_order {
            if let Some(m) = final_values.get(t) {
                println!("{:<4} {}", t, m);
            }
        }
    }
    Ok(())
}
