use anyhow::{Context, Result};
use clap::Parser;
use std::collections::BTreeSet;
use std::env;
use std::fs;
use std::io::{self, BufRead};
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Parser, Debug)]
#[command(about = "Chunk files by estimated token count")]
struct Args {
    /// Max tokens per chunk.
    #[arg(short, long, default_value_t = 32_000, value_name = "TOKENS")]
    size: u64,

    /// Files or directories. If omitted, paths are read from stdin.
    paths: Vec<PathBuf>,
}

fn bytes_per_token() -> f64 {
    env::var("BYTES_PER_TOKEN")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(3.0)
}

fn git_root(dir: &Path) -> Option<PathBuf> {
    let out = Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(["rev-parse", "--show-toplevel"])
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let root = String::from_utf8(out.stdout).ok()?;
    Some(PathBuf::from(root.trim()))
}

fn git_list_files(dir: &Path) -> Option<Vec<PathBuf>> {
    let root = git_root(dir)?;
    let rel = dir.strip_prefix(&root).unwrap_or(dir);
    let mut cmd = Command::new("git");
    cmd.arg("-C").arg(&root).args([
        "ls-files",
        "--cached",
        "--others",
        "--exclude-standard",
        "--",
    ]);
    if !rel.as_os_str().is_empty() {
        cmd.arg(rel);
    }
    let out = cmd.output().ok()?;
    if !out.status.success() {
        return None;
    }
    let stdout = String::from_utf8(out.stdout).ok()?;
    let mut paths = Vec::new();
    for line in stdout.lines().filter(|line| !line.is_empty()) {
        let path = root.join(line);
        if path.is_file() {
            paths.push(path);
        }
    }
    Some(paths)
}

fn walk_files(dir: &Path, out: &mut Vec<PathBuf>) -> Result<()> {
    let mut entries = fs::read_dir(dir)
        .with_context(|| format!("read directory {}", dir.display()))?
        .collect::<std::result::Result<Vec<_>, _>>()?;
    entries.sort_by_key(|entry| entry.path());
    for entry in entries {
        let path = entry.path();
        let file_type = entry.file_type()?;
        if file_type.is_dir() {
            walk_files(&path, out)?;
        } else if file_type.is_file() {
            out.push(path);
        }
    }
    Ok(())
}

fn expand(path: &Path) -> Result<Vec<PathBuf>> {
    if path.is_file() {
        return Ok(vec![path.to_path_buf()]);
    }
    if path.is_dir() {
        if let Some(paths) = git_list_files(path) {
            return Ok(paths);
        }
        let mut paths = Vec::new();
        walk_files(path, &mut paths)?;
        return Ok(paths);
    }
    Ok(Vec::new())
}

fn input_paths(args: &[PathBuf]) -> Result<Vec<PathBuf>> {
    let mut paths = Vec::new();
    if args.is_empty() {
        for line in io::stdin().lock().lines() {
            let line = line?;
            let trimmed = line.trim();
            if !trimmed.is_empty() {
                paths.extend(expand(Path::new(trimmed))?);
            }
        }
    } else {
        for path in args {
            paths.extend(expand(path)?);
        }
    }

    let mut seen = BTreeSet::new();
    paths.retain(|path| seen.insert(path.clone()));
    Ok(paths)
}

fn tokens(path: &Path, bytes_per_token: f64) -> Option<u64> {
    let size = fs::metadata(path).ok()?.len();
    if size == 0 {
        return None;
    }
    Some((size as f64 / bytes_per_token) as u64)
}

fn main() -> Result<()> {
    let args = Args::parse();
    let bytes_per_token = bytes_per_token();
    let max_bytes = (args.size as f64 * bytes_per_token) as u64;
    let paths = input_paths(&args.paths)?;

    let mut chunk_index = 0_u64;
    let mut chunk_bytes = 0_u64;
    let mut chunk_tokens = 0_u64;
    let mut chunk_items: Vec<(PathBuf, u64)> = Vec::new();
    let mut total_tokens = 0_u64;
    let mut total_files = 0_u64;

    let flush = |chunk_index: &mut u64,
                 chunk_items: &mut Vec<(PathBuf, u64)>,
                 chunk_tokens: &mut u64,
                 total_tokens: &mut u64,
                 total_files: &mut u64| {
        if chunk_items.is_empty() {
            return;
        }
        *chunk_index += 1;
        if *chunk_index > 1 {
            println!();
        }
        println!("chunk {}: {}", chunk_index, chunk_tokens);
        for (path, count) in chunk_items.iter() {
            println!("{} {}", count, path.display());
        }
        *total_tokens += *chunk_tokens;
        *total_files += chunk_items.len() as u64;
        chunk_items.clear();
        *chunk_tokens = 0;
    };

    for path in paths {
        let Some(count) = tokens(&path, bytes_per_token) else {
            continue;
        };
        let bytes = (count as f64 * bytes_per_token) as u64;
        if !chunk_items.is_empty() && chunk_bytes + bytes > max_bytes {
            flush(
                &mut chunk_index,
                &mut chunk_items,
                &mut chunk_tokens,
                &mut total_tokens,
                &mut total_files,
            );
            chunk_bytes = 0;
        }
        chunk_bytes += bytes;
        chunk_tokens += count;
        chunk_items.push((path, count));
    }

    flush(
        &mut chunk_index,
        &mut chunk_items,
        &mut chunk_tokens,
        &mut total_tokens,
        &mut total_files,
    );

    println!(
        "\nchunks: {}  files: {}  tokens: {}",
        chunk_index, total_files, total_tokens
    );
    println!(
        "target: {}/chunk  ({} bytes/token)",
        args.size, bytes_per_token
    );
    Ok(())
}
