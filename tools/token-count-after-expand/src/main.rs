use anyhow::{Context, Result, bail};
use std::env;
use std::fs;
use std::io::{self, BufRead};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::Duration;
use wait_timeout::ChildExt;

fn bytes_per_token() -> f64 {
    env::var("BYTES_PER_TOKEN")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(3.0)
}

fn find_repo_root() -> Result<PathBuf> {
    let cwd = env::current_dir().context("get current directory")?;
    for dir in cwd.ancestors() {
        if dir.join("config/opencode.json").is_file()
            && dir.join("plugins/opencode-plugin-md-expand").is_dir()
        {
            return Ok(dir.to_path_buf());
        }
    }
    bail!("could not find repo root from {}", cwd.display())
}

fn strip_frontmatter(text: &str) -> String {
    let mut lines = text.split_inclusive('\n').collect::<Vec<_>>();
    if lines.is_empty() || lines[0].trim() != "---" {
        return text.to_string();
    }
    for index in 1..lines.len() {
        let stripped = lines[index].trim();
        if stripped == "---" || stripped == "..." {
            return lines.split_off(index + 1).concat();
        }
    }
    text.to_string()
}

fn render_and_count(root: &Path, path: &str) -> (String, usize, Option<String>) {
    let mut child = match Command::new("bun")
        .args([
            "plugins/opencode-plugin-md-expand/src/cli/cli.ts",
            "render",
            path,
        ])
        .current_dir(root)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
    {
        Ok(child) => child,
        Err(_) => return (path.to_string(), 0, Some("bun not found".to_string())),
    };

    match child.wait_timeout(Duration::from_secs(30)) {
        Ok(Some(_)) => {}
        Ok(None) => {
            let _ = child.kill();
            let _ = child.wait();
            return (path.to_string(), 0, Some("timeout (30s)".to_string()));
        }
        Err(err) => return (path.to_string(), 0, Some(err.to_string())),
    }

    let output = match child.wait_with_output() {
        Ok(output) => output,
        Err(err) => return (path.to_string(), 0, Some(err.to_string())),
    };

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        let message = if stderr.is_empty() {
            format!("exit code {}", output.status.code().unwrap_or(1))
        } else {
            stderr
        };
        return (path.to_string(), 0, Some(message));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let cleaned = strip_frontmatter(&stdout);
    (path.to_string(), cleaned.chars().count(), None)
}

fn walk_md(dir: &Path, out: &mut Vec<String>) -> Result<()> {
    let mut entries = fs::read_dir(dir)
        .with_context(|| format!("read directory {}", dir.display()))?
        .collect::<std::result::Result<Vec<_>, _>>()?;
    entries.sort_by_key(|entry| entry.path());
    for entry in entries {
        let path = entry.path();
        let ty = entry.file_type()?;
        if ty.is_dir() {
            walk_md(&path, out)?;
        } else if ty.is_file() && path.extension().and_then(|v| v.to_str()) == Some("md") {
            out.push(path.to_string_lossy().into_owned());
        }
    }
    Ok(())
}

fn paths_from_args(args: &[String]) -> Result<Vec<String>> {
    let mut paths = Vec::new();
    if args.is_empty() {
        for line in io::stdin().lock().lines() {
            let line = line?;
            let trimmed = line.trim();
            if !trimmed.is_empty() {
                paths.push(trimmed.to_string());
            }
        }
    } else {
        for arg in args {
            let path = Path::new(arg);
            if path.is_file() {
                paths.push(arg.clone());
            } else if path.is_dir() {
                walk_md(path, &mut paths)?;
            } else {
                eprintln!("skip: not found: {}", arg);
            }
        }
    }
    Ok(paths)
}

fn main() -> Result<()> {
    let args: Vec<String> = env::args().skip(1).collect();
    let paths = paths_from_args(&args)?;
    if paths.is_empty() {
        if args.is_empty() {
            eprintln!("Usage: token-count-after-expand [paths...]");
        } else {
            eprintln!("error: no valid paths found");
        }
        std::process::exit(1);
    }

    let root = find_repo_root()?;
    let bytes_per_token = bytes_per_token();
    let mut total = 0_i64;
    let mut count = 0_u64;
    let mut errors = 0_u64;

    for path in paths {
        let (path, chars, err) = render_and_count(&root, &path);
        if let Some(err) = err {
            eprintln!("! {}  render failed: {}", path, err);
            errors += 1;
            continue;
        }
        let tokens = (chars as f64 / bytes_per_token) as i64;
        total += tokens;
        count += 1;
        println!("{}  {}", tokens, path);
    }

    if count > 0 {
        println!("{}  total", total);
    }
    println!("# {} chars/token", bytes_per_token);
    if errors > 0 {
        eprintln!("! {} errors", errors);
    }
    Ok(())
}
