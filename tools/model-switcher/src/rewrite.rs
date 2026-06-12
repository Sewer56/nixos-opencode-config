use crate::types::{ApplyResult, Env, TierSet};
use anyhow::Context;
use regex::Regex;
use std::collections::BTreeMap;
use std::sync::LazyLock;

/// Regex for discovering tier tags from model lines in agent markdown files.
pub static MODEL_LINE_DISCOVERY_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"^\s*model:\s*\S+\s*#\s*(\S+)\b.*$"#).unwrap());

/// Build a regex that matches tagged frontmatter model lines with given tier names.
/// Tiers are sorted longest-first so e.g. "HIGH-FAST" matches before "HIGH".
pub fn build_model_line_re(tiers: &[String]) -> Regex {
    let mut sorted = tiers.to_vec();
    sorted.sort_by_key(|b| std::cmp::Reverse(b.len()));
    let alt: Vec<String> = sorted.iter().map(|t| regex::escape(t)).collect();
    let pattern = format!(r#"^(\s*model:\s*)(\S+)(\s*#\s*({})\b.*)$"#, alt.join("|"));
    Regex::new(&pattern).expect("build model line regex")
}

/// Apply a profile's tier values to all agent markdown files.
/// If dry_run is true, no files are written.
pub fn apply_profile(
    env: &Env,
    values: &TierSet,
    dry_run: bool,
    _tier_order: &[String],
    re: &Regex,
) -> anyhow::Result<ApplyResult> {
    let mut result = ApplyResult::default();
    let files = agent_files(env)?;

    for file in &files {
        let data = std::fs::read_to_string(file).with_context(|| format!("read: {}", file))?;
        let (new_text, by_tier, changed_lines) = rewrite_content(&data, values, re);
        if changed_lines == 0 {
            continue;
        }
        result.files.insert(file.clone(), changed_lines);
        result.lines += changed_lines;
        for (tier, count) in by_tier {
            *result.tiers.entry(tier).or_insert(0) += count;
        }
        if !dry_run {
            write_file_atomic(file, new_text.as_bytes())?;
        }
    }
    Ok(result)
}

/// Count current assignments per tier/model in agent files.
pub fn current_counts(
    env: &Env,
    tier_order: &[String],
    re: &Regex,
) -> anyhow::Result<BTreeMap<String, BTreeMap<String, usize>>> {
    let mut counts: BTreeMap<String, BTreeMap<String, usize>> = BTreeMap::new();
    for tier in tier_order {
        counts.insert(tier.clone(), BTreeMap::new());
    }
    let files = agent_files(env)?;
    for file in &files {
        let data = std::fs::read_to_string(file).with_context(|| format!("read: {}", file))?;
        for line in data.lines() {
            if let Some(parsed) = parse_tagged_model_line(line, re) {
                *counts
                    .entry(parsed.tier.clone())
                    .or_default()
                    .entry(parsed.model)
                    .or_insert(0) += 1;
            }
        }
    }
    Ok(counts)
}

/// Rewrite full file content. Pure function — easy to test.
/// Returns (new_content, changes_by_tier, lines_changed).
pub fn rewrite_content(
    input: &str,
    values: &TierSet,
    re: &Regex,
) -> (String, BTreeMap<String, usize>, usize) {
    let mut out = String::new();
    let mut by_tier: BTreeMap<String, usize> = BTreeMap::new();
    let mut changed = 0;

    for line in input.split_inclusive('\n') {
        let (new_line, tier, did_change) = rewrite_line(line, values, re);
        out.push_str(&new_line);
        if did_change {
            if let Some(t) = tier {
                *by_tier.entry(t).or_insert(0) += 1;
            }
            changed += 1;
        }
    }

    (out, by_tier, changed)
}

/// Find all .md files under agent directories.
pub fn agent_files(env: &Env) -> anyhow::Result<Vec<String>> {
    let mut files = Vec::new();
    for dir in &env.agent_dirs {
        walk_dir_entries(dir, &mut files)?;
    }
    files.sort();
    Ok(files)
}

/// Rewrite a single line if it is a tagged model assignment whose tier maps to a different model.
/// Returns (new_line, tier, changed).
pub fn rewrite_line(line: &str, values: &TierSet, re: &Regex) -> (String, Option<String>, bool) {
    let (body, eol) = split_eol(line);
    let caps = match re.captures(body) {
        Some(c) => c,
        None => return (line.to_string(), None, false),
    };
    let old_model = caps.get(2).unwrap().as_str();
    let tier = caps.get(4).unwrap().as_str().to_string();
    let new_model = match values.get(&tier) {
        Some(m) if !m.is_empty() => m,
        _ => return (line.to_string(), Some(tier), false),
    };
    if old_model == new_model {
        return (line.to_string(), Some(tier), false);
    }
    let prefix = caps.get(1).unwrap().as_str();
    let suffix = caps.get(3).unwrap().as_str();
    (
        format!("{}{}{}{}", prefix, new_model, suffix, eol),
        Some(tier),
        true,
    )
}

// -- private helpers --

/// A parsed tagged model line.
#[derive(Debug)]
struct TaggedModelLine {
    model: String,
    tier: String,
}

fn parse_tagged_model_line(line: &str, re: &Regex) -> Option<TaggedModelLine> {
    let (body, _eol) = split_eol(line);
    re.captures(body).map(|caps| TaggedModelLine {
        model: caps.get(2).unwrap().as_str().to_string(),
        tier: caps.get(4).unwrap().as_str().to_string(),
    })
}

fn split_eol(line: &str) -> (&str, &str) {
    if let Some(body) = line.strip_suffix("\r\n") {
        (body, "\r\n")
    } else if let Some(body) = line.strip_suffix('\n') {
        (body, "\n")
    } else {
        (line, "")
    }
}

fn walk_dir_entries(dir: &str, files: &mut Vec<String>) -> anyhow::Result<()> {
    walk_dir_entries_depth(dir, files, 0)
}

const MAX_WALK_DEPTH: usize = 32;

fn walk_dir_entries_depth(dir: &str, files: &mut Vec<String>, depth: usize) -> anyhow::Result<()> {
    if depth > MAX_WALK_DEPTH {
        return Ok(());
    }
    for entry in std::fs::read_dir(dir).with_context(|| format!("read agent dir: {}", dir))? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            walk_dir_entries_depth(&path.to_string_lossy(), files, depth + 1)?;
        } else if path.extension().map(|e| e == "md").unwrap_or(false) {
            files.push(path.to_string_lossy().into_owned());
        }
    }
    Ok(())
}

fn write_file_atomic(path: &str, data: &[u8]) -> anyhow::Result<()> {
    let tmp = format!("{}.tmp", path);
    std::fs::write(&tmp, data).context("write tmp")?;
    std::fs::rename(&tmp, path)
        .inspect_err(|_| {
            let _ = std::fs::remove_file(&tmp);
        })
        .context("rename atomic")?;
    Ok(())
}
