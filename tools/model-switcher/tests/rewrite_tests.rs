use opencode_model_switcher::rewrite;
use opencode_model_switcher::types::{Assignment, Env, TierSet};

fn assignment(model: &str, variant: &str) -> Assignment {
    Assignment {
        model: model.into(),
        variant: variant.into(),
    }
}

fn values() -> TierSet {
    TierSet::from([
        ("EASY".into(), assignment("new-easy", "low")),
        ("MEDIUM".into(), assignment("new-medium", "medium")),
        ("HARD".into(), assignment("new-hard", "high")),
    ])
}

fn test_env() -> anyhow::Result<(tempfile::TempDir, Env)> {
    let dir = tempfile::tempdir()?;
    let agent_dir = dir.path().join("config").join("agent");
    std::fs::create_dir_all(&agent_dir)?;
    let root = dir.path().to_string_lossy().into_owned();
    let tier_file = dir
        .path()
        .join("config/model-switcher.json")
        .to_string_lossy()
        .into_owned();
    Ok((
        dir,
        Env {
            root,
            tier_file,
            agent_dirs: vec![agent_dir.to_string_lossy().into_owned()],
        },
    ))
}

#[test]
fn test_build_model_line_re_matches_tiers() {
    let re = rewrite::build_model_line_re(&["EASY".into(), "MEDIUM".into(), "HARD".into()]);
    assert!(re.is_match("model: some-model # EASY"));
    assert!(re.is_match("  model: other # MEDIUM keep comment"));
    assert!(re.is_match("model: x # HARD\r"));
    assert!(!re.is_match("model: unmarked"));
}

#[test]
fn test_rewrite_content_updates_models_and_variants() {
    let tiers = vec!["EASY".into(), "MEDIUM".into(), "HARD".into()];
    let re = rewrite::build_model_line_re(&tiers);
    let input = concat!(
        "---\n",
        "model: old-easy # EASY\n",
        "variant: max\n",
        "model: old-medium # MEDIUM keep\n",
        "variant: low # preserve\n",
        "  model: old-hard # HARD\r\n",
        "  variant: xhigh\r\n",
        "---\n"
    );
    let (output, by_tier, changed) = rewrite::rewrite_content(input, &values(), &re);
    assert_eq!(changed, 6);
    assert_eq!(by_tier["EASY"], 2);
    assert_eq!(by_tier["MEDIUM"], 2);
    assert_eq!(by_tier["HARD"], 2);
    assert_eq!(
        output,
        concat!(
            "---\n",
            "model: new-easy # EASY\n",
            "variant: low\n",
            "model: new-medium # MEDIUM keep\n",
            "variant: medium # preserve\n",
            "  model: new-hard # HARD\r\n",
            "  variant: high\r\n",
            "---\n"
        )
    );
}

#[test]
fn test_rewrite_content_inserts_missing_variant_with_indent_and_eol() {
    let re = rewrite::build_model_line_re(&["EASY".into()]);
    let input = "  model: old # EASY\r\ndescription: keep\r\n";
    let (output, by_tier, changed) = rewrite::rewrite_content(
        input,
        &TierSet::from([("EASY".into(), assignment("new", "low"))]),
        &re,
    );
    assert_eq!(changed, 2);
    assert_eq!(by_tier["EASY"], 2);
    assert_eq!(
        output,
        "  model: new # EASY\r\n  variant: low\r\ndescription: keep\r\n"
    );
}

#[test]
fn test_rewrite_content_is_unchanged_when_assignment_matches() {
    let re = rewrite::build_model_line_re(&["EASY".into()]);
    let current = TierSet::from([("EASY".into(), assignment("same", "low"))]);
    let input = "model: same # EASY\nvariant: low\n";
    let (output, _, changed) = rewrite::rewrite_content(input, &current, &re);
    assert_eq!(changed, 0);
    assert_eq!(output, input);
}

#[test]
fn test_apply_profile_dry_run_and_current_counts() {
    let (_dir, env) = test_env().unwrap();
    let tiers = vec!["EASY".into(), "MEDIUM".into(), "HARD".into()];
    let re = rewrite::build_model_line_re(&tiers);
    let agent_path = std::path::Path::new(&env.agent_dirs[0]).join("agent.md");
    std::fs::write(
        &agent_path,
        "model: old # EASY\nvariant: max\nmodel: unmarked\n",
    )
    .unwrap();

    let result = rewrite::apply_profile(&env, &values(), true, &tiers, &re).unwrap();
    assert_eq!(result.lines, 2);
    assert_eq!(
        std::fs::read_to_string(&agent_path).unwrap(),
        "model: old # EASY\nvariant: max\nmodel: unmarked\n"
    );

    rewrite::apply_profile(&env, &values(), false, &tiers, &re).unwrap();
    assert_eq!(
        std::fs::read_to_string(&agent_path).unwrap(),
        "model: new-easy # EASY\nvariant: low\nmodel: unmarked\n"
    );
    let counts = rewrite::current_counts(&env, &tiers, &re).unwrap();
    assert_eq!(counts["EASY"]["new-easy"], 1);
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
    assert_eq!(files.len(), 2);
}
