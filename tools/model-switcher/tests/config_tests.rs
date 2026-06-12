use opencode_model_switcher::config;
use opencode_model_switcher::types::{Config, Env, TierSet};

fn test_env_with_config(json: &str) -> anyhow::Result<(tempfile::TempDir, Env)> {
    let dir = tempfile::tempdir()?;
    let agent_dir = dir.path().join("config").join("agent");
    std::fs::create_dir_all(&agent_dir)?;
    let tier_file = dir.path().join("config").join("model-switcher.json");
    std::fs::write(&tier_file, json)?;
    let env = Env {
        root: dir.path().to_string_lossy().into_owned(),
        tier_file: tier_file.to_string_lossy().into_owned(),
        agent_dirs: vec![agent_dir.to_string_lossy().into_owned()],
    };
    Ok((dir, env))
}

#[test]
fn test_validate_work_requires_work_provider() {
    let tier_order = vec!["LOW".to_string(), "MED".to_string(), "HIGH".to_string()];
    let mut good = TierSet::new();
    good.insert("LOW".into(), "sewer-axonhub-work/gpt-5.4-mini".into());
    good.insert("MED".into(), "sewer-axonhub-work/gpt-5.4".into());
    good.insert("HIGH".into(), "sewer-axonhub-work/gpt-5.5".into());
    assert!(config::validate_work(&good, &tier_order).is_ok());

    let mut bad = good.clone();
    bad.insert("MED".into(), "sewer-axonhub/MiniMax-M3".into());
    assert!(config::validate_work(&bad, &tier_order).is_err());
}

#[test]
fn test_marshal_config_keeps_tier_order() {
    let tier_order = vec!["LOW".to_string(), "MED".to_string(), "HIGH".to_string()];
    let mut work = TierSet::new();
    work.insert("HIGH".into(), "sewer-axonhub-work/high".into());
    work.insert("LOW".into(), "sewer-axonhub-work/low".into());
    work.insert("MED".into(), "sewer-axonhub-work/med".into());
    let mut cfg = Config::new();
    cfg.insert("work".into(), work);

    let data = config::marshal_config(&cfg, &tier_order);
    let want = "{\n  \"$tierOrder\": {\"0\": \"LOW\",\"1\": \"MED\",\"2\": \"HIGH\"},\n  \"work\": {\n    \
                \"LOW\": \"sewer-axonhub-work/low\",\n    \"MED\": \"sewer-axonhub-work/med\",\n    \
                \"HIGH\": \"sewer-axonhub-work/high\"\n  }\n}\n";
    assert_eq!(
        data, want,
        "marshal mismatch\nwant:\n{}\ngot:\n{}",
        want, data
    );
}

#[test]
fn test_validate_config_rejects_empty_profiles() {
    let tier_order = vec!["LOW".to_string()];
    let cfg = Config::new();
    let err = config::validate_config(&cfg, &tier_order).unwrap_err();
    assert!(err.to_string().contains("at least one profile"));
}

#[test]
fn test_validate_config_rejects_mismatched_tier_keys() {
    let tier_order = vec!["LOW".to_string(), "MED".to_string()];
    let mut cfg = Config::new();
    let mut normal = TierSet::new();
    normal.insert("LOW".into(), "a".into());
    normal.insert("MED".into(), "b".into());
    cfg.insert("normal".into(), normal);

    let mut extra = TierSet::new();
    extra.insert("LOW".into(), "c".into());
    extra.insert("MED".into(), "d".into());
    extra.insert("HIGH".into(), "e".into());
    cfg.insert("extra".into(), extra);

    let err = config::validate_config(&cfg, &tier_order).unwrap_err();
    assert!(err.to_string().contains("differ"));
}

#[test]
fn test_validate_config_rejects_empty_model_values() {
    let tier_order = vec!["LOW".to_string(), "MED".to_string()];
    let mut cfg = Config::new();
    let mut normal = TierSet::new();
    normal.insert("LOW".into(), "good".into());
    normal.insert("MED".into(), "  ".into()); // whitespace-only model name
    cfg.insert("normal".into(), normal);

    let err = config::validate_config(&cfg, &tier_order).unwrap_err();
    assert!(err.to_string().contains("empty model"));
}
#[test]
fn test_load_config_reads_tier_file() {
    let json = r#"{
  "$tierOrder": {"0": "LOW","1": "MED"},
  "normal": { "LOW": "a", "MED": "b" }
}"#;
    let (_dir, env) = test_env_with_config(json).unwrap();
    let loaded = config::load_config(&env).unwrap();
    assert_eq!(loaded.tier_order, vec!["LOW", "MED"]);
    assert_eq!(
        loaded.profiles.get("normal").unwrap().get("LOW").unwrap(),
        "a"
    );
}

#[test]
fn test_save_config_writes_atomically() {
    let json = r#"{
  "$tierOrder": {"0": "LOW"},
  "normal": { "LOW": "old" }
}"#;
    let (_dir, env) = test_env_with_config(json).unwrap();
    let mut loaded = config::load_config(&env).unwrap();
    loaded
        .profiles
        .get_mut("normal")
        .unwrap()
        .insert("LOW".into(), "new".into());
    config::save_config(&env, &loaded).unwrap();

    let reloaded = config::load_config(&env).unwrap();
    assert_eq!(
        reloaded.profiles.get("normal").unwrap().get("LOW").unwrap(),
        "new"
    );
}
