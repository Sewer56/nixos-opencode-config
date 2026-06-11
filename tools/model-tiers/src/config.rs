use crate::env::rel;
use crate::rewrite::agent_files;
use crate::types::{Config, Env, LoadedConfig, TierSet, WORK_PROVIDER};
use anyhow::{Context, bail};

/// Load and validate model-tiers.json from env.tier_file.
pub fn load_config(env: &Env) -> anyhow::Result<LoadedConfig> {
    let data = std::fs::read_to_string(&env.tier_file)
        .with_context(|| format!("read tier file: {}", rel(env, &env.tier_file)))?;
    let cfg: Config = serde_json::from_str(&data).with_context(|| "parse tier config")?;
    let tier_order = derive_tier_order(env, &cfg);
    validate_config(&cfg, &tier_order)?;
    Ok(LoadedConfig {
        tier_order,
        profiles: cfg,
    })
}

/// Derive canonical tier order from: $tierOrder in config → profile keys → file scan.
pub fn derive_tier_order(env: &Env, cfg: &Config) -> Vec<String> {
    let mut seen = std::collections::HashSet::new();
    let mut result = Vec::new();

    // 1. Parse numeric $tierOrder keys
    if let Some(order) = cfg.get("$tierOrder") {
        let mut indexed: Vec<(usize, String)> = Vec::new();
        for (k, v) in order {
            if let Ok(idx) = k.parse::<usize>() {
                if !v.is_empty() {
                    indexed.push((idx, v.clone()));
                }
            }
        }
        indexed.sort_by_key(|(i, _)| *i);
        for (_, tier) in indexed {
            if !tier.is_empty() && seen.insert(tier.clone()) {
                result.push(tier);
            }
        }
    }

    // 2. Collect tier keys from all non-$ profiles
    let mut profile_tiers: Vec<String> = Vec::new();
    for (profile, values) in cfg {
        if profile.starts_with('$') {
            continue;
        }
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

/// Validate that all profiles have identical tier key sets and non-empty values.
pub fn validate_config(cfg: &Config, _tier_order: &[String]) -> anyhow::Result<()> {
    if profile_names(cfg).is_empty() {
        bail!("tier config must contain at least one profile");
    }

    let mut first_keys: Option<std::collections::HashSet<String>> = None;
    let mut first_profile = "";

    for (profile, values) in cfg {
        if profile.starts_with('$') {
            continue;
        }
        let keys: std::collections::HashSet<String> = values.keys().cloned().collect();
        if first_keys.is_none() {
            first_keys = Some(keys);
            first_profile = profile;
        } else if keys != *first_keys.as_ref().unwrap() {
            bail!(
                "profile {:?} tier keys differ from {:?}",
                profile,
                first_profile
            );
        }
        for (tier, model) in values {
            if model.trim().is_empty() {
                bail!("profile {:?} has empty model for {}", profile, tier);
            }
        }
    }
    Ok(())
}

/// Work profile must only use work-provider models.
pub fn validate_work(values: &TierSet, tier_order: &[String]) -> anyhow::Result<()> {
    let mut bad = Vec::new();
    for tier in tier_order {
        if let Some(model) = values.get(tier) {
            if !model.starts_with(WORK_PROVIDER) {
                bad.push(format!("{}={}", tier, model));
            }
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

/// Save config atomically: write to .tmp, then rename.
pub fn save_config(env: &Env, loaded: &LoadedConfig) -> anyhow::Result<()> {
    validate_config(&loaded.profiles, &loaded.tier_order)?;
    let data = marshal_config(&loaded.profiles, &loaded.tier_order)?;
    let tmp = format!("{}.tmp", env.tier_file);
    std::fs::write(&tmp, &data).context("write tier file tmp")?;
    std::fs::rename(&tmp, &env.tier_file).context("rename tier file")?;
    Ok(())
}

/// Marshal config to JSON preserving tier order (LOW/MED/HIGH) and profile order.
pub fn marshal_config(cfg: &Config, tier_order: &[String]) -> anyhow::Result<String> {
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
                let model = values.get(tier).map(|s| s.as_str()).unwrap_or("");
                let model_json = serde_json::to_string(model).unwrap();
                out.push_str(&format!("    \"{}\": {}", tier, model_json));
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
    Ok(out)
}

fn profile_names(cfg: &Config) -> Vec<&String> {
    cfg.keys().filter(|k| !k.starts_with('$')).collect()
}

pub fn sorted_profiles(cfg: &Config) -> Vec<String> {
    let mut names: Vec<String> = cfg
        .keys()
        .filter(|k| !k.starts_with('$'))
        .cloned()
        .collect();
    names.sort();
    names
}

pub fn clone_tier_set(values: &TierSet, tier_order: &[String]) -> TierSet {
    let mut copy = TierSet::new();
    for tier in tier_order {
        if let Some(v) = values.get(tier) {
            copy.insert(tier.clone(), v.clone());
        }
    }
    copy
}
