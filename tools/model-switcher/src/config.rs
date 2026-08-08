use crate::env::rel;
use crate::rewrite::agent_files;
use crate::types::{Config, Env, LoadedConfig, TierSet, VARIANTS, WORK_PROVIDER};
use anyhow::{Context, bail};
use serde::Deserialize;

#[derive(Deserialize)]
struct ConfigFile {
    #[serde(rename = "$tierOrder", default)]
    tier_order: std::collections::BTreeMap<String, String>,
    #[serde(flatten)]
    profiles: Config,
}

/// Load and validate model-switcher.json from env.tier_file.
pub fn load_config(env: &Env) -> anyhow::Result<LoadedConfig> {
    let data = std::fs::read_to_string(&env.tier_file)
        .with_context(|| format!("read tier file: {}", rel(env, &env.tier_file)))?;
    let file: ConfigFile = serde_json::from_str(&data).with_context(|| "parse tier config")?;
    let tier_order = derive_tier_order(env, &file.profiles, &file.tier_order);
    validate_config(&file.profiles, &tier_order)?;
    Ok(LoadedConfig {
        tier_order,
        profiles: file.profiles,
    })
}

/// Save config atomically: write to .tmp, then rename.
pub fn save_config(env: &Env, loaded: &LoadedConfig) -> anyhow::Result<()> {
    validate_config(&loaded.profiles, &loaded.tier_order)?;
    let data = marshal_config(&loaded.profiles, &loaded.tier_order);
    if let Some(parent) = std::path::Path::new(&env.tier_file).parent() {
        std::fs::create_dir_all(parent).context("create config dir")?;
    }
    let tmp = format!("{}.tmp", env.tier_file);
    std::fs::write(&tmp, &data).context("write tier file tmp")?;
    std::fs::rename(&tmp, &env.tier_file).context("rename tier file")?;
    Ok(())
}

/// Work profile must only use work-provider models.
pub fn validate_work(values: &TierSet, tier_order: &[String]) -> anyhow::Result<()> {
    let mut bad = Vec::new();
    for tier in tier_order {
        if let Some(assignment) = values.get(tier)
            && !assignment.model.starts_with(WORK_PROVIDER)
        {
            bad.push(format!("{}={}", tier, assignment.model));
        }
    }
    if !bad.is_empty() {
        bail!(
            "work profile must use {} models: {}",
            WORK_PROVIDER,
            bad.join(", ")
        );
    }
    Ok(())
}

/// Derive canonical tier order from: `$tierOrder` in config, then profile tier keys,
/// then tier names discovered in agent markdown files.
pub fn derive_tier_order(
    env: &Env,
    cfg: &Config,
    configured: &std::collections::BTreeMap<String, String>,
) -> Vec<String> {
    let mut seen = std::collections::HashSet::new();
    let mut result = Vec::new();

    // 1. Parse numeric $tierOrder keys
    let mut indexed: Vec<(usize, String)> = Vec::new();
    for (k, v) in configured {
        if let Ok(idx) = k.parse::<usize>()
            && !v.is_empty()
        {
            indexed.push((idx, v.clone()));
        }
    }
    indexed.sort_by_key(|(i, _)| *i);
    for (_, tier) in indexed {
        if seen.insert(tier.clone()) {
            result.push(tier);
        }
    }

    // 2. Collect tier keys from all non-$ profiles
    let mut profile_tiers: Vec<String> = Vec::new();
    for values in cfg.values() {
        for tier in values.keys() {
            if !seen.contains(tier) {
                profile_tiers.push(tier.clone());
            }
        }
    }
    profile_tiers.sort();
    for tier in &profile_tiers {
        seen.insert(tier.clone());
    }
    result.extend(profile_tiers);

    // 3. Discover tiers from agent files (best-effort)
    if let Ok(tiers) = discover_tiers_from_files(env) {
        let mut extra: Vec<String> = tiers.into_iter().filter(|t| !seen.contains(t)).collect();
        extra.sort();
        result.extend(extra);
    }

    result
}

/// Marshal config to JSON preserving tier and profile order.
pub fn marshal_config(cfg: &Config, tier_order: &[String]) -> String {
    let mut out = String::from("{\n");

    // $tierOrder as first key
    out.push_str("  \"$tierOrder\": {");
    for (i, tier) in tier_order.iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        out.push_str(&format!("\"{}\": \"{}\"", i, tier));
    }
    let profiles = sorted_profiles(cfg);
    if profiles.is_empty() {
        out.push_str("}\n");
    } else {
        out.push_str("},\n");
    }

    for (pi, profile) in profiles.iter().enumerate() {
        let profile_json = serde_json::to_string(profile).unwrap();
        out.push_str(&format!("  {}: {{\n", profile_json));
        if let Some(values) = cfg.get(profile) {
            for (ti, tier) in tier_order.iter().enumerate() {
                let assignment = values.get(tier).expect("validated tier assignment");
                let assignment_json = serde_json::to_string(assignment).unwrap();
                out.push_str(&format!("    \"{}\": {}", tier, assignment_json));
                if ti != tier_order.len() - 1 {
                    out.push(',');
                }
                out.push('\n');
            }
        }
        out.push_str("  }");
        if pi != profiles.len() - 1 {
            out.push(',');
        }
        out.push('\n');
    }
    out.push_str("}\n");
    out
}

/// Validate that all profiles have identical tier key sets and non-empty values.
pub fn validate_config(cfg: &Config, _tier_order: &[String]) -> anyhow::Result<()> {
    if profile_names(cfg).is_empty() {
        bail!("tier config must contain at least one profile");
    }

    let mut first_keys: Option<std::collections::HashSet<String>> = None;
    let mut first_profile = "";

    for (profile, values) in cfg {
        let keys: std::collections::HashSet<String> = values.keys().cloned().collect();
        if let Some(ref first) = first_keys {
            if keys != *first {
                bail!(
                    "profile {:?} tier keys differ from {:?}",
                    profile,
                    first_profile
                );
            }
        } else {
            first_keys = Some(keys);
            first_profile = profile;
        }
        for (tier, assignment) in values {
            if assignment.model.trim().is_empty() {
                bail!("profile {:?} has empty model for {}", profile, tier);
            }
            if !VARIANTS.contains(&assignment.variant.as_str()) {
                bail!(
                    "profile {:?} has invalid variant {:?} for {}",
                    profile,
                    assignment.variant,
                    tier
                );
            }
        }
    }
    Ok(())
}

/// Return profile names (non-`$`-prefixed keys) from the config, sorted alphabetically.
pub fn sorted_profiles(cfg: &Config) -> Vec<String> {
    let mut names: Vec<String> = cfg.keys().cloned().collect();
    names.sort();
    names
}

/// Scan agent markdown files for `# <TIER>` tags.
fn discover_tiers_from_files(env: &Env) -> anyhow::Result<Vec<String>> {
    let files = agent_files(env)?;
    let mut seen = std::collections::HashSet::new();
    for file in &files {
        let data = std::fs::read_to_string(file)?;
        for line in data.lines() {
            if let Some(caps) = crate::rewrite::MODEL_LINE_DISCOVERY_RE.captures(line) {
                seen.insert(caps.get(1).unwrap().as_str().to_string());
            }
        }
    }
    let mut result: Vec<String> = seen.into_iter().collect();
    result.sort();
    Ok(result)
}

fn profile_names(cfg: &Config) -> Vec<&String> {
    cfg.keys().collect()
}
