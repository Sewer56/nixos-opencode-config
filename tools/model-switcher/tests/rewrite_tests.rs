//! Rewrite and agent-inventory integration tests.

use opencode_model_switcher::rewrite;
use opencode_model_switcher::types::{Assignment, Env, TierSet};

#[test]
fn agent_inventory_matches_rewrites_not_body_examples_or_external_paths() {
    let (dir, mut env) = test_env().unwrap();
    let local = dir.path().join(".opencode/agent");
    std::fs::create_dir_all(&local).unwrap();
    env.agent_dirs.push(local.to_string_lossy().into_owned());
    let tiers = vec!["WRITER".into(), "CODER".into()];
    let re = rewrite::build_model_line_re(&tiers);
    let input = "---\nmodel: same # WRITER\nvariant: low\n---\nmodel: example # CODER\n";
    for root in &env.agent_dirs {
        std::fs::write(std::path::Path::new(root).join("same.md"), input).unwrap();
        std::fs::write(
            std::path::Path::new(root).join("body.md"),
            "model: example # WRITER\n",
        )
        .unwrap();
        std::fs::write(
            std::path::Path::new(root).join("prefix.md"),
            "---\nmodel: old # WRITER-OTHER\n---\n",
        )
        .unwrap();
    }
    let outside = dir.path().join("outside.md");
    std::fs::write(&outside, input).unwrap();
    #[cfg(unix)]
    std::os::unix::fs::symlink(&outside, local.join("escape.md")).unwrap();
    let agents = rewrite::affected_agents(&env, &re).unwrap();
    assert_eq!(
        agents["WRITER"],
        [".opencode/agent/same.md", "config/agent/same.md"]
    );
    assert!(!agents.contains_key("CODER"));
    let values = TierSet::from([("WRITER".into(), assignment("same", "low"))]);
    assert_eq!(
        rewrite::apply_profile(&env, &values, true, &tiers, &re)
            .unwrap()
            .lines,
        0
    );
    let values = TierSet::from([("WRITER".into(), assignment("new", "medium"))]);
    assert_eq!(
        rewrite::apply_profile(&env, &values, false, &tiers, &re)
            .unwrap()
            .files
            .len(),
        2
    );
    assert_eq!(rewrite::affected_agents(&env, &re).unwrap(), agents);
    assert_eq!(std::fs::read_to_string(outside).unwrap(), input);
    env.agent_dirs
        .push(dir.path().join("missing").to_string_lossy().into_owned());
    assert!(rewrite::affected_agents(&env, &re).is_err());
}

#[test]
fn exact_tags_reject_unknown_suffixes_without_variant_writes() {
    let tiers = [
        "HARD",
        "STYLE",
        "STYLE-REVIEW",
        "CORRECTNESS-REVIEW",
        "CODER",
    ]
    .map(String::from);
    let re = rewrite::build_model_line_re(&tiers);
    let values = tiers
        .iter()
        .map(|tier| (tier.clone(), assignment("new", "low")))
        .collect();
    for tag in [
        "HARD-UNKNOWN",
        "STYLE-REVIEW-UNKNOWN",
        "CODER.extra",
        "CODER/other",
        "CODER_OLD",
    ] {
        let input = format!("---\nmodel: old # {tag}\nvariant: max\n---\n");
        let (output, counts, changed) = rewrite::rewrite_content(&input, &values, &re);
        assert_eq!(output, input);
        assert_eq!(changed, 0);
        assert!(counts.is_empty());
    }
    let caps = re
        .captures("model: old # STYLE-REVIEW keep comment")
        .unwrap();
    assert_eq!(&caps[4], "STYLE-REVIEW");
}

#[test]
fn seven_roles_rewrite_both_roots_with_independent_models_and_variants() {
    let (dir, mut env) = test_env().unwrap();
    let local = dir.path().join(".opencode/agent");
    std::fs::create_dir_all(&local).unwrap();
    env.agent_dirs.push(local.to_string_lossy().into_owned());
    let tiers = [
        "EASY",
        "MEDIUM",
        "HARD",
        "STYLE-REVIEW",
        "CORRECTNESS-REVIEW",
        "CODER",
        "WRITER",
    ]
    .map(String::from);
    let re = rewrite::build_model_line_re(&tiers);
    for root in &env.agent_dirs {
        for tier in &tiers {
            std::fs::write(
                std::path::Path::new(root).join(format!("{tier}.md")),
                format!("---\r\nmodel: old # {tier} keep\r\nvariant: max # comment\r\n---\r\n"),
            )
            .unwrap();
        }
    }
    for profile in ["normal", "work"] {
        let values: TierSet = tiers
            .iter()
            .enumerate()
            .map(|(i, tier)| {
                (
                    tier.clone(),
                    assignment(
                        &format!("{profile}/{tier}"),
                        ["low", "medium", "high", "xhigh", "max", "low", "medium"][i],
                    ),
                )
            })
            .collect();
        let before: Vec<_> = rewrite::agent_files(&env)
            .unwrap()
            .iter()
            .map(|p| std::fs::read(p).unwrap())
            .collect();
        let preview = rewrite::apply_profile(&env, &values, true, &tiers, &re).unwrap();
        let after: Vec<_> = rewrite::agent_files(&env)
            .unwrap()
            .iter()
            .map(|p| std::fs::read(p).unwrap())
            .collect();
        assert_eq!(before, after);
        let applied = rewrite::apply_profile(&env, &values, false, &tiers, &re).unwrap();
        assert_eq!(preview.lines, applied.lines);
        assert_eq!(preview.tiers, applied.tiers);
        assert_eq!(applied.files.len(), 14);
        for tier in &tiers {
            for root in &env.agent_dirs {
                let content =
                    std::fs::read_to_string(std::path::Path::new(root).join(format!("{tier}.md")))
                        .unwrap();
                assert_eq!(
                    content,
                    format!(
                        "---\r\nmodel: {} # {tier} keep\r\nvariant: {} # comment\r\n---\r\n",
                        values[tier].model, values[tier].variant
                    )
                );
            }
        }
        assert_eq!(
            rewrite::apply_profile(&env, &values, true, &tiers, &re)
                .unwrap()
                .lines,
            0
        );
    }
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

#[test]
fn test_apply_profile_dry_run_then_apply() {
    let (_dir, env) = test_env().unwrap();
    let tiers = vec!["EASY".into(), "MEDIUM".into(), "HARD".into()];
    let re = rewrite::build_model_line_re(&tiers);
    let agent_path = std::path::Path::new(&env.agent_dirs[0]).join("agent.md");
    std::fs::write(
        &agent_path,
        "---\nmodel: old # EASY\nvariant: max\nmodel: unmarked\n---\n",
    )
    .unwrap();

    let result = rewrite::apply_profile(&env, &values(), true, &tiers, &re).unwrap();
    assert_eq!(result.lines, 2);
    assert_eq!(
        std::fs::read_to_string(&agent_path).unwrap(),
        "---\nmodel: old # EASY\nvariant: max\nmodel: unmarked\n---\n"
    );

    rewrite::apply_profile(&env, &values(), false, &tiers, &re).unwrap();
    assert_eq!(
        std::fs::read_to_string(&agent_path).unwrap(),
        "---\nmodel: new-easy # EASY\nvariant: low\nmodel: unmarked\n---\n"
    );
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
fn test_rewrite_content_inserts_missing_variant_with_indent_and_eol() {
    let re = rewrite::build_model_line_re(&["EASY".into()]);
    let input = "---\r\n  model: old # EASY\r\ndescription: keep\r\n---\r\n";
    let (output, by_tier, changed) = rewrite::rewrite_content(
        input,
        &TierSet::from([("EASY".into(), assignment("new", "low"))]),
        &re,
    );
    assert_eq!(changed, 2);
    assert_eq!(by_tier["EASY"], 2);
    assert_eq!(
        output,
        "---\r\n  model: new # EASY\r\n  variant: low\r\ndescription: keep\r\n---\r\n"
    );
}

#[test]
fn test_rewrite_content_is_unchanged_when_assignment_matches() {
    let re = rewrite::build_model_line_re(&["EASY".into()]);
    let current = TierSet::from([("EASY".into(), assignment("same", "low"))]);
    let input = "---\nmodel: same # EASY\nvariant: low\n---\n";
    let (output, _, changed) = rewrite::rewrite_content(input, &current, &re);
    assert_eq!(changed, 0);
    assert_eq!(output, input);
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

fn values() -> TierSet {
    TierSet::from([
        ("EASY".into(), assignment("new-easy", "low")),
        ("MEDIUM".into(), assignment("new-medium", "medium")),
        ("HARD".into(), assignment("new-hard", "high")),
    ])
}

fn assignment(model: &str, variant: &str) -> Assignment {
    Assignment {
        model: model.into(),
        variant: variant.into(),
    }
}
