use anyhow::{Context, Result};
use serde::Serialize;
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::fs::{self, File};
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};

use crate::format::*;
use crate::models::*;

pub(crate) struct ArtifactWriter<'a> {
    pub(crate) artifacts_dir: &'a Path,
    pub(crate) artifacts_rel_dir: &'a Path,
    pub(crate) artifacts_created: &'a mut bool,
    pub(crate) artifact_count: &'a mut usize,
}

pub(crate) fn build_deliverable_snapshot(
    path: &str,
    session_dir: &Path,
    relative_session_dir: &Path,
) -> Result<Option<DeliverableSnapshot>> {
    let Some(source_path) = resolve_workspace_deliverable_path(path) else {
        return Ok(None);
    };
    if !source_path.exists() || !source_path.is_file() {
        return Ok(None);
    }

    let text = fs::read_to_string(&source_path).with_context(|| format!("read {}", source_path.display()))?;
    let snapshot_rel = relative_session_dir.join("deliverables").join(path);
    let snapshot_path = session_dir.join("deliverables").join(path);
    if let Some(parent) = snapshot_path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    fs::write(&snapshot_path, &text).with_context(|| format!("write {}", snapshot_path.display()))?;

    Ok(Some(DeliverableSnapshot {
        snapshot_file: path_string(&snapshot_rel),
        content_sha256: sha256_hex(text.as_bytes()),
        line_count: text.lines().count(),
        snapshot_source: String::from("workspace-current"),
    }))
}

pub(crate) fn resolve_workspace_deliverable_path(path: &str) -> Option<PathBuf> {
    if path.trim().is_empty() || path.starts_with("external/") {
        return None;
    }
    let relative = Path::new(path);
    if relative.is_absolute() {
        return None;
    }
    if relative.components().any(|component| matches!(component, std::path::Component::ParentDir)) {
        return None;
    }
    Some(repo_root_dir().join(relative))
}

pub(crate) fn repo_root_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .unwrap_or_else(|| Path::new(env!("CARGO_MANIFEST_DIR")))
        .to_path_buf()
}

pub(crate) fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

pub(crate) fn ensure_artifacts_dir(created: &mut bool, dir: &Path) -> Result<()> {
    if !*created {
        fs::create_dir_all(dir).with_context(|| format!("create {}", dir.display()))?;
        *created = true;
    }
    Ok(())
}

pub(crate) fn write_text_artifact(
    aw: &mut ArtifactWriter,
    file_name: &str,
    text: &str,
    force: bool,
    threshold: usize,
) -> Result<Option<String>> {
    if text.trim().is_empty() {
        return Ok(None);
    }
    if !force && text.chars().count() <= threshold {
        return Ok(None);
    }
    ensure_artifacts_dir(aw.artifacts_created, aw.artifacts_dir)?;
    let path = aw.artifacts_dir.join(file_name);
    fs::write(&path, text).with_context(|| format!("write {}", path.display()))?;
    *aw.artifact_count += 1;
    Ok(Some(path_string(&aw.artifacts_rel_dir.join(file_name))))
}

pub(crate) fn write_json_artifact(
    aw: &mut ArtifactWriter,
    file_name: &str,
    value: &Value,
    force: bool,
    threshold: usize,
) -> Result<Option<String>> {
    let text = serde_json::to_string(value).map_err(|err| anyhow::anyhow!(err.to_string()))?;
    if !force && text.len() <= threshold {
        return Ok(None);
    }
    ensure_artifacts_dir(aw.artifacts_created, aw.artifacts_dir)?;
    let path = aw.artifacts_dir.join(file_name);
    fs::write(&path, text).with_context(|| format!("write {}", path.display()))?;
    *aw.artifact_count += 1;
    Ok(Some(path_string(&aw.artifacts_rel_dir.join(file_name))))
}

pub(crate) fn write_artifacts_manifest(
    session_dir: &Path,
    relative_session_dir: &Path,
) -> Result<(Option<String>, Vec<ArtifactManifestEntry>)> {
    let artifacts_dir = session_dir.join("artifacts");
    let artifacts_rel_dir = relative_session_dir.join("artifacts");
    if !artifacts_dir.exists() {
        return Ok((None, Vec::new()));
    }

    let mut entries = fs::read_dir(&artifacts_dir)
        .with_context(|| format!("read {}", artifacts_dir.display()))?
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.file_type().map(|ft| ft.is_file()).unwrap_or(false))
        .filter_map(|entry| {
            let name = entry.file_name().into_string().ok()?;
            if name == "index.json" {
                return None;
            }
            let metadata = entry.metadata().ok()?;
            let (category, message_index, tool_index) = classify_artifact_manifest_entry(&name);
            Some(ArtifactManifestEntry {
                path: path_string(&artifacts_rel_dir.join(&name)),
                category,
                size_bytes: metadata.len(),
                message_index,
                tool_index,
            })
        })
        .collect::<Vec<_>>();
    entries.sort_by(|left, right| left.path.cmp(&right.path));

    write_json_pretty(
        artifacts_dir.join("index.json"),
        &ArtifactManifestFile {
            artifacts_dir: path_string(&artifacts_rel_dir),
            total_size_bytes: entries.iter().map(|entry| entry.size_bytes).sum(),
            entries: entries.clone(),
        },
    )?;
    Ok((Some(path_string(&artifacts_rel_dir.join("index.json"))), entries))
}

pub(crate) fn classify_artifact_manifest_entry(name: &str) -> (String, Option<usize>, Option<usize>) {
    if let Some(rest) = name.strip_prefix("message-") {
        let (message_index, suffix) = rest.split_once('-').unwrap_or((rest, ""));
        let message_index = message_index.parse::<usize>().ok();
        let category = if suffix == "reasoning.txt" {
            "message-reasoning"
        } else if suffix == "prompt.txt" {
            "message-prompt"
        } else if suffix == "user.txt" {
            "message-user"
        } else if suffix == "text.txt" {
            "message-text"
        } else {
            "message-other"
        };
        return (String::from(category), message_index, None);
    }

    if let Some(rest) = name.strip_prefix("tool-") {
        let mut parts = rest.splitn(3, '-');
        let message_index = parts.next().and_then(|value| value.parse::<usize>().ok());
        let tool_index = parts.next().and_then(|value| value.parse::<usize>().ok());
        let suffix = parts.next().unwrap_or_default();
        let category = if suffix == "input.json" {
            "tool-input"
        } else if suffix == "patch.diff" {
            "tool-patch"
        } else if suffix == "output.txt" {
            "tool-output"
        } else if suffix == "error.txt" {
            "tool-error"
        } else {
            "tool-other"
        };
        return (String::from(category), message_index, tool_index);
    }

    (String::from("other"), None, None)
}

pub(crate) fn write_json_pretty(path: PathBuf, value: &impl Serialize) -> Result<()> {
    let file = File::create(&path).with_context(|| format!("create {}", path.display()))?;
    let writer = BufWriter::new(file);
    serde_json::to_writer_pretty(writer, value).with_context(|| format!("write {}", path.display()))?;
    Ok(())
}

pub(crate) fn write_text(path: PathBuf, text: &str) -> Result<()> {
    fs::write(&path, text).with_context(|| format!("write {}", path.display()))?;
    Ok(())
}

pub(crate) fn write_jsonl(path: PathBuf, lines: &[Value]) -> Result<()> {
    let file = File::create(&path).with_context(|| format!("create {}", path.display()))?;
    let mut writer = BufWriter::new(file);
    for line in lines {
        serde_json::to_writer(&mut writer, line).with_context(|| format!("write line to {}", path.display()))?;
        writer.write_all(b"\n")?;
    }
    writer.flush()?;
    Ok(())
}

pub(crate) fn unique_child_dir(base: &Path, name: &str) -> Result<PathBuf> {
    let mut candidate = base.join(name);
    let mut suffix = 2usize;
    while candidate.exists() {
        candidate = base.join(format!("{name}-{suffix}"));
        suffix += 1;
    }
    fs::create_dir_all(&candidate).with_context(|| format!("create {}", candidate.display()))?;
    Ok(candidate)
}

pub(crate) fn default_export_base_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("exports")
}

pub(crate) fn session_folder_name(
    is_root: bool,
    session_path: &str,
    agent: Option<&str>,
    title: &str,
    session_id: &str,
) -> String {
    let prefix = if is_root { "root" } else { "subagent" };
    let agent = agent.map(sanitize_filename).unwrap_or_else(|| String::from("unknown"));
    format!(
        "{}__{}__{}__{}",
        session_path.replace('.', "-"),
        prefix,
        agent,
        sanitize_filename(&format!("{}-{}", title, short_id(session_id))),
    )
}
