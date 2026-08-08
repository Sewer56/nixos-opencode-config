use opencode_model_switcher::config;
use opencode_model_switcher::types::{Assignment, Config, Env, TierSet};

#[test]
fn test_load_and_save_config() {
    let json = r#"{
  "$tierOrder": {"0": "EASY","1": "HARD"},
  "normal": {
    "EASY": {"model":"a","variant":"low"},
    "HARD": {"model":"b","variant":"high"}
  }
}"#;
    let (_dir, env) = test_env_with_config(json).unwrap();
    let mut loaded = config::load_config(&env).unwrap();
    assert_eq!(loaded.tier_order, vec!["EASY", "HARD"]);
    assert_eq!(loaded.profiles["normal"]["EASY"], assignment("a", "low"));

    loaded
        .profiles
        .get_mut("normal")
        .unwrap()
        .insert("EASY".into(), assignment("new", "xhigh"));
    config::save_config(&env, &loaded).unwrap();
    let reloaded = config::load_config(&env).unwrap();
    assert_eq!(
        reloaded.profiles["normal"]["EASY"],
        assignment("new", "xhigh")
    );
}

#[test]
fn test_load_config_rejects_old_string_schema() {
    let json = r#"{"$tierOrder":{"0":"EASY"},"normal":{"EASY":"old"}}"#;
    let (_dir, env) = test_env_with_config(json).unwrap();
    assert!(config::load_config(&env).is_err());
}

#[test]
fn test_marshal_config_keeps_tier_order() {
    let tier_order = vec!["EASY".to_string(), "MEDIUM".to_string(), "HARD".to_string()];
    let mut work = TierSet::new();
    work.insert("HARD".into(), assignment("work/high", "high"));
    work.insert("EASY".into(), assignment("work/low", "low"));
    work.insert("MEDIUM".into(), assignment("work/med", "medium"));
    let mut cfg = Config::new();
    cfg.insert("work".into(), work);

    let data = config::marshal_config(&cfg, &tier_order);
    let want = concat!(
        "{\n  \"$tierOrder\": {\"0\": \"EASY\",\"1\": \"MEDIUM\",\"2\": \"HARD\"},\n",
        "  \"work\": {\n",
        "    \"EASY\": {\"model\":\"work/low\",\"variant\":\"low\"},\n",
        "    \"MEDIUM\": {\"model\":\"work/med\",\"variant\":\"medium\"},\n",
        "    \"HARD\": {\"model\":\"work/high\",\"variant\":\"high\"}\n",
        "  }\n}\n"
    );
    assert_eq!(data, want);
}

#[test]
fn test_validate_config_rejects_empty_model_and_invalid_variant() {
    let mut cfg = Config::new();
    cfg.insert(
        "normal".into(),
        TierSet::from([("EASY".into(), assignment("  ", "low"))]),
    );
    assert!(
        config::validate_config(&cfg, &["EASY".into()])
            .unwrap_err()
            .to_string()
            .contains("empty model")
    );

    cfg.get_mut("normal")
        .unwrap()
        .insert("EASY".into(), assignment("model", "extreme"));
    assert!(
        config::validate_config(&cfg, &["EASY".into()])
            .unwrap_err()
            .to_string()
            .contains("invalid variant")
    );
}

#[test]
fn test_validate_config_rejects_empty_profiles() {
    let err = config::validate_config(&Config::new(), &["EASY".into()]).unwrap_err();
    assert!(err.to_string().contains("at least one profile"));
}

#[test]
fn test_validate_config_rejects_mismatched_tier_keys() {
    let mut cfg = Config::new();
    cfg.insert(
        "normal".into(),
        TierSet::from([("EASY".into(), assignment("a", "low"))]),
    );
    cfg.insert(
        "extra".into(),
        TierSet::from([
            ("EASY".into(), assignment("b", "low")),
            ("HARD".into(), assignment("c", "high")),
        ]),
    );
    let err = config::validate_config(&cfg, &["EASY".into(), "HARD".into()]).unwrap_err();
    assert!(err.to_string().contains("differ"));
}

#[test]
fn test_validate_work_requires_work_provider() {
    let tier_order = vec!["EASY".to_string(), "MEDIUM".to_string(), "HARD".to_string()];
    let mut good = TierSet::new();
    good.insert("EASY".into(), assignment("sewer-axonhub-work/small", "low"));
    good.insert(
        "MEDIUM".into(),
        assignment("sewer-axonhub-work/medium", "medium"),
    );
    good.insert(
        "HARD".into(),
        assignment("sewer-axonhub-work/large", "high"),
    );
    assert!(config::validate_work(&good, &tier_order).is_ok());

    let mut bad = good.clone();
    bad.insert("MEDIUM".into(), assignment("sewer-axonhub/other", "medium"));
    assert!(config::validate_work(&bad, &tier_order).is_err());
}

fn assignment(model: &str, variant: &str) -> Assignment {
    Assignment {
        model: model.into(),
        variant: variant.into(),
    }
}

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
