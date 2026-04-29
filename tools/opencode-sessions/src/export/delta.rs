use anyhow::{Context, Result};
use serde::de::DeserializeOwned;
use serde_json::{Map, Value};
use std::collections::HashMap;
use std::fs::{self};
use std::path::Path;

use crate::constants::*;
use crate::models::*;

pub(crate) fn build_iteration_meta(base_dir: &Path, root_name: &str, export_root: &Path) -> Result<IterationMeta> {
    let mut group = fs::read_dir(base_dir)
        .with_context(|| format!("read {}", base_dir.display()))?
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false))
        .filter_map(|entry| entry.file_name().into_string().ok())
        .filter(|name| name == root_name || name.starts_with(&format!("{root_name}-")))
        .collect::<Vec<_>>();
    group.sort_by_key(|name| export_group_sort_key(root_name, name));

    let current_name = export_root
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or(root_name)
        .to_string();
    let iteration_number = group.iter().position(|name| name == &current_name).map(|idx| idx + 1).unwrap_or(group.len().max(1));
    let previous_export_path = group
        .iter()
        .position(|name| name == &current_name)
        .and_then(|idx| idx.checked_sub(1))
        .and_then(|idx| group.get(idx))
        .map(|name| name.to_string());

    Ok(IterationMeta {
        group_key: root_name.to_string(),
        iteration_number,
        previous_export_path,
    })
}

pub(crate) fn export_group_sort_key(root_name: &str, name: &str) -> (usize, usize, String) {
    if name == root_name {
        return (0, 0, name.to_string());
    }
    if let Some(suffix) = name.strip_prefix(&format!("{root_name}-")) {
        return (
            1,
            suffix.parse::<usize>().unwrap_or(usize::MAX),
            name.to_string(),
        );
    }
    (2, usize::MAX, name.to_string())
}

pub(crate) fn json_number_i64(value: Option<&Value>) -> i64 {
    value
        .and_then(|value| value.as_i64().or_else(|| value.as_u64().and_then(|num| i64::try_from(num).ok())))
        .unwrap_or_default()
}

pub(crate) fn json_number_f64(value: Option<&Value>) -> f64 {
    value.and_then(Value::as_f64).unwrap_or_default()
}

pub(crate) fn build_totals_delta(previous_obj: &Map<String, Value>, current: &ExportIndexFile) -> Option<TotalsDelta> {
    let previous = previous_obj.get("totals").and_then(Value::as_object)?;
    let delta = TotalsDelta {
        session_count: current.totals.session_count as i64 - json_number_i64(previous.get("session_count")),
        turn_count: current.totals.turn_count as i64 - json_number_i64(previous.get("turn_count")),
        message_count: current.totals.message_count as i64 - json_number_i64(previous.get("message_count")),
        user_message_count: current.totals.user_message_count as i64 - json_number_i64(previous.get("user_message_count")),
        assistant_message_count: current.totals.assistant_message_count as i64
            - json_number_i64(previous.get("assistant_message_count")),
        text_chars: current.totals.text_chars as i64 - json_number_i64(previous.get("text_chars")),
        reasoning_chars: current.totals.reasoning_chars as i64 - json_number_i64(previous.get("reasoning_chars")),
        tool_calls: current.totals.tool_calls as i64 - json_number_i64(previous.get("tool_calls")),
        input_tokens: current.totals.input_tokens as i64 - json_number_i64(previous.get("input_tokens")),
        output_tokens: current.totals.output_tokens as i64 - json_number_i64(previous.get("output_tokens")),
        reasoning_tokens: current.totals.reasoning_tokens as i64 - json_number_i64(previous.get("reasoning_tokens")),
        cache_read_tokens: current.totals.cache_read_tokens as i64 - json_number_i64(previous.get("cache_read_tokens")),
        cache_write_tokens: current.totals.cache_write_tokens as i64 - json_number_i64(previous.get("cache_write_tokens")),
        cost: current.totals.cost - json_number_f64(previous.get("cost")),
    };
    let has_non_zero = delta.session_count != 0
        || delta.turn_count != 0
        || delta.message_count != 0
        || delta.user_message_count != 0
        || delta.assistant_message_count != 0
        || delta.text_chars != 0
        || delta.reasoning_chars != 0
        || delta.tool_calls != 0
        || delta.input_tokens != 0
        || delta.output_tokens != 0
        || delta.reasoning_tokens != 0
        || delta.cache_read_tokens != 0
        || delta.cache_write_tokens != 0
        || delta.cost != 0.0;
    has_non_zero.then_some(delta)
}

pub(crate) fn build_tool_rollup_deltas(previous_obj: &Map<String, Value>, current: &ExportIndexFile) -> Vec<ToolRollupDelta> {
    let mut previous_by_tool = HashMap::new();
    if let Some(items) = previous_obj.get("tool_rollup").and_then(Value::as_array) {
        for item in items {
            let Some(tool) = item.get("tool").and_then(Value::as_str) else {
                continue;
            };
            previous_by_tool.insert(
                tool.to_string(),
                (
                    json_number_i64(item.get("calls")),
                    json_number_i64(item.get("error_calls")),
                ),
            );
        }
    }

    let mut deltas = current
        .tool_rollup
        .iter()
        .filter_map(|item| {
            let (previous_calls, previous_errors) = previous_by_tool.remove(&item.tool).unwrap_or_default();
            let delta = ToolRollupDelta {
                tool: item.tool.clone(),
                calls_delta: item.calls as i64 - previous_calls,
                error_calls_delta: item.error_calls as i64 - previous_errors,
            };
            (delta.calls_delta != 0 || delta.error_calls_delta != 0).then_some(delta)
        })
        .collect::<Vec<_>>();

    deltas.extend(previous_by_tool.into_iter().filter_map(|(tool, (calls, errors))| {
        let delta = ToolRollupDelta {
            tool,
            calls_delta: -calls,
            error_calls_delta: -errors,
        };
        (delta.calls_delta != 0 || delta.error_calls_delta != 0).then_some(delta)
    }));

    deltas.sort_by(|left, right| {
        right
            .calls_delta
            .abs()
            .cmp(&left.calls_delta.abs())
            .then_with(|| right.error_calls_delta.abs().cmp(&left.error_calls_delta.abs()))
            .then_with(|| left.tool.cmp(&right.tool))
    });
    deltas.truncate(TOOL_ROLLUP_DELTA_LIMIT);
    deltas
}

pub(crate) fn read_jsonl_typed<T: DeserializeOwned>(path: &Path) -> Result<Vec<T>> {
    let text = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    text.lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| serde_json::from_str(line).with_context(|| format!("parse {}", path.display())))
        .collect()
}

pub(crate) fn total_tokens_from_turn_delta(turn: &TurnDeltaDigest) -> u64 {
    turn.input_tokens + turn.output_tokens + turn.reasoning_tokens + turn.cache_read_tokens + turn.cache_write_tokens
}

pub(crate) fn build_turn_deltas(base_dir: &Path, export_root: &Path, previous_obj: &Map<String, Value>, current: &ExportIndexFile) -> Result<Vec<TurnDeltaEntry>> {
    let Some(previous_export_path) = &current.iteration_meta.previous_export_path else {
        return Ok(Vec::new());
    };
    let Some(previous_turns_rel) = previous_obj
        .get("recommended_read_order")
        .and_then(Value::as_array)
        .and_then(|items| items.iter().filter_map(Value::as_str).find(|path| path.ends_with("turns.jsonl")))
    else {
        return Ok(Vec::new());
    };
    let Some(current_turns_rel) = current.recommended_read_order.iter().find(|path| path.ends_with("turns.jsonl")) else {
        return Ok(Vec::new());
    };

    let previous_turns_path = base_dir.join(previous_export_path).join(previous_turns_rel);
    let current_turns_path = export_root.join(current_turns_rel);
    if !previous_turns_path.exists() || !current_turns_path.exists() {
        return Ok(Vec::new());
    }

    let previous_turns = read_jsonl_typed::<TurnDeltaDigest>(&previous_turns_path)?;
    let current_turns = read_jsonl_typed::<TurnDeltaDigest>(&current_turns_path)?;
    let previous_by_turn = previous_turns
        .into_iter()
        .map(|turn| (turn.turn_index, turn))
        .collect::<HashMap<_, _>>();

    let mut deltas = current_turns
        .into_iter()
        .filter_map(|current_turn| {
            let previous_turn = previous_by_turn.get(&current_turn.turn_index);
            let current_total_tokens = total_tokens_from_turn_delta(&current_turn);
            let mut changed_fields = Vec::new();
            if previous_turn.and_then(|turn| turn.agent_strategy.as_deref()) != current_turn.agent_strategy.as_deref() {
                changed_fields.push(String::from("agent_strategy"));
            }
            if previous_turn.and_then(|turn| turn.turn_cost_tier.as_deref()) != current_turn.turn_cost_tier.as_deref() {
                changed_fields.push(String::from("turn_cost_tier"));
            }
            if previous_turn.and_then(|turn| turn.turn_effectiveness.as_deref()) != current_turn.turn_effectiveness.as_deref() {
                changed_fields.push(String::from("turn_effectiveness"));
            }
            if previous_turn.map(total_tokens_from_turn_delta) != Some(current_total_tokens) {
                changed_fields.push(String::from("total_tokens"));
            }
            if previous_turn.map(|turn| turn.tool_call_count) != Some(current_turn.tool_call_count) {
                changed_fields.push(String::from("tool_call_count"));
            }
            if previous_turn.map(|turn| turn.error_count) != Some(current_turn.error_count) {
                changed_fields.push(String::from("error_count"));
            }
            if previous_turn.map(|turn| turn.modified_file_count) != Some(current_turn.modified_file_count) {
                changed_fields.push(String::from("modified_file_count"));
            }
            (!changed_fields.is_empty()).then(|| TurnDeltaEntry {
                turn_index: current_turn.turn_index,
                changed_fields,
                previous_agent_strategy: previous_turn.and_then(|turn| turn.agent_strategy.clone()),
                current_agent_strategy: current_turn.agent_strategy.unwrap_or_else(|| String::from("unknown")),
                previous_turn_cost_tier: previous_turn.and_then(|turn| turn.turn_cost_tier.clone()),
                current_turn_cost_tier: current_turn.turn_cost_tier.unwrap_or_else(|| String::from("unknown")),
                previous_turn_effectiveness: previous_turn.and_then(|turn| turn.turn_effectiveness.clone()),
                current_turn_effectiveness: current_turn
                    .turn_effectiveness
                    .unwrap_or_else(|| String::from("unknown")),
                previous_total_tokens: previous_turn.map(total_tokens_from_turn_delta),
                current_total_tokens,
                previous_tool_call_count: previous_turn.map(|turn| turn.tool_call_count),
                current_tool_call_count: current_turn.tool_call_count,
                previous_error_count: previous_turn.map(|turn| turn.error_count),
                current_error_count: current_turn.error_count,
                previous_modified_file_count: previous_turn.map(|turn| turn.modified_file_count),
                current_modified_file_count: current_turn.modified_file_count,
            })
        })
        .collect::<Vec<_>>();

    deltas.sort_by(|left, right| {
        right
            .current_total_tokens
            .abs_diff(right.previous_total_tokens.unwrap_or_default())
            .cmp(&left.current_total_tokens.abs_diff(left.previous_total_tokens.unwrap_or_default()))
            .then_with(|| left.turn_index.cmp(&right.turn_index))
    });
    deltas.truncate(TURN_DELTA_LIMIT);
    Ok(deltas)
}

pub(crate) fn build_delta_from_previous(base_dir: &Path, export_root: &Path, current: &ExportIndexFile) -> Result<Option<DeltaFromPrevious>> {
    let Some(previous_export_path) = &current.iteration_meta.previous_export_path else {
        return Ok(None);
    };
    let previous_index_path = base_dir.join(previous_export_path).join("index.json");
    if !previous_index_path.exists() {
        return Ok(None);
    }
    let previous_value: Value = serde_json::from_str(
        &fs::read_to_string(&previous_index_path).with_context(|| format!("read {}", previous_index_path.display()))?,
    )
    .with_context(|| format!("parse {}", previous_index_path.display()))?;
    let previous_obj = previous_value.as_object().cloned().unwrap_or_default();
    let current_value = serde_json::to_value(current).map_err(|err| anyhow::anyhow!(err.to_string()))?;
    let current_obj = current_value.as_object().cloned().unwrap_or_default();
    let current_fields = [
        "schema_version",
        "schema_file",
        "fields_file",
        "iteration_meta",
        "classification_policy",
        "artifact_policy",
        "hotspots",
        "tool_rollup",
        "session_index",
        "totals",
        "root_session_status",
        "token_efficiency",
    ];
    let mut added_index_fields = Vec::new();
    let mut changed_index_fields = Vec::new();
    let mut removed_index_fields = Vec::new();
    for field in current_fields {
        match (previous_obj.get(field), current_obj.get(field)) {
            (Some(_), None) => removed_index_fields.push(field.to_string()),
            (Some(previous), Some(_)) => {
                if current_obj.get(field) != Some(previous) {
                    changed_index_fields.push(field.to_string());
                }
            }
            (None, Some(_)) => added_index_fields.push(field.to_string()),
            (None, None) => {}
        }
    }
    Ok(Some(DeltaFromPrevious {
        previous_export_path: previous_export_path.clone(),
        previous_schema_version: previous_obj
            .get("schema_version")
            .and_then(Value::as_str)
            .unwrap_or("unknown")
            .to_string(),
        current_schema_version: current.schema_version.to_string(),
        added_index_fields,
        removed_index_fields,
        changed_index_fields,
        totals_delta: build_totals_delta(&previous_obj, current),
        tool_rollup_deltas: build_tool_rollup_deltas(&previous_obj, current),
        turn_deltas: build_turn_deltas(base_dir, export_root, &previous_obj, current)?,
    }))
}
