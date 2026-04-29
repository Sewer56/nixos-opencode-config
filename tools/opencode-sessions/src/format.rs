use anyhow::Result;
use chrono::{DateTime, Local, TimeZone, Utc};
use serde::Serialize;
use serde_json::{Map, Value, json};
use std::path::Path;

use crate::constants::*;
use crate::models::*;

pub(crate) fn average_u64(total: u64, count: usize) -> Option<f64> {
    (count > 0).then_some(total as f64 / count as f64)
}

pub(crate) fn average_usize(total: usize, count: usize) -> Option<f64> {
    (count > 0).then_some(total as f64 / count as f64)
}

pub(crate) fn sample_strings(mut items: Vec<String>, limit: usize) -> Vec<String> {
    items.sort();
    items.truncate(limit);
    items
}

pub(crate) fn normalize_tool_path(path: &str) -> String {
    let mut trimmed = path.trim().replace('\\', "/");
    if trimmed.is_empty() {
        return trimmed;
    }
    let repo_root_markers = [
        "home-manager/programs/opencode",
        "home-manager/programs/opencode/",
        "/home/sewer/nixos/users/sewer/home-manager/programs/opencode",
        "/home/sewer/nixos/users/sewer/home-manager/programs/opencode/",
    ];
    for marker in repo_root_markers {
        if trimmed == marker {
            return String::from(".");
        }
        if let Some((_, suffix)) = trimmed.rsplit_once(marker) {
            trimmed = suffix.to_string();
            break;
        }
        if let Some(index) = trimmed.find(marker) {
            trimmed = trimmed[index + marker.len()..].to_string();
            break;
        }
    }
    if trimmed.is_empty() {
        return String::from(".");
    }
    let external_root_markers = ["/home/sewer/Project/", "home/sewer/Project/"];
    for marker in external_root_markers {
        if let Some((_, suffix)) = trimmed.rsplit_once(marker) {
            return format!("external/{}", suffix);
        }
    }
    let segments = trimmed.split('/').filter(|part| !part.is_empty()).collect::<Vec<_>>();
    if let Some(index) = segments.iter().position(|segment| segment.contains("__export-")) {
        return format!("exports/{}", segments[index..].join("/"));
    }
    if let Some(index) = segments
        .iter()
        .position(|segment| segment.starts_with("0__root__") || segment.contains("__subagent__"))
    {
        if index > 0 && segments[index - 1] == "sessions" {
            return segments[index - 1..].join("/");
        }
        return format!("sessions/{}", segments[index..].join("/"));
    }
    let candidates = [
        "tools/opencode-sessions/",
        "opencode-source/packages/",
        "opencode/",
    ];
    for marker in candidates {
        if let Some((_, suffix)) = trimmed.rsplit_once(marker) {
            return format!("{marker}{suffix}");
        }
        if let Some(index) = trimmed.find(marker) {
            return trimmed[index..].to_string();
        }
    }

    if trimmed.starts_with("/home/") || trimmed.starts_with("home/") {
        if segments.len() > 2 {
            return format!("external/{}", segments[2..].join("/"));
        }
        return format!("external/{}", segments.join("/"));
    }

    if segments.len() <= PATH_FALLBACK_COMPONENTS {
        return segments.join("/");
    }
    segments[segments.len() - PATH_FALLBACK_COMPONENTS..].join("/")
}

pub(crate) fn normalize_tool_input_preview(tool: &str, mut preview: Value) -> Value {
    let Some(object) = preview.as_object_mut() else {
        return preview;
    };
    for key in ["filePath", "path", "workdir"] {
        if let Some(value) = object.get_mut(key)
            && let Some(path) = value.as_str()
        {
            *value = Value::String(normalize_tool_path(path));
        }
    }
    if tool == "bash"
        && let Some(Value::String(description)) = object.get("description")
    {
        object.insert(String::from("description"), Value::String(truncate_text(description, TOOL_PART_PREVIEW_LIMIT)));
    }
    preview
}

pub(crate) fn to_json_values<T: Serialize>(items: &[T]) -> Result<Vec<Value>> {
    items
        .iter()
        .map(serde_json::to_value)
        .collect::<Result<Vec<_>, _>>()
        .map_err(|err| anyhow::anyhow!(err.to_string()))
}

pub(crate) fn token_total(tokens: Option<&TokenStatsExport>) -> u64 {
    tokens
        .map(|tokens| tokens.total.unwrap_or(tokens.input + tokens.output + tokens.reasoning + tokens.cache_read + tokens.cache_write))
        .unwrap_or_default()
}

pub(crate) fn join_blocks(blocks: &[&str]) -> String {
    blocks
        .iter()
        .map(|block| block.trim())
        .filter(|block| !block.is_empty())
        .collect::<Vec<_>>()
        .join("\n\n")
}

pub(crate) fn path_string(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

pub(crate) fn is_zero_usize(value: &usize) -> bool {
    *value == 0
}

pub(crate) fn is_one_usize(value: &usize) -> bool {
    *value == 1
}

pub(crate) fn is_zero_u64(value: &u64) -> bool {
    *value == 0
}

pub(crate) fn is_zero_i64(value: &i64) -> bool {
    *value == 0
}

pub(crate) fn is_zero_f64(value: &f64) -> bool {
    *value == 0.0
}

pub(crate) fn is_completed_status(value: &str) -> bool {
    value == "completed"
}

pub(crate) fn non_empty_owned(value: Option<&str>) -> Option<String> {
    value.map(str::trim).filter(|value| !value.is_empty()).map(str::to_string)
}

pub(crate) fn sanitize_filename(input: &str) -> String {
    let mut out = String::new();
    for ch in input.chars().flat_map(char::to_lowercase) {
        if ch.is_ascii_alphanumeric() {
            out.push(ch);
        } else if matches!(ch, ' ' | '-' | '_' | '.')
            && !out.ends_with('-') {
                out.push('-');
            }
    }
    let trimmed = out.trim_matches('-');
    if trimmed.is_empty() {
        String::from("untitled")
    } else {
        trimmed.chars().take(80).collect()
    }
}

pub(crate) fn extract_subagent_from_title(title: &str) -> Option<String> {
    let start = title.rfind("(@")? + 2;
    let rest = &title[start..];
    let end = rest.find(" subagent)")?;
    let agent = rest[..end].trim();
    (!agent.is_empty()).then(|| agent.to_string())
}

pub(crate) fn short_id(session_id: &str) -> String {
    session_id.chars().take(12).collect()
}

pub(crate) fn truncate_text(text: &str, limit: usize) -> String {
    let text = text.trim();
    let count = text.chars().count();
    if count <= limit {
        return text.to_string();
    }
    let prefix = text.chars().take(limit).collect::<String>();
    format!("{}…(+{} chars)", prefix, count.saturating_sub(limit))
}

pub(crate) fn shrink_json(value: &Value, max_string: usize, max_items: usize, depth: usize) -> Value {
    if depth == 0 {
        return Value::String(String::from("[truncated-depth]"));
    }

    match value {
        Value::Null | Value::Bool(_) | Value::Number(_) => value.clone(),
        Value::String(text) => Value::String(truncate_text(text, max_string)),
        Value::Array(items) => {
            let mut out = items
                .iter()
                .take(max_items)
                .map(|item| shrink_json(item, max_string, max_items, depth - 1))
                .collect::<Vec<_>>();
            if items.len() > max_items {
                out.push(json!({"truncated_items": items.len() - max_items}));
            }
            Value::Array(out)
        }
        Value::Object(map) => {
            let mut out = Map::new();
            for (index, (key, item)) in map.iter().enumerate() {
                if key == "metadata" {
                    continue;
                }
                if index >= max_items {
                    out.insert(String::from("truncated_keys"), json!(map.len() - max_items));
                    break;
                }
                out.insert(
                    key.clone(),
                    shrink_json(item, max_string, max_items, depth - 1),
                );
            }
            Value::Object(out)
        }
    }
}

pub(crate) fn format_duration(duration_ms: i64) -> String {
    if duration_ms < 1000 {
        return format!("{}ms", duration_ms.max(0));
    }
    let seconds = duration_ms / 1000;
    let hours = seconds / 3600;
    let minutes = (seconds % 3600) / 60;
    let secs = seconds % 60;

    if hours > 0 {
        format!("{}h{:02}m{:02}s", hours, minutes, secs)
    } else if minutes > 0 {
        format!("{}m{:02}s", minutes, secs)
    } else {
        format!("{}s", secs)
    }
}

pub(crate) fn format_local_timestamp(ms: i64) -> String {
    Local
        .timestamp_millis_opt(ms)
        .single()
        .map(|value| value.format("%Y-%m-%d %H:%M:%S").to_string())
        .unwrap_or_else(|| format!("{ms}"))
}

pub(crate) fn format_timestamp_slug(ms: i64) -> String {
    Utc.timestamp_millis_opt(ms)
        .single()
        .map(|value| value.format("%Y%m%d-%H%M%S").to_string())
        .unwrap_or_else(|| String::from("unknown-time"))
}

pub(crate) fn format_system_time(time: std::time::SystemTime) -> String {
    let datetime: DateTime<Local> = time.into();
    datetime.format("%Y-%m-%d %H:%M:%S").to_string()
}

pub(crate) fn format_bytes(bytes: u64) -> String {
    const KB: f64 = 1024.0;
    const MB: f64 = KB * 1024.0;
    const GB: f64 = MB * 1024.0;
    let value = bytes as f64;
    if value >= GB {
        return format!("{:.1}G", value / GB);
    }
    if value >= MB {
        return format!("{:.1}M", value / MB);
    }
    if value >= KB {
        return format!("{:.1}K", value / KB);
    }
    format!("{}B", bytes)
}
