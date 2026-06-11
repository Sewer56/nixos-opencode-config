use anyhow::{Context, Result};
use regex::Regex;
use std::collections::BTreeSet;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};

fn generate_id() -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos()
        .to_string();
    let suffix = if nanos.len() > 6 {
        &nanos[nanos.len() - 6..]
    } else {
        &nanos
    };
    format!("STAT-{}", suffix)
}

fn repo_root() -> Result<PathBuf> {
    let cwd = env::current_dir().context("get current directory")?;
    for dir in cwd.ancestors() {
        if dir.join(".git").exists() || dir.join("config/opencode.json").is_file() {
            return Ok(dir.to_path_buf());
        }
    }
    Ok(cwd)
}

fn run_git(args: &[&str]) -> (i32, String, String) {
    match Command::new("git").args(args).output() {
        Ok(output) => (
            output.status.code().unwrap_or(1),
            String::from_utf8_lossy(&output.stdout).into_owned(),
            String::from_utf8_lossy(&output.stderr).into_owned(),
        ),
        Err(err) => (1, String::new(), err.to_string()),
    }
}

fn add_finding(
    findings: &mut Vec<String>,
    ids: &mut Vec<String>,
    severity: &str,
    path: &str,
    problem: &str,
    fix: &str,
) {
    let id = generate_id();
    findings.push(format!(
        "| {} | {} | {} | {} | {} |\n",
        id, severity, path, problem, fix
    ));
    ids.push(id);
}

fn first_line(text: &str) -> &str {
    text.lines().next().unwrap_or("")
}

fn path_exists(path: &str) -> bool {
    Path::new(path).is_file()
}

fn render(path: &str) -> (bool, String, String) {
    match Command::new("bun")
        .args([
            "plugins/opencode-plugin-md-expand/src/cli/cli.ts",
            "render",
            path,
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
    {
        Ok(output) => (
            output.status.success(),
            String::from_utf8_lossy(&output.stdout).into_owned(),
            String::from_utf8_lossy(&output.stderr).into_owned(),
        ),
        Err(err) => (false, String::new(), err.to_string()),
    }
}

fn ids_out(ids: &[String]) -> String {
    if ids.is_empty() {
        return "None".to_string();
    }
    let joined = ids.join(", ");
    match joined.rsplit_once(',') {
        Some((head, _)) => head.to_string(),
        None => joined,
    }
}

fn main() -> Result<()> {
    let artifact_base = env::args().nth(1).unwrap_or_default();
    if artifact_base.is_empty() {
        eprintln!("Usage: iterate-static-check <artifact_base>");
        std::process::exit(2);
    }

    let root = repo_root()?;
    env::set_current_dir(&root).with_context(|| format!("chdir {}", root.display()))?;

    let state_path = format!("{}.prep.md", artifact_base);
    let log_path = format!("{}.md", artifact_base);
    let result_path = format!("{}.static-check.md", artifact_base);

    if !Path::new(&state_path).is_file() || !Path::new(&log_path).is_file() {
        eprintln!(
            "FAIL: missing prep state ({}) or edit log ({})",
            state_path, log_path
        );
        std::process::exit(2);
    }

    let mut findings = Vec::new();
    let mut ids = Vec::new();
    let artifact_prefix = format!("{}.", artifact_base);

    let (_, changed_stdout, _) = run_git(&["diff", "--name-only", "HEAD"]);
    let (_, untracked_stdout, _) = run_git(&["ls-files", "--others", "--exclude-standard"]);
    let mut all_paths = BTreeSet::new();
    for line in changed_stdout.lines().chain(untracked_stdout.lines()) {
        if !line.is_empty() && !line.starts_with(&artifact_prefix) {
            all_paths.insert(line.to_string());
        }
    }

    let import_re = Regex::new(r#"\{\{[^}]*file="[^"]*opencode-source/"#)?;
    let agent_ref_re = Regex::new(r"@[A-Za-z][A-Za-z0-9_./-]+")?;
    let prefixes = [
        ".opencode/agent/",
        "config/agent/",
        ".opencode/command/",
        "config/command/",
    ];

    for path in &all_paths {
        if !path_exists(path) || !path.ends_with(".md") {
            continue;
        }
        let content = fs::read_to_string(path).with_context(|| format!("read {}", path))?;

        if import_re.is_match(&content) {
            add_finding(
                &mut findings,
                &mut ids,
                "BLOCKING",
                path,
                "renderer import points into opencode-source/",
                "use a local config/ or .opencode/ path or remove the import",
            );
        }

        let is_agent_or_command = prefixes.iter().any(|prefix| path.starts_with(prefix));
        if is_agent_or_command && first_line(&content) != "---" {
            add_finding(
                &mut findings,
                &mut ids,
                "BLOCKING",
                path,
                "missing opening frontmatter delimiter",
                "add '---' as the first line",
            );
        }

        let mut refs = BTreeSet::new();
        for mat in agent_ref_re.find_iter(&content) {
            refs.insert(mat.as_str().trim_start_matches('@').to_string());
        }
        for reference in refs {
            let name = reference
                .split_once('/')
                .map(|(_, name)| name)
                .unwrap_or(&reference);
            let dot_path = format!(".opencode/agent/{}.md", name);
            let config_path = format!("config/agent/{}.md", name);
            if !Path::new(&dot_path).is_file() && !Path::new(&config_path).is_file() {
                add_finding(
                    &mut findings,
                    &mut ids,
                    "BLOCKING",
                    path,
                    &format!("unresolved @agent reference: {}", reference),
                    "create the agent file or fix the reference",
                );
            }
        }
    }

    for path in &all_paths {
        if !path_exists(path) || !path.ends_with(".md") {
            continue;
        }
        if !prefixes.iter().any(|prefix| path.starts_with(prefix)) {
            continue;
        }

        let (ok, stdout, stderr) = render(path);
        if !ok {
            let err = first_line(stderr.trim()).to_string();
            add_finding(
                &mut findings,
                &mut ids,
                "BLOCKING",
                path,
                &format!("renderer failed: {}", err),
                "fix the source template or import, then re-render",
            );
            continue;
        }

        let rendered_lines: Vec<&str> = stdout.lines().collect();
        if rendered_lines
            .iter()
            .any(|line| line.trim_end_matches(|c| c == ' ' || c == '\t') != *line)
        {
            add_finding(
                &mut findings,
                &mut ids,
                "BLOCKING",
                path,
                "rendered output has trailing whitespace",
                "remove trailing spaces from the source prompt",
            );
        }

        let mut prev_blank = false;
        for line in &rendered_lines {
            let blank = line.trim().is_empty();
            if prev_blank && blank {
                add_finding(
                    &mut findings,
                    &mut ids,
                    "BLOCKING",
                    path,
                    "rendered output has consecutive blank lines",
                    "collapse repeated blank lines in the source prompt",
                );
                break;
            }
            prev_blank = blank;
        }

        let mut fence: Option<&str> = None;
        let mut opened_line = 0_usize;
        for (idx, line) in rendered_lines.iter().enumerate() {
            let stripped = line.trim();
            if stripped.starts_with("```") {
                if fence.is_none() {
                    fence = Some("```");
                    opened_line = idx + 1;
                } else if fence == Some("```") {
                    fence = None;
                    opened_line = 0;
                }
            } else if stripped.starts_with("~~~") {
                if fence.is_none() {
                    fence = Some("~~~");
                    opened_line = idx + 1;
                } else if fence == Some("~~~") {
                    fence = None;
                    opened_line = 0;
                }
            }
        }
        if fence.is_some() {
            add_finding(
                &mut findings,
                &mut ids,
                "BLOCKING",
                path,
                &format!(
                    "rendered output has an unclosed markdown fence starting at line {}",
                    opened_line
                ),
                "close the fence or switch inner examples to the other fence marker",
            );
        }
    }

    let diff_check = Command::new("git")
        .args(["diff", "--check", "HEAD"])
        .stderr(Stdio::piped())
        .stdout(Stdio::piped())
        .output()
        .context("run git diff --check")?;
    if !diff_check.status.success() {
        let mut output = String::from_utf8_lossy(&diff_check.stdout).into_owned();
        output.push_str(&String::from_utf8_lossy(&diff_check.stderr));
        for line in output.lines().filter(|line| !line.trim().is_empty()) {
            add_finding(
                &mut findings,
                &mut ids,
                "BLOCKING",
                "git-diff",
                line,
                "fix whitespace in the named file",
            );
        }
    }

    let mut finding_table =
        "| ID | Severity | Path | Problem | Fix |\n|----|----------|------|---------|-----|\n"
            .to_string();
    let decision;
    let ids_text;
    let summary;
    if findings.is_empty() {
        decision = "PASS";
        finding_table.push_str("| None | none | None | no findings | None |\n");
        ids_text = "None".to_string();
        summary = "static check passed".to_string();
    } else {
        decision = "BLOCKING";
        finding_table.push_str(&findings.concat());
        ids_text = ids_out(&ids);
        summary = format!(
            "static check found {} (and possibly more) BLOCKING findings",
            ids_text
        );
    }

    let changed_lines: Vec<String> = all_paths
        .iter()
        .take(50)
        .map(|p| format!("- {}", p))
        .collect();
    let changed_block = if changed_lines.is_empty() {
        "- None".to_string()
    } else {
        changed_lines.join("\n")
    };
    let body = format!(
        "# Iterate Edit Static Check\nSchema: v1\nDecision: {}\n\n## Changed Paths\n{}\n\n## Findings\n{}\n## Verified\n- None\n",
        decision, changed_block, finding_table
    );
    fs::write(&result_path, body).with_context(|| format!("write {}", result_path))?;

    let changed_csv = if all_paths.is_empty() {
        "(none)".to_string()
    } else {
        all_paths.iter().cloned().collect::<Vec<_>>().join(",")
    };
    println!("# STATIC CHECK");
    println!("Decision: {}", decision);
    println!("Result: {}", root.join(&result_path).display());
    println!("Changed Paths: {}", changed_csv);
    println!("IDs: {}", ids_text);
    println!("Summary: {}", summary);

    if decision == "BLOCKING" {
        std::process::exit(1);
    }
    Ok(())
}
