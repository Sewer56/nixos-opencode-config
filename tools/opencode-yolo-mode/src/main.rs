use anyhow::{Context, Result, bail};
use clap::{Parser, Subcommand};
use ignore::WalkBuilder;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Parser, Debug)]
#[command(about = "Toggle opencode external_directory '*' between ask (regular) and allow (yolo)")]
struct Args {
    #[command(subcommand)]
    cmd: Option<Cmd>,

    /// Repository root override. Default: walk upward from CWD.
    #[arg(long, value_name = "PATH")]
    repo: Option<PathBuf>,
}

struct Found {
    line_idx: usize,
    decision: String,
}

#[derive(Subcommand, Debug)]
enum Cmd {
    /// Flip every external_directory '*' to allow (guards after '*' keep winning).
    On,
    /// Restore every external_directory '*' to ask.
    Off,
    /// Report current mode without changing anything.
    Status,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let root = find_repo(args.repo)?;
    let targets = collect_targets(&root)?;
    if targets.is_empty() {
        bail!(
            "no agent markdown or config/opencode.json found under {}",
            root.display()
        );
    }
    match args.cmd.unwrap_or(Cmd::Status) {
        Cmd::Status => status(&targets),
        Cmd::On => apply(&targets, "allow"),
        Cmd::Off => apply(&targets, "ask"),
    }
}

fn apply(targets: &[PathBuf], to: &str) -> Result<()> {
    let mut changed = 0_u32;
    let mut already = 0_u32;
    for path in targets {
        let text = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
        let found = scan(&text, path);
        if found.is_empty() {
            continue; // no external_directory entry; nothing to flip
        }
        let Some(found) = found.into_iter().find(|f| f.decision != to) else {
            already += 1;
            continue;
        };
        let replacement = match flip_line(&text, found.line_idx, to, path) {
            Some(line) => line,
            None => {
                println!("  skipped (unparsable line): {}", path.display());
                continue;
            }
        };
        write_line(path, &text, found.line_idx, &replacement)?;
        changed += 1;
    }
    println!("target: '*' -> {to}");
    println!("  changed: {changed}  already {to}: {already}");
    println!("restart opencode for the change to take effect");
    if to == "allow" {
        println!(
            "note: tests/test_draft_workflow.py expects '\"*\": ask' while ON; run it only after `off`"
        );
    }
    Ok(())
}

/// Agent markdown under both agent directories, plus the global JSONC config.
fn collect_targets(root: &Path) -> Result<Vec<PathBuf>> {
    let mut targets = Vec::new();
    for dir in [
        root.join("config").join("agent"),
        root.join(".opencode").join("agent"),
    ] {
        if !dir.is_dir() {
            continue;
        }
        for entry in WalkBuilder::new(&dir).hidden(false).build().flatten() {
            let path = entry.into_path();
            if path.extension().is_some_and(|ext| ext == "md") {
                targets.push(path);
            }
        }
    }
    let json = root.join("config").join("opencode.json");
    if json.is_file() {
        targets.push(json);
    }
    targets.sort();
    Ok(targets)
}

/// Walk upward from CWD (or the --repo override) until a directory containing
/// `config/agent`, `.opencode/agent`, or `config/opencode.json` is found.
fn find_repo(override_root: Option<PathBuf>) -> Result<PathBuf> {
    if let Some(root) = override_root {
        if !root.join("config").is_dir() && !root.join(".opencode").is_dir() {
            bail!(
                "{} does not look like the opencode config repository",
                root.display()
            );
        }
        return Ok(root);
    }
    let mut dir = std::env::current_dir().context("get current directory")?;
    loop {
        if dir.join("config").join("agent").is_dir()
            || dir.join(".opencode").join("agent").is_dir()
            || dir.join("config").join("opencode.json").is_file()
        {
            return Ok(dir);
        }
        match dir.parent() {
            Some(parent) => dir = parent.to_path_buf(),
            None => bail!(
                "opencode config repository not found; run from the repository or pass --repo"
            ),
        }
    }
}

fn status(targets: &[PathBuf]) -> Result<()> {
    let (mut ask, mut allow, mut none, mut mixed) = (0_u32, 0_u32, 0_u32, 0_u32);
    for path in targets {
        let text = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
        let decisions: Vec<String> = scan(&text, path).into_iter().map(|f| f.decision).collect();
        if decisions.is_empty() {
            none += 1;
        } else if decisions.iter().all(|d| d == "ask") {
            ask += 1;
        } else if decisions.iter().all(|d| d == "allow") {
            allow += 1;
        } else {
            mixed += 1;
        }
    }
    let mode = match (mixed, allow, ask) {
        (0, 0, _) => "OFF (regular)",
        (0, _, 0) => "ON (yolo)",
        _ => "MIXED",
    };
    println!("yolo mode: {mode}");
    println!("  allow: {allow}  ask: {ask}  mixed: {mixed}  no external_directory: {none}");
    if mixed > 0 {
        println!("  run `opencode-yolo-mode on` or `off` to force one state");
    }
    println!("  guards after '*' (secrets deny/ask) always keep winning");
    Ok(())
}

/// Build the replacement for the target line, preserving indent, original
/// colon spacing, and any trailing content (JSON comma).
fn flip_line(text: &str, line_idx: usize, to: &str, path: &Path) -> Option<String> {
    let line = text.lines().nth(line_idx)?;
    let indent = &line[..line.len() - line.trim_start().len()];
    let trimmed = line.trim();
    let new_trimmed = if path.extension().is_some_and(|ext| ext == "json") {
        // `"*": "ask",`
        let (key, value) = trimmed.split_once("\":")?;
        let v = value.trim();
        let closing = v[1..].find('"')? + 1;
        format!("{key}\": \"{to}\"{}", &v[closing + 1..])
    } else {
        // `"*": ask`
        let (key, value) = trimmed.split_once(':')?;
        let spacing = &value[..value.len() - value.trim_start().len()];
        format!("{key}:{spacing}{to}")
    };
    Some(format!("{indent}{new_trimmed}"))
}

/// Every external_directory '*' (or scalar) entry inside the file.
/// Markdown scanning is limited to the frontmatter block; JSON scans all lines.
fn scan(text: &str, path: &Path) -> Vec<Found> {
    let lines: Vec<&str> = text.lines().collect();
    let limit = match frontmatter_end(&lines, path) {
        Some(end) => end,
        None => lines.len(),
    };
    let mut found = Vec::new();
    let mut i = 0;
    while i < limit {
        if let Some(f) = star_at(&lines, i, path) {
            let (idx, decision) = f;
            found.push(Found {
                line_idx: idx,
                decision,
            });
            i = idx + 1;
        } else {
            i += 1;
        }
    }
    found
}

fn write_line(path: &Path, text: &str, line_idx: usize, replacement: &str) -> Result<()> {
    let mut lines: Vec<String> = text.lines().map(str::to_owned).collect();
    lines[line_idx] = replacement.to_owned();
    let mut out = lines.join("\n");
    if text.ends_with('\n') {
        out.push('\n');
    }
    fs::write(path, out).with_context(|| format!("write {}", path.display()))
}

/// Line index just past the YAML frontmatter block, or None for JSON or absent frontmatter.
fn frontmatter_end(lines: &[&str], path: &Path) -> Option<usize> {
    if path.extension().is_some_and(|ext| ext == "json") {
        return None;
    }
    if lines.first()?.trim() != "---" {
        return None;
    }
    Some(lines.iter().skip(1).position(|l| l.trim() == "---")? + 1)
}

/// If `lines[i]` opens an external_directory map, return the index and decision
/// of its '*' entry. Scalar forms (whole-tool allow) are never touched: they are
/// per-agent capabilities (e.g. plan/build), not the yolo wildcard.
fn star_at(lines: &[&str], i: usize, path: &Path) -> Option<(usize, String)> {
    let trimmed = lines[i].trim();
    let is_json = path.extension().is_some_and(|ext| ext == "json");

    if is_json {
        if !trimmed.starts_with("\"external_directory\": {") {
            return None;
        }
        let mut depth = brace_delta(lines[i]);
        let mut j = i + 1;
        while j < lines.len() && depth > 0 {
            if let Some(decision) = json_star_decision(lines[j].trim()) {
                return Some((j, decision));
            }
            depth += brace_delta(lines[j]);
            j += 1;
        }
        return None;
    }

    if trimmed != "external_directory:" {
        return None;
    }
    let indent = indent_of(lines[i]);
    let mut j = i + 1;
    while j < lines.len() {
        let line = lines[j];
        if line.trim().is_empty() {
            j += 1;
            continue;
        }
        if indent_of(line) <= indent {
            break;
        }
        if let Some(decision) = yaml_star_decision(line.trim()) {
            return Some((j, decision));
        }
        j += 1;
    }
    None
}

fn brace_delta(line: &str) -> i32 {
    line.matches('{').count() as i32 - line.matches('}').count() as i32
}

fn indent_of(line: &str) -> usize {
    line.len() - line.trim_start().len()
}

fn json_star_decision(trimmed: &str) -> Option<String> {
    let (key, value) = trimmed.split_once("\":")?;
    if key.trim() != "\"*" {
        return None;
    }
    is_decision(value.trim().trim_matches(['"', ',']))
}

fn yaml_star_decision(trimmed: &str) -> Option<String> {
    let (key, value) = trimmed.split_once(':')?;
    if key.trim() != "\"*\"" {
        return None;
    }
    is_decision(value.trim())
}

fn is_decision(value: &str) -> Option<String> {
    (value == "ask" || value == "allow").then(|| value.to_owned())
}
