use opencode_model_tiers::cli;
use opencode_model_tiers::types::Env;

/// Build a minimal test Env with temp directories.
fn test_env() -> anyhow::Result<(tempfile::TempDir, Env)> {
    let dir = tempfile::tempdir()?;
    let agent_dir = dir.path().join("config").join("agent");
    std::fs::create_dir_all(&agent_dir)?;
    let tier_file = dir.path().join("config").join("model-tiers.json");
    std::fs::write(
        &tier_file,
        r#"{
  "$tierOrder": {"0": "LOW","1": "MED","2": "HIGH"},
  "normal": { "LOW": "low-model", "MED": "med-model", "HIGH": "high-model" },
  "work": { "LOW": "sewer-axonhub-work/low", "MED": "sewer-axonhub-work/med", "HIGH": "sewer-axonhub-work/high" }
}
"#,
    )?;
    let env = Env {
        root: dir.path().to_string_lossy().into_owned(),
        tier_file: tier_file.to_string_lossy().into_owned(),
        agent_dirs: vec![agent_dir.to_string_lossy().into_owned()],
    };
    Ok((dir, env))
}

#[test]
fn test_help_command_returns_ok() {
    let (_dir, env) = test_env().unwrap();
    let result = cli::run_cli(&env, &["help".to_string()]);
    assert!(result.is_ok());
}

#[test]
fn test_run_cli_unknown_command() {
    let (_dir, env) = test_env().unwrap();
    let result = cli::run_cli(&env, &["nonexistent".to_string()]);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("unknown command"));
}

#[test]
fn test_run_cli_apply_rejects_unknown_flag() {
    let (_dir, env) = test_env().unwrap();
    let result = cli::run_cli(
        &env,
        &[
            "apply".to_string(),
            "normal".to_string(),
            "--nope".to_string(),
        ],
    );
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("usage"));
}

#[test]
fn test_take_bool_flag_accepts_flag_before_and_after_positional() {
    let (_dir, env) = test_env().unwrap();
    // --dry-run after profile name
    let result = cli::run_cli(
        &env,
        &[
            "apply".to_string(),
            "normal".to_string(),
            "--dry-run".to_string(),
        ],
    );
    assert!(result.is_ok(), "--dry-run after profile should succeed");
    // --dry-run before profile name
    let result = cli::run_cli(
        &env,
        &[
            "apply".to_string(),
            "--dry-run".to_string(),
            "normal".to_string(),
        ],
    );
    assert!(result.is_ok(), "--dry-run before profile should succeed");
}

#[test]
#[ignore = "requires a TTY; skipped in Nix sandbox"]
fn test_run_cli_empty_args_errors_without_terminal() {
    let (_dir, env) = test_env().unwrap();
    let result = cli::run_cli(&env, &[]);
    assert!(result.is_err());
}
