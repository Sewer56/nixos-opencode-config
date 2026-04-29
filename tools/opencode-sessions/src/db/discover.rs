use anyhow::{Context, Result, bail};
use rusqlite::{Connection, OpenFlags};
use std::fs::{self};
use std::path::{Path, PathBuf};
use std::time::Duration;

use crate::format::*;

pub(crate) fn print_discovered_dbs(explicit: Option<&Path>) -> Result<()> {
    let discovered = discover_db_paths()?;
    let default = resolve_db_path(explicit).ok();

    if discovered.is_empty() {
        bail!("no OpenCode sqlite files found under {}", opencode_data_dir()?.display());
    }

    for path in discovered {
        let metadata = fs::metadata(&path).with_context(|| format!("read metadata for {}", path.display()))?;
        let modified = metadata.modified().ok().map(format_system_time).unwrap_or_else(|| "unknown".into());
        let mark = if default.as_deref() == Some(path.as_path()) {
            "*"
        } else {
            " "
        };
        println!(
            "{} {}  size={}  modified={}",
            mark,
            path.display(),
            format_bytes(metadata.len()),
            modified
        );
    }

    Ok(())
}

pub(crate) fn opencode_data_dir() -> Result<PathBuf> {
    dirs::home_dir()
        .map(|home| home.join(".local/share/opencode"))
        .context("could not resolve home directory")
}

pub(crate) fn discover_db_paths() -> Result<Vec<PathBuf>> {
    let data_dir = opencode_data_dir()?;
    if !data_dir.is_dir() {
        return Ok(Vec::new());
    }

    let mut found = Vec::new();
    for entry in fs::read_dir(&data_dir).with_context(|| format!("read {}", data_dir.display()))? {
        let entry = entry?;
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let Some(name) = path.file_name().and_then(|value| value.to_str()) else {
            continue;
        };
        if name == "opencode.db" || (name.starts_with("opencode-") && name.ends_with(".db")) {
            found.push(path);
        }
    }

    found.sort_by(|left, right| {
        let left_meta = fs::metadata(left).ok();
        let right_meta = fs::metadata(right).ok();
        let left_mtime = left_meta.and_then(|meta| meta.modified().ok());
        let right_mtime = right_meta.and_then(|meta| meta.modified().ok());
        right_mtime
            .cmp(&left_mtime)
            .then_with(|| left.file_name().cmp(&right.file_name()))
    });
    Ok(found)
}

pub(crate) fn resolve_db_path(explicit: Option<&Path>) -> Result<PathBuf> {
    if let Some(path) = explicit {
        let path = if path.is_absolute() {
            path.to_path_buf()
        } else {
            std::env::current_dir()?.join(path)
        };
        if !path.is_file() {
            bail!("db file not found: {}", path.display());
        }
        return Ok(path);
    }

    if let Some(env_db) = std::env::var_os("OPENCODE_DB") {
        let raw = PathBuf::from(env_db);
        if raw.as_os_str() != ":memory:" {
            let resolved = if raw.is_absolute() { raw } else { opencode_data_dir()?.join(raw) };
            if resolved.is_file() {
                return Ok(resolved);
            }
        }
    }

    let discovered = discover_db_paths()?;
    discovered
        .into_iter()
        .next()
        .context("no OpenCode sqlite database found; use --db to point at one")
}

pub(crate) fn open_db(path: &Path) -> Result<Connection> {
    let flags = OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX;
    let conn = Connection::open_with_flags(path, flags)
        .with_context(|| format!("open sqlite db {}", path.display()))?;
    conn.busy_timeout(Duration::from_secs(5))?;
    conn.pragma_update(None, "query_only", true)?;
    Ok(conn)
}
