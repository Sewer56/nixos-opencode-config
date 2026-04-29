use serde_json::Value;
use std::collections::{BTreeSet, HashMap, HashSet};

use crate::constants::*;
use crate::format::*;
use crate::models::*;
use crate::export::classify::*;

pub(crate) fn extract_export_reference_paths(text: &str) -> Vec<String> {
    let mut paths = Vec::new();
    let mut seen = HashSet::new();
    for token in text.split_whitespace() {
        if !token.contains("__export-") {
            continue;
        }
        let cleaned = token
            .trim_matches(|c: char| matches!(c, '`' | '"' | '\'' | ',' | '.' | ')' | '(' | ']' | '['))
            .trim();
        if cleaned.is_empty() {
            continue;
        }
        let normalized = normalize_tool_path(cleaned);
        if seen.insert(normalized.clone()) {
            paths.push(normalized);
        }
    }
    paths
}

pub(crate) fn classify_export_reference_status(paths: &[String], current_export_name: &str) -> Option<String> {
    if paths.is_empty() {
        return None;
    }
    let current_count = paths.iter().filter(|path| path.contains(current_export_name)).count();
    if current_count == paths.len() {
        return Some(String::from("current-export"));
    }
    if current_count > 0 {
        return Some(String::from("mixed-export"));
    }
    Some(String::from("stale-export"))
}

pub(crate) fn resolve_current_export_paths(paths: &[String], current_export_name: &str) -> Vec<String> {
    let current_root = format!("exports/{current_export_name}");
    let mut out = Vec::new();
    let mut seen = HashSet::new();
    for path in paths {
        let resolved = path
            .find("exports/")
            .map(|index| &path[index + "exports/".len()..])
            .map(|suffix| {
                let mut parts = suffix.splitn(2, '/');
                let _old_root = parts.next().unwrap_or_default();
                let rest = parts.next().unwrap_or_default();
                if rest.is_empty() {
                    current_root.clone()
                } else {
                    format!("{current_root}/{rest}")
                }
            })
            .unwrap_or_else(|| current_root.clone());
        if seen.insert(resolved.clone()) {
            out.push(resolved);
        }
    }
    out
}

pub(crate) fn build_resolved_prompt_preview(paths: &[String], current_export_name: &str) -> Option<String> {
    let resolved = resolve_current_export_paths(paths, current_export_name);
    if resolved.is_empty() || resolved == paths {
        return None;
    }
    Some(truncate_text(
        &format!("Resolved export refs: {}", resolved.join(", ")),
        SUBTASK_PREVIEW_LIMIT,
    ))
}

pub(crate) fn infer_snapshot_completeness(session_status: &str, staleness_ms: i64) -> String {
    if session_status == "completed" {
        return String::from("final");
    }
    if session_status == "running" {
        if staleness_ms >= 43_200_000 {
            return String::from("stale-running-snapshot");
        }
        return String::from("live-running-snapshot");
    }
    String::from("partial")
}

pub(crate) fn map_child_delegations(
    session: &LoadedSession,
    _compact_messages: &[CompactMessage],
    current_export_name: &str,
) -> HashMap<String, ChildDelegationInfo> {
    let mut by_id = HashMap::new();

    for (message_index, loaded) in session.messages.iter().enumerate() {
        for (tool_index, part) in loaded
            .parts
            .iter()
            .filter(|part| part.raw.get("type").and_then(Value::as_str) == Some("tool"))
            .enumerate()
        {
            if part.raw.get("tool").and_then(Value::as_str) != Some("task") {
                continue;
            }
            let Some(state) = part.raw.get("state") else {
                continue;
            };
            let output = state.get("output").and_then(Value::as_str).unwrap_or("");
            let Some(task_id) = extract_task_id(output) else {
                continue;
            };
            let input = state.get("input");
            let description = input
                .and_then(|value| value.get("description"))
                .and_then(Value::as_str)
                .map(|value| truncate_text(value.trim(), TOOL_PART_PREVIEW_LIMIT));
            let prompt_preview = input
                .and_then(|value| value.get("prompt"))
                .and_then(Value::as_str)
                .map(|value| truncate_text(value.trim(), SUBTASK_PREVIEW_LIMIT));
            let prompt_export_paths = input
                .and_then(|value| value.get("prompt"))
                .and_then(Value::as_str)
                .map(extract_export_reference_paths)
                .unwrap_or_default();
            let prompt_preview_resolved = build_resolved_prompt_preview(&prompt_export_paths, current_export_name);
            let export_reference_status = classify_export_reference_status(&prompt_export_paths, current_export_name);

            by_id.insert(
                task_id,
                ChildDelegationInfo {
                    message_index,
                    tool_index,
                    description,
                    prompt_preview,
                    prompt_preview_resolved,
                    prompt_export_paths,
                    export_reference_status,
                    input_file: None,
                },
            );
        }
    }

    by_id
}

pub(crate) fn collect_turn_reasoning_themes(messages: &[MessageDigest], range: Option<&std::ops::RangeInclusive<usize>>) -> Vec<String> {
    let Some(range) = range else {
        return Vec::new();
    };
    let mut seen = HashSet::new();
    let mut themes = Vec::new();
    for idx in range.clone() {
        for theme in &messages[idx].reasoning_themes {
            if seen.insert(theme.clone()) {
                themes.push(theme.clone());
            }
            if themes.len() >= 5 {
                return themes;
            }
        }
    }
    themes
}

pub(crate) fn build_turn_reasoning_summary(
    messages: &[MessageDigest],
    range: Option<&std::ops::RangeInclusive<usize>>,
    themes: &[String],
) -> Option<String> {
    let range = range?;
    let mut snippets = Vec::new();
    let mut seen = HashSet::new();
    for idx in range.clone() {
        let Some(summary) = messages[idx].reasoning_summary.as_ref() else {
            continue;
        };
        let summary = truncate_text(summary, 120);
        if seen.insert(summary.clone()) {
            snippets.push(summary);
        }
        if snippets.len() >= 2 {
            break;
        }
    }
    if !snippets.is_empty() {
        return Some(truncate_text(&snippets.join(" | "), REASONING_SUMMARY_LIMIT));
    }
    (!themes.is_empty()).then(|| truncate_text(&format!("Themes: {}", themes.join(", ")), REASONING_SUMMARY_LIMIT))
}

pub(crate) fn build_turn_digests(
    session_path: &str,
    messages: &[MessageDigest],
    tools: &[ToolCallDigest],
    child_links: &[ChildLink],
) -> Vec<TurnDigest> {
    let user_indexes = messages
        .iter()
        .enumerate()
        .filter_map(|(index, message)| (message.role == "user").then_some(index))
        .collect::<Vec<_>>();
    let child_by_parent = child_links
        .iter()
        .filter_map(|child| {
            child
                .parent_message_index
                .zip(child.parent_tool_index)
                .map(|(message_index, tool_index)| ((message_index, tool_index), child))
        })
        .collect::<HashMap<_, _>>();

    let mut turns = Vec::new();
    for (turn_index, &user_idx) in user_indexes.iter().enumerate() {
        let user = &messages[user_idx];
        let next_user_idx = user_indexes.get(turn_index + 1).copied();
        let assistant_start = (user_idx + 1..messages.len()).find(|&idx| messages[idx].role != "user");
        let assistant_end = next_user_idx.map(|idx| idx.saturating_sub(1)).or_else(|| messages.len().checked_sub(1));
        let assistant_range = assistant_start
            .zip(assistant_end)
            .filter(|(start, end)| start <= end)
            .map(|(start, end)| start..=end);

        let mut assistant_message_count = 0usize;
        let mut assistant_duration_ms = 0i64;
        let mut input_tokens = 0u64;
        let mut output_tokens = 0u64;
        let mut reasoning_tokens = 0u64;
        let mut cache_read_tokens = 0u64;
        let mut cache_write_tokens = 0u64;

        if let Some(range) = assistant_range.clone() {
            for idx in range {
                let message = &messages[idx];
                assistant_message_count += 1;
                assistant_duration_ms += message.duration_ms.unwrap_or_default();
                if let Some(tokens) = &message.tokens {
                    input_tokens += tokens.input;
                    output_tokens += tokens.output;
                    reasoning_tokens += tokens.reasoning;
                    cache_read_tokens += tokens.cache_read;
                    cache_write_tokens += tokens.cache_write;
                }
            }
        }

        let tool_slice = tools
            .iter()
            .filter(|tool| {
                tool.message_index > user.message_index
                    && next_user_idx
                        .map(|next_idx| tool.message_index < messages[next_idx].message_index)
                        .unwrap_or(true)
            })
            .collect::<Vec<_>>();
        let tool_call_count = tool_slice.len();
        let tool_duration_ms = tool_slice.iter().map(|tool| tool.duration_ms.unwrap_or_default()).sum();
        let error_count = tool_slice.iter().filter(|tool| tool.status == "error").count();
        let total_read_events = tool_slice.iter().map(|tool| tool.read_paths.len()).sum::<usize>();
        let mut change_stats = TurnChangeStats {
            patch_calls: 0,
            files_added: 0,
            files_updated: 0,
            files_deleted: 0,
            files_moved: 0,
            lines_added: 0,
            lines_deleted: 0,
        };
        let mut change_intents = BTreeSet::new();
        let mut tool_rollup_map: HashMap<String, TurnToolAggregate> = HashMap::new();
        let mut call_purpose_map: HashMap<String, usize> = HashMap::new();
        let mut delegation_previews = Vec::new();
        let mut read_files = BTreeSet::new();
        let mut modified_files = BTreeSet::new();
        let mut strongest_patch: Option<(&ToolCallDigest, usize)> = None;

        for tool in tool_slice {
            let entry = tool_rollup_map.entry(tool.tool.clone()).or_insert_with(|| TurnToolAggregate {
                tool: tool.tool.clone(),
                calls: 0,
                error_calls: 0,
            });
            entry.calls += 1;
            if tool.status == "error" {
                entry.error_calls += 1;
            }
            if let Some(purpose) = &tool.call_purpose {
                *call_purpose_map.entry(purpose.clone()).or_default() += 1;
            }
            read_files.extend(tool.read_paths.iter().cloned());
            modified_files.extend(tool.modified_paths.iter().cloned());
            if let Some(summary) = &tool.patch_summary {
                change_stats.patch_calls += 1;
                change_stats.files_added += summary.files_added;
                change_stats.files_updated += summary.files_updated;
                change_stats.files_deleted += summary.files_deleted;
                change_stats.files_moved += summary.files_moved;
                change_stats.lines_added += summary.lines_added;
                change_stats.lines_deleted += summary.lines_deleted;
                let churn = summary.lines_added + summary.lines_deleted;
                let replace = strongest_patch.as_ref().map(|(_, best)| churn > *best).unwrap_or(true);
                if replace {
                    strongest_patch = Some((tool, churn));
                }
            }
            if let Some(intent) = &tool.patch_intent {
                change_intents.insert(intent.clone());
            }

            if let Some(child) = child_by_parent.get(&(tool.message_index, tool.tool_index)) {
                delegation_previews.push(TurnDelegationPreview {
                    session_path: child.session_path.clone(),
                    session_id: child.session_id.clone(),
                    agent: child.agent.clone(),
                    parent_message_index: tool.message_index,
                    parent_tool_index: tool.tool_index,
                });
            }
        }

        let mut tool_rollup = tool_rollup_map.into_values().collect::<Vec<_>>();
        tool_rollup.sort_by(|left, right| right.calls.cmp(&left.calls).then_with(|| left.tool.cmp(&right.tool)));
        let mut call_purpose_rollup = call_purpose_map
            .into_iter()
            .map(|(purpose, calls)| TurnPurposeAggregate { purpose, calls })
            .collect::<Vec<_>>();
        call_purpose_rollup.sort_by(|left, right| right.calls.cmp(&left.calls).then_with(|| left.purpose.cmp(&right.purpose)));

        let response_elapsed_ms = assistant_range
            .as_ref()
            .and_then(|range| messages.get(*range.end()))
            .map(|last| last.time_ms.saturating_sub(user.time_ms) + last.duration_ms.unwrap_or_default());
        let wall_to_next_user_ms = next_user_idx
            .and_then(|idx| messages.get(idx))
            .map(|next| next.time_ms.saturating_sub(user.time_ms))
            .unwrap_or_default();
        let final_assistant = assistant_range
            .as_ref()
            .and_then(|range| messages.get(*range.end()));
        let total_tokens = input_tokens + output_tokens + reasoning_tokens + cache_read_tokens + cache_write_tokens;
        let cache_hit_ratio = (total_tokens > 0)
            .then_some(cache_read_tokens as f64 / total_tokens as f64)
            .filter(|ratio| ratio.is_finite());
        let tokens_per_tool_call = average_u64(total_tokens, tool_call_count).filter(|value| value.is_finite());
        let read_file_count = read_files.len();
        let modified_file_count = modified_files.len();
        let read_files_all = read_files.iter().cloned().collect::<Vec<_>>();
        let modified_files_all = modified_files.iter().cloned().collect::<Vec<_>>();
        let retry_ratio = average_usize(error_count, tool_call_count);
        let redundant_read_ratio = (total_read_events > 0)
            .then_some(total_read_events.saturating_sub(read_file_count) as f64 / total_read_events as f64);
        let read_file_sample_limit = if redundant_read_ratio.unwrap_or_default() >= 0.3 {
            3
        } else {
            TURN_FILE_SAMPLE_LIMIT
        };
        let modified_file_sample_limit = if modified_file_count > 1 { 3 } else { TURN_FILE_SAMPLE_LIMIT };

        let (user_intent, user_intent_confidence) = classify_user_intent(user.text_preview.as_deref());
        let alternative_user_intents = classify_alternative_user_intents(
            user.text_preview.as_deref(),
            &user_intent,
            user_intent_confidence,
        );
        let user_tags = classify_user_tags(user.text_preview.as_deref());
        let next_user_classification = next_user_idx
            .and_then(|idx| messages.get(idx))
            .filter(|next| next.role == "user")
            .map(|next| classify_user_intent(next.text_preview.as_deref()));
        let next_user_intent = next_user_classification.as_ref().map(|(intent, _)| intent.clone());
        let next_user_intent_confidence = next_user_classification.map(|(_, confidence)| confidence);
        let next_user_tags = next_user_idx
            .and_then(|idx| messages.get(idx))
            .filter(|next| next.role == "user")
            .map(|next| classify_user_tags(next.text_preview.as_deref()))
            .unwrap_or_default();

        let outcome = infer_turn_outcome(
            tool_call_count,
            &delegation_previews,
            next_user_intent.as_deref(),
            next_user_tags.as_slice(),
        );
        let final_assistant_text_preview = final_assistant
            .and_then(|message| message.text_preview.as_deref())
            .map(|text| truncate_text(text, TURN_PREVIEW_LIMIT));
        let final_assistant_kind = classify_assistant_text_kind(
            final_assistant.and_then(|message| message.text_preview.as_deref()),
        );
        let agent_strategy = infer_agent_strategy(&tool_rollup, error_count, &delegation_previews, modified_file_count);
        let success = infer_turn_success(
            error_count,
            &outcome,
            final_assistant_kind.as_deref(),
            final_assistant_text_preview.is_some(),
            modified_file_count,
            &tool_rollup,
        );
        let turn_cost_tier = classify_turn_cost_tier(total_tokens, tool_call_count, response_elapsed_ms);
        let turn_change_summary = summarize_turn_change(
            modified_file_count,
            &modified_files,
            final_assistant_text_preview.as_deref(),
            outcome.as_str(),
        );
        let turn_reasoning_themes = collect_turn_reasoning_themes(messages, assistant_range.as_ref());
        let turn_reasoning_summary = build_turn_reasoning_summary(messages, assistant_range.as_ref(), &turn_reasoning_themes);
        let key_diff_preview = strongest_patch
            .and_then(|(tool, _)| build_key_diff_preview(tool.patch_summary.as_ref(), tool.patch_intent.as_deref()));

        turns.push(TurnDigest {
            session_path: session_path.to_string(),
            turn_index,
            user_message_index: user.message_index,
            message_index_end: final_assistant
                .map(|message| message.message_index)
                .unwrap_or(user.message_index),
            time_ms: user.time_ms,
            user_intent,
            user_intent_confidence,
            user_tags,
            alternative_user_intents,
            user_text_preview: user.text_preview.clone(),
            user_text_file: user.text_file.clone(),
            assistant_message_start: assistant_range.as_ref().map(|range| messages[*range.start()].message_index),
            assistant_message_end: assistant_range.as_ref().map(|range| messages[*range.end()].message_index),
            assistant_message_count,
            response_elapsed_ms,
            wall_to_next_user_ms,
            assistant_duration_ms,
            tool_duration_ms,
            tool_call_count,
            error_count,
            delegation_count: delegation_previews.len(),
            input_tokens,
            output_tokens,
            reasoning_tokens,
            cache_read_tokens,
            cache_write_tokens,
            cache_hit_ratio,
            total_tokens,
            tokens_per_tool_call,
            read_file_count,
            read_files: sample_strings(read_files.clone().into_iter().collect(), read_file_sample_limit),
            modified_file_count,
            modified_files: sample_strings(modified_files.clone().into_iter().collect(), modified_file_sample_limit),
            tool_rollup,
            call_purpose_rollup,
            delegations: delegation_previews,
            delegations_file: None,
            final_assistant_message_index: final_assistant.map(|message| message.message_index),
            final_assistant_text_preview,
            final_assistant_text_file: final_assistant.and_then(|message| message.text_file.clone()),
            final_assistant_kind,
            agent_strategy,
            outcome,
            success,
            turn_cost_tier,
            turn_effectiveness: String::new(),
            recommended_attention: String::new(),
            effectiveness_signals: TurnEffectivenessSignals {
                files_modified_count: modified_file_count,
                files_survived_to_end: 0,
                retry_ratio,
                redundant_read_ratio,
            },
            failure_narrative: None,
            optimization_hints: Vec::new(),
            reasoning_summary: turn_reasoning_summary,
            reasoning_themes: turn_reasoning_themes,
            turn_change_summary,
            change_stats,
            change_intents: change_intents.into_iter().collect(),
            key_diff_preview,
            next_user_message_index: next_user_idx.map(|idx| messages[idx].message_index),
            next_user_intent,
            next_user_intent_confidence,
            next_user_tags,
            read_files_all,
            modified_files_all,
        });
    }

    finalize_turn_effectiveness(&mut turns);

    turns
}

pub(crate) fn finalize_turn_effectiveness(turns: &mut [TurnDigest]) {
    let mut last_modified_turn: HashMap<(String, String), usize> = HashMap::new();
    for turn in turns.iter() {
        for path in &turn.modified_files_all {
            last_modified_turn.insert((turn.session_path.clone(), path.clone()), turn.turn_index);
        }
    }

    for turn in turns.iter_mut() {
        let files_survived_to_end = turn
            .modified_files_all
            .iter()
            .filter(|path| last_modified_turn.get(&(turn.session_path.clone(), (*path).clone())) == Some(&turn.turn_index))
            .count();
        turn.effectiveness_signals.files_survived_to_end = files_survived_to_end;
        turn.turn_effectiveness = classify_turn_effectiveness(turn, files_survived_to_end);
        turn.recommended_attention = recommend_turn_attention(turn);
        turn.failure_narrative = build_failure_narrative(turn, files_survived_to_end);
        turn.optimization_hints = build_optimization_hints(turn);
    }
}

pub(crate) fn extract_task_id(output: &str) -> Option<String> {
    let prefix = "task_id:";
    output
        .lines()
        .find_map(|line| line.trim().strip_prefix(prefix).map(str::trim))
        .and_then(|value| value.split_whitespace().next())
        .map(str::to_string)
        .filter(|value| !value.is_empty())
}

pub(crate) fn summarize_patch_text(text: &str) -> Option<PatchSummary> {
    if text.trim().is_empty() {
        return None;
    }

    let mut added = 0usize;
    let mut updated = 0usize;
    let mut deleted = 0usize;
    let mut moved = 0usize;
    let mut hunk_count = 0usize;
    let mut added_lines = 0usize;
    let mut removed_lines = 0usize;
    let sample_paths = sample_strings(patch_paths_from_text(text), PATCH_FILE_SAMPLE_LIMIT);

    for line in text.lines() {
        if line.starts_with("*** Add File:") {
            added += 1;
        } else if line.starts_with("*** Update File:") {
            updated += 1;
        } else if line.starts_with("*** Delete File:") {
            deleted += 1;
        } else if line.starts_with("*** Move to:") {
            moved += 1;
        } else if line.starts_with("@@") {
            hunk_count += 1;
        } else if line.starts_with('+') && !line.starts_with("+++") {
            added_lines += 1;
        } else if line.starts_with('-') && !line.starts_with("---") {
            removed_lines += 1;
        }
    }

    Some(PatchSummary {
        files_added: added,
        files_updated: updated,
        files_deleted: deleted,
        files_moved: moved,
        hunks: hunk_count,
        lines_added: added_lines,
        lines_deleted: removed_lines,
        sample_paths,
    })
}

pub(crate) fn render_patch_summary(summary: &PatchSummary) -> String {
    let mut parts = Vec::new();
    if summary.files_added > 0 {
        parts.push(format!("add {}", summary.files_added));
    }
    if summary.files_updated > 0 {
        parts.push(format!("update {}", summary.files_updated));
    }
    if summary.files_deleted > 0 {
        parts.push(format!("delete {}", summary.files_deleted));
    }
    if summary.files_moved > 0 {
        parts.push(format!("move {}", summary.files_moved));
    }
    if summary.hunks > 0 {
        parts.push(format!("hunks {}", summary.hunks));
    }
    if summary.lines_added > 0 || summary.lines_deleted > 0 {
        parts.push(format!("lines +{}/-{}", summary.lines_added, summary.lines_deleted));
    }
    if parts.is_empty() {
        return String::from("patch");
    }
    parts.join(", ")
}

pub(crate) fn build_key_diff_preview(summary: Option<&PatchSummary>, intent: Option<&str>) -> Option<String> {
    let summary = summary?;
    let mut parts = Vec::new();
    if let Some(intent) = intent {
        parts.push(intent.to_string());
    }
    parts.push(render_patch_summary(summary));
    if !summary.sample_paths.is_empty() {
        parts.push(summary.sample_paths.iter().take(3).cloned().collect::<Vec<_>>().join(", "));
    }
    Some(truncate_text(&parts.join(" | "), TURN_PREVIEW_LIMIT))
}

pub(crate) fn classify_patch_intent(paths: &[String], summary: &PatchSummary) -> Option<String> {
    let lower_paths = paths.iter().map(|path| path.to_lowercase()).collect::<Vec<_>>();
    let has_code = lower_paths.iter().any(|path| {
        path.ends_with(".rs")
            || path.ends_with(".ts")
            || path.ends_with(".tsx")
            || path.ends_with(".js")
            || path.ends_with(".jsx")
            || path.ends_with(".py")
            || path.ends_with(".go")
    });
    let all_docs = !lower_paths.is_empty()
        && lower_paths.iter().all(|path| path.ends_with(".md") || path.contains("/docs/") || path.contains("readme"));
    let all_tests = !lower_paths.is_empty()
        && lower_paths.iter().all(|path| path.contains("test") || path.ends_with("_test.rs") || path.ends_with(".spec.ts"));
    let all_config = !lower_paths.is_empty()
        && lower_paths
            .iter()
            .all(|path| path.ends_with(".json") || path.ends_with(".toml") || path.ends_with(".nix") || path.contains("gitignore"));
    if has_code && summary.lines_added >= 100 && summary.files_added > 0 {
        return Some(String::from("feature"));
    }
    if all_docs {
        return Some(String::from("docs"));
    }
    if all_tests {
        return Some(String::from("test"));
    }
    if all_config {
        return Some(String::from("config"));
    }
    if summary.files_deleted > 0 && summary.files_added == 0 && summary.files_updated == 0 {
        return Some(String::from("refactor"));
    }
    if summary.files_moved > 0 || summary.lines_deleted > summary.lines_added.saturating_mul(2) {
        return Some(String::from("refactor"));
    }
    if summary.hunks > 0 && summary.lines_added + summary.lines_deleted <= 24 {
        return Some(String::from("fix"));
    }
    if summary.files_added > 0 && summary.files_updated == 0 {
        return Some(String::from("feature"));
    }
    (!paths.is_empty()).then_some(String::from("feature"))
}

pub(crate) fn build_token_efficiency(
    turn_count: usize,
    tool_calls: usize,
    input_tokens: u64,
    output_tokens: u64,
    reasoning_tokens: u64,
    cache_read_tokens: u64,
) -> TokenEfficiency {
    let total = input_tokens + output_tokens + reasoning_tokens + cache_read_tokens;
    TokenEfficiency {
        cache_hit_ratio: (total > 0).then_some(cache_read_tokens as f64 / total as f64),
        avg_input_tokens_per_turn: average_u64(input_tokens, turn_count),
        avg_output_tokens_per_turn: average_u64(output_tokens, turn_count),
        avg_reasoning_tokens_per_turn: average_u64(reasoning_tokens, turn_count),
        avg_tool_calls_per_turn: average_usize(tool_calls, turn_count),
        avg_input_tokens_per_tool_call: average_u64(input_tokens, tool_calls),
    }
}

pub(crate) fn build_message_turn_index(turns: &[TurnDigest]) -> HashMap<usize, usize> {
    let mut map = HashMap::new();
    for turn in turns {
        map.insert(turn.user_message_index, turn.turn_index);
        if let Some((start, end)) = turn.assistant_message_start.zip(turn.assistant_message_end) {
            for message_index in start..=end {
                map.insert(message_index, turn.turn_index);
            }
        }
    }
    map
}

pub(crate) fn tool_read_paths(tool: &str, input_value: Option<&Value>) -> Vec<String> {
    if tool != "read" {
        return Vec::new();
    }
    input_value
        .and_then(|value| value.get("filePath"))
        .and_then(Value::as_str)
        .map(|path| vec![normalize_tool_path(path)])
        .unwrap_or_default()
}

pub(crate) fn patch_paths_from_text(text: &str) -> Vec<String> {
    let mut items = Vec::new();
    let mut seen = HashSet::new();
    for line in text.lines() {
        let path = line
            .strip_prefix("*** Add File:")
            .or_else(|| line.strip_prefix("*** Update File:"))
            .or_else(|| line.strip_prefix("*** Delete File:"))
            .or_else(|| line.strip_prefix("*** Move to:"))
            .map(str::trim);
        let Some(path) = path else {
            continue;
        };
        let label = normalize_tool_path(path);
        if seen.insert(label.clone()) {
            items.push(label);
        }
    }
    items
}

pub(crate) fn patch_path_presence_from_text(text: &str) -> HashMap<String, bool> {
    let mut out = HashMap::new();
    for line in text.lines() {
        let added_or_updated = line
            .strip_prefix("*** Add File:")
            .or_else(|| line.strip_prefix("*** Update File:"))
            .or_else(|| line.strip_prefix("*** Move to:"))
            .map(str::trim);
        if let Some(path) = added_or_updated {
            out.insert(normalize_tool_path(path), true);
            continue;
        }
        let deleted = line.strip_prefix("*** Delete File:").map(str::trim);
        if let Some(path) = deleted {
            out.insert(normalize_tool_path(path), false);
        }
    }
    out
}
