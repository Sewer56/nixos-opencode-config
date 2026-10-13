//! Toggle OpenCode permission `ask` rules so unattended runs never wait.
//!
//! - `status`: count rules still waiting for approval and rules yolo rewrote.
//! - `on`: rewrite every `ask` in agent frontmatter and `config/opencode.json`.
//! - `off`: restore every rewritten rule to `ask`.

use anyhow::{Context, Result, bail};
use clap::{Parser, Subcommand};
use ignore::WalkBuilder;
use rule::{Mode, RuleLine, Syntax};
use std::fs;
use std::path::{Path, PathBuf};

mod rule;

#[derive(Parser, Debug)]
#[command(about = "Toggle OpenCode permission ask rules between regular and yolo mode")]
struct Args {
    #[command(subcommand)]
    cmd: Option<Cmd>,

    /// Repository root override. Default: walk upward from CWD.
    #[arg(long, value_name = "PATH")]
    repo: Option<PathBuf>,
}

#[derive(Subcommand, Debug)]
enum Cmd {
    /// Allow every ask; deny path and command guards so runs fail fast.
    On,

    /// Restore every yolo-rewritten rule to ask.
    Off,

    /// Report current mode without changing anything.
    Status,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let root = find_repo(args.repo)?;
    let targets = collect_targets(&root);
    if targets.is_empty() {
        bail!(
            "no agent markdown or config/opencode.json under {}",
            root.display()
        );
    }
    match args.cmd.unwrap_or(Cmd::Status) {
        Cmd::Status => status(&targets),
        Cmd::On => apply(&targets, Mode::Yolo),
        Cmd::Off => apply(&targets, Mode::Regular),
    }
}

fn apply(targets: &[(PathBuf, Syntax)], to: Mode) -> Result<()> {
    // Rewrite each file that has rules in the other mode
    let (mut rules, mut files) = (0_usize, 0_usize);
    for (path, syntax) in targets {
        let text = read(path)?;
        let (out, changed) = toggle(&text, *syntax, to);
        if changed > 0 {
            fs::write(path, out).with_context(|| format!("write {}", path.display()))?;
            rules += changed;
            files += 1;
        }
    }

    // Report the change and how to pick it up
    let label = match to {
        Mode::Yolo => "on",
        Mode::Regular => "off",
    };
    println!("yolo mode {label}: rewrote {rules} rules in {files} files");
    println!("restart opencode for the change to take effect");
    Ok(())
}

/// Agent markdown under both agent directories, plus the global JSONC config.
fn collect_targets(root: &Path) -> Vec<(PathBuf, Syntax)> {
    // Gather agent markdown from global and repo-local agent directories
    let mut targets: Vec<(PathBuf, Syntax)> = [
        root.join("config").join("agent"),
        root.join(".opencode").join("agent"),
    ]
    .iter()
    .filter(|dir| dir.is_dir())
    .flat_map(|dir| WalkBuilder::new(dir).hidden(false).build().flatten())
    .map(ignore::DirEntry::into_path)
    .filter(|path| path.extension().is_some_and(|ext| ext == "md"))
    .map(|path| (path, Syntax::Yaml))
    .collect();

    // Add the global config last so output order stays stable
    targets.sort_unstable_by(|a, b| a.0.cmp(&b.0));
    let json = root.join("config").join("opencode.json");
    if json.is_file() {
        targets.push((json, Syntax::Json));
    }
    targets
}

/// Find the repository root: `--repo`, or the nearest ancestor of CWD
/// holding `config/opencode.json`.
fn find_repo(override_root: Option<PathBuf>) -> Result<PathBuf> {
    // Trust an explicit root only if it looks like this repository
    if let Some(root) = override_root {
        if !root.join("config").join("opencode.json").is_file() {
            bail!("{} has no config/opencode.json", root.display());
        }
        return Ok(root);
    }

    // Otherwise search the current directory and its ancestors
    let cwd = std::env::current_dir().context("get current directory")?;
    cwd.ancestors()
        .find(|dir| dir.join("config").join("opencode.json").is_file())
        .map(Path::to_path_buf)
        .context("opencode config repository not found; run from it or pass --repo")
}

fn status(targets: &[(PathBuf, Syntax)]) -> Result<()> {
    // Count toggleable rules in each mode across all files
    let (mut regular, mut yolo) = (0_usize, 0_usize);
    for (path, syntax) in targets {
        let text = read(path)?;
        for rule in rule_lines(&text, *syntax).filter_map(|(_, rule)| rule) {
            match rule.mode() {
                Some(Mode::Regular) => regular += 1,
                Some(Mode::Yolo) => yolo += 1,
                None => {}
            }
        }
    }

    // Summarize the overall mode
    let mode = match (yolo, regular) {
        (0, _) => "OFF (regular)",
        (_, 0) => "ON (yolo)",
        _ => "MIXED: run `opencode-yolo-mode on` or `off`",
    };
    println!("yolo mode: {mode}");
    println!("  rewritten by yolo: {yolo}  waiting for approval: {regular}");
    Ok(())
}

fn read(path: &Path) -> Result<String> {
    fs::read_to_string(path).with_context(|| format!("read {}", path.display()))
}

/// Rewrite every rule in `text` not yet in `to`, keeping all other bytes.
///
/// # Returns
///
/// The new text and the number of rewritten rules.
fn toggle(text: &str, syntax: Syntax, to: Mode) -> (String, usize) {
    let mut out = String::with_capacity(text.len() + 512);
    let mut changed = 0;
    for (line, rule) in rule_lines(text, syntax) {
        match rule.and_then(|rule| rule.rewrite(to)) {
            Some(new) => {
                out.push_str(&new);
                out.push_str(&line[line.trim_end_matches(['\r', '\n']).len()..]);
                changed += 1;
            }
            None => out.push_str(line),
        }
    }
    (out, changed)
}

/// Each line with its ending, paired with its rule if it can hold permissions.
///
/// # Remarks
///
/// Markdown only scans the frontmatter.
/// Rule examples in agent bodies stay untouched.
fn rule_lines(text: &str, syntax: Syntax) -> impl Iterator<Item = (&str, Option<RuleLine<'_>>)> {
    let limit = match syntax {
        Syntax::Json => usize::MAX,
        Syntax::Yaml => frontmatter_end(text),
    };
    text.split_inclusive('\n')
        .enumerate()
        .map(move |(idx, line)| {
            let body = line.trim_end_matches(['\r', '\n']);
            let rule = if idx < limit {
                RuleLine::parse(body, syntax)
            } else {
                None
            };
            (line, rule)
        })
}

/// Index of the closing `---` line, or 0 when the file has no frontmatter.
fn frontmatter_end(text: &str) -> usize {
    let mut lines = text.lines();
    if lines.next().map(str::trim) != Some("---") {
        return 0;
    }
    lines
        .position(|line| line.trim() == "---")
        .map_or(0, |pos| pos + 1)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    const AGENT: &str = "---\n\
        mode: subagent\n\
        permissions:\n  \
        - { action: \"*\", resource: \"*\", effect: deny }\n  \
        - { action: external_directory, resource: \"*\", effect: ask }\n  \
        - { action: shell, resource: \"git push *\", effect: ask }\n\
        ---\n\
        Example:\n  \
        - { action: edit, resource: \"*\", effect: ask }\n";

    // ── Toggle ──────────────────────────────────────────────────────────────

    #[rstest]
    fn toggle_should_rewrite_frontmatter_and_restore_exact_bytes() {
        // Act
        let (yolo, on) = toggle(AGENT, Syntax::Yaml, Mode::Yolo);
        let (regular, off) = toggle(&yolo, Syntax::Yaml, Mode::Regular);

        // Assert
        assert_eq!(on, 2);
        assert!(yolo.contains(r#"resource: "*", effect: allow } # yolo"#));
        assert!(yolo.contains(r#"resource: "git push *", effect: deny } # yolo"#));
        assert!(yolo.ends_with("  - { action: edit, resource: \"*\", effect: ask }\n"));
        assert_eq!(off, 2);
        assert_eq!(regular, AGENT);
    }
}
