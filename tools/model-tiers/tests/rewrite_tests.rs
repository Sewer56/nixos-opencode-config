use opencode_model_tiers::rewrite;
use opencode_model_tiers::types::{Env, TierSet};

fn test_env() -> anyhow::Result<(tempfile::TempDir, Env)> {
    let dir = tempfile::tempdir()?;
    let agent_dir = dir.path().join("config").join("agent");
    std::fs::create_dir_all(&agent_dir)?;
    let tier_file = dir.path().join("config").join("model-tiers.json");
    std::fs::write(
        &tier_file,
        r#"{
  "$tierOrder": {"0": "LOW","1": "MED","2": "HIGH"},
  "normal": { "LOW": "low", "MED": "med", "HIGH": "high" },
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
fn test_build_model_line_re_matches_correctly() {
    let tiers = vec!["LOW".to_string(), "MED".to_string(), "HIGH".to_string()];
    let re = rewrite::build_model_line_re(&tiers);
    assert!(re.is_match("model: some-model # LOW"));
    assert!(re.is_match("  model: other-model    # MED keep comment"));
    assert!(re.is_match("model: x # HIGH\r"));
    assert!(!re.is_match("model: unmarked"));
    assert!(!re.is_match("description: leave me"));
}

#[test]
fn test_build_model_line_re_longest_first() {
    // HIGH-FAST must match before HIGH
    let tiers = vec![
        "LOW".to_string(),
        "HIGH".to_string(),
        "HIGH-FAST".to_string(),
    ];
    let re = rewrite::build_model_line_re(&tiers);
    let caps = re.captures("model: x # HIGH-FAST").unwrap();
    assert_eq!(caps.get(4).unwrap().as_str(), "HIGH-FAST");
}

#[test]
fn test_rewrite_content_preserves_tier_markers_and_comments() {
    let tier_order = vec!["LOW".to_string(), "MED".to_string(), "HIGH".to_string()];
    let re = rewrite::build_model_line_re(&tier_order);
    let input = concat!(
        "---\n",
        "model: old-low # LOW\n",
        "model: old-med    # MED keep this comment\n",
        "  model: old-high # HIGH\r\n",
        "model: unmarked\n",
        "description: leave me\n",
        "---\n",
        "\n",
    );

    let mut values = TierSet::new();
    values.insert("LOW".into(), "new-low".into());
    values.insert("MED".into(), "new-med".into());
    values.insert("HIGH".into(), "new-high".into());

    let (output, by_tier, changed) = rewrite::rewrite_content(input, &values, &re);

    assert_eq!(changed, 3);
    for tier in &tier_order {
        assert_eq!(by_tier.get(tier).copied().unwrap_or(0), 1);
    }

    let want = concat!(
        "---\n",
        "model: new-low # LOW\n",
        "model: new-med    # MED keep this comment\n",
        "  model: new-high # HIGH\r\n",
        "model: unmarked\n",
        "description: leave me\n",
        "---\n",
        "\n",
    );
    assert_eq!(
        output, want,
        "rewrite mismatch\nwant:\n{:?}\ngot:\n{:?}",
        want, output
    );
}

#[test]
fn test_rewrite_content_preserves_crlf() {
    let tier_order = vec!["LOW".to_string()];
    let re = rewrite::build_model_line_re(&tier_order);
    let input = "model: old # LOW\r\n";
    let mut values = TierSet::new();
    values.insert("LOW".into(), "new".into());

    let (output, _by_tier, changed) = rewrite::rewrite_content(input, &values, &re);
    assert_eq!(changed, 1);
    assert_eq!(output, "model: new # LOW\r\n");
}

#[test]
fn test_apply_profile_dry_run_does_not_write() {
    let (_dir, env) = test_env().unwrap();
    let tier_order = vec!["LOW".to_string(), "MED".to_string(), "HIGH".to_string()];
    let re = rewrite::build_model_line_re(&tier_order);
    let agent_path = std::path::Path::new(&env.agent_dirs[0]).join("agent.md");
    std::fs::write(&agent_path, "model: old # LOW\n").unwrap();

    let mut values = TierSet::new();
    values.insert("LOW".into(), "new".into());
    values.insert("MED".into(), "med".into());
    values.insert("HIGH".into(), "high".into());

    let result = rewrite::apply_profile(&env, &values, true, &tier_order, &re).unwrap();
    assert_eq!(result.lines, 1);
    assert_eq!(result.files.len(), 1);
    assert_eq!(result.tiers.get("LOW").copied().unwrap_or(0), 1);

    // Dry run should NOT have written
    let content = std::fs::read_to_string(&agent_path).unwrap();
    assert_eq!(content, "model: old # LOW\n");

    // Real apply
    let result = rewrite::apply_profile(&env, &values, false, &tier_order, &re).unwrap();
    assert_eq!(result.lines, 1);
    let content = std::fs::read_to_string(&agent_path).unwrap();
    assert_eq!(content, "model: new # LOW\n");
}

#[test]
fn test_current_counts_ignores_unmarked_models() {
    let (_dir, env) = test_env().unwrap();
    let tier_order = vec!["LOW".to_string(), "MED".to_string(), "HIGH".to_string()];
    let re = rewrite::build_model_line_re(&tier_order);

    let a = std::path::Path::new(&env.agent_dirs[0]).join("a.md");
    std::fs::write(&a, "model: low # LOW\nmodel: nope\n").unwrap();

    let nested = std::path::Path::new(&env.agent_dirs[0]).join("nested");
    std::fs::create_dir_all(&nested).unwrap();
    std::fs::write(nested.join("b.md"), "model: med # MED\n").unwrap();

    let counts = rewrite::current_counts(&env, &tier_order, &re).unwrap();
    assert_eq!(
        counts.get("LOW").unwrap().get("low").copied().unwrap_or(0),
        1
    );
    assert_eq!(
        counts.get("MED").unwrap().get("med").copied().unwrap_or(0),
        1
    );
    assert_eq!(
        counts.get("LOW").unwrap().get("nope").copied().unwrap_or(0),
        0
    );
}

#[test]
fn test_agent_files_finds_md_files_recursively() {
    let (_dir, env) = test_env().unwrap();
    let nested = std::path::Path::new(&env.agent_dirs[0]).join("sub");
    std::fs::create_dir_all(&nested).unwrap();
    std::fs::write(std::path::Path::new(&env.agent_dirs[0]).join("a.md"), "").unwrap();
    std::fs::write(nested.join("b.md"), "").unwrap();
    std::fs::write(nested.join("not-md.txt"), "").unwrap();

    let files = rewrite::agent_files(&env).unwrap();
    let names: Vec<&str> = files
        .iter()
        .map(|f| {
            std::path::Path::new(f)
                .file_name()
                .unwrap()
                .to_str()
                .unwrap()
        })
        .collect();
    assert!(names.contains(&"a.md"));
    assert!(names.contains(&"b.md"));
    assert!(!names.contains(&"not-md.txt"));
}

#[test]
fn test_rewrite_line_unchanged_when_model_matches() {
    let tier_order = vec!["LOW".to_string()];
    let re = rewrite::build_model_line_re(&tier_order);
    let mut values = TierSet::new();
    values.insert("LOW".into(), "same-model".into());

    let (new_line, _tier, changed) =
        rewrite::rewrite_line("model: same-model # LOW\n", &values, &re);
    assert!(!changed);
    assert_eq!(new_line, "model: same-model # LOW\n");
}

#[test]
fn test_rewrite_line_unchanged_when_no_tier_match() {
    let tier_order = vec!["LOW".to_string()];
    let re = rewrite::build_model_line_re(&tier_order);
    let mut values = TierSet::new();
    values.insert("MED".into(), "some-model".into()); // different tier

    let (new_line, _tier, changed) = rewrite::rewrite_line("model: old # LOW\n", &values, &re);
    assert!(!changed);
    assert_eq!(new_line, "model: old # LOW\n");
}
