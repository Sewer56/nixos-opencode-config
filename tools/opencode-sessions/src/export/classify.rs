use serde_json::Value;
use std::collections::{BTreeSet, HashSet};

use crate::constants::*;
use crate::format::*;
use crate::models::*;
use crate::export::turn::*;

pub(crate) fn build_session_narrative(
    title: &str,
    session_status: &str,
    staleness_ms: i64,
    turns: &[TurnDigest],
    deliverables: &[SessionDeliverableEntry],
    pivotal_turns: &[usize],
    child_links: &[ChildLink],
) -> Option<String> {
    if turns.is_empty() {
        return None;
    }
    let high_value = turns.iter().filter(|turn| turn.turn_effectiveness == "high-value").count();
    let waste = turns
        .iter()
        .filter(|turn| matches!(turn.turn_effectiveness.as_str(), "waste" | "low-value"))
        .count();
    let top_files = deliverables
        .iter()
        .take(3)
        .map(|item| item.path.clone())
        .collect::<Vec<_>>();

    let mut parts = vec![format!("Session `{title}` status={session_status} across {} turns.", turns.len())];
    parts.push(format!(
        "Snapshot: {}.",
        infer_snapshot_completeness(session_status, staleness_ms)
    ));
    if !pivotal_turns.is_empty() {
        parts.push(format!("Pivotal turns: {}.", pivotal_turns.iter().map(|idx| idx.to_string()).collect::<Vec<_>>().join(", ")));
    }
    parts.push(format!("High-value turns: {high_value}; low-value/waste turns: {waste}."));
    if !top_files.is_empty() {
        parts.push(format!("Main deliverables: {}.", top_files.join(", ")));
    }
    if !child_links.is_empty() {
        parts.push(format!("Child sessions: {}.", child_links.len()));
    }
    Some(truncate_text(&parts.join(" "), REASONING_SUMMARY_LIMIT))
}

pub(crate) fn classify_user_intent(text: Option<&str>) -> (String, f64) {
    let text = text.unwrap_or("").to_lowercase();
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return (String::from("continuation"), 0.3);
    }
    if text.contains("remember your task")
        || text.contains("not human")
        || text.contains("keep it simple")
    {
        return (String::from("redirect"), 0.97);
    }
    if text.contains("please note")
        || text.contains("don't ")
        || text.contains("do not ")
        || text.contains("must ")
        || text.contains("should not ")
    {
        return (String::from("redirect"), 0.84);
    }
    if trimmed == "continue"
        || trimmed == "resume"
        || trimmed == "keep going"
        || trimmed == "go on"
        || text.contains("continue")
        || text.contains("keep going")
        || text.contains("resume")
    {
        return (String::from("continuation"), 0.9);
    }
    if text.contains("can you")
        || text.contains("could you")
        || text.contains("would you")
        || trimmed.starts_with("please ")
    {
        return (String::from("followup-request"), 0.88);
    }
    if ["what ", "why ", "how ", "when ", "where ", "did ", "does ", "is ", "are "]
        .iter()
        .any(|prefix| trimmed.starts_with(prefix))
        || trimmed.ends_with('?')
    {
        return (String::from("followup-request"), 0.82);
    }
    if text.contains("okay. i want some changes")
        || text.contains("i want")
        || text.contains("while you're at it")
        || text.contains("also ")
        || text.contains("instead")
    {
        return (String::from("scope-change"), 0.84);
    }
    if text.contains("fixed")
        || text.contains("done")
        || text.contains("thanks")
        || text.contains("looks good")
        || text.contains("lgtm")
    {
        return (String::from("approval"), 0.76);
    }
    (String::from("task"), 0.82)
}

pub(crate) fn classify_alternative_user_intents(text: Option<&str>, primary: &str, confidence: f64) -> Vec<String> {
    let text = text.unwrap_or("").to_lowercase();
    let mut out = Vec::new();
    let mut push = |label: &str| {
        if label != primary && !out.iter().any(|item| item == label) {
            out.push(label.to_string());
        }
    };

    if confidence <= 0.5 {
        match primary {
            "continuation" => {
                push("followup-request");
                push("task");
            }
            "task" => {
                push("followup-request");
                push("continuation");
            }
            "followup-request" => {
                push("task");
                push("continuation");
            }
            _ => {}
        }
    }

    if text.contains("continue") || text.contains("keep going") || text.contains("again") {
        push("continuation");
    }
    if text.contains('?') || text.starts_with("what ") || text.starts_with("how ") || text.starts_with("why ") {
        push("followup-request");
    }
    if text.contains("please") || text.contains("do ") || text.contains("add ") || text.contains("fix ") {
        push("task");
    }
    out.truncate(2);
    out
}

pub(crate) fn classify_user_tags(text: Option<&str>) -> Vec<String> {
    let text = text.unwrap_or("").to_lowercase();
    let mut tags = Vec::new();
    if text.contains("subagent") {
        tags.push(String::from("subagents"));
    }
    if text.contains("tui") {
        tags.push(String::from("tui"));
    }
    if text.contains("cli") {
        tags.push(String::from("cli"));
    }
    if text.contains("machine") || text.contains("llm") {
        tags.push(String::from("machine-optimization"));
    }
    if text.contains("stats") || text.contains("how long") {
        tags.push(String::from("metrics"));
    }
    tags
}

pub(crate) fn infer_turn_outcome(
    tool_call_count: usize,
    delegations: &[TurnDelegationPreview],
    next_user_intent: Option<&str>,
    next_user_tags: &[String],
) -> String {
    if matches!(next_user_intent, Some("redirect") | Some("scope-change")) {
        return String::from("redirected");
    }
    if !delegations.is_empty() {
        return String::from("delegated");
    }
    if tool_call_count > 0 {
        return String::from("executed");
    }
    if next_user_tags.iter().any(|tag| tag == "metrics" || tag == "tui") {
        return String::from("followup-needed");
    }
    String::from("answered")
}

pub(crate) fn is_analysis_tool(tool: &str) -> bool {
    matches!(tool, "read" | "grep" | "glob" | "bash" | "task")
}

pub(crate) fn is_analysis_only_turn_shape(
    modified_file_count: usize,
    tool_rollup: &[TurnToolAggregate],
    final_assistant_text_present: bool,
) -> bool {
    modified_file_count == 0
        && final_assistant_text_present
        && !tool_rollup.is_empty()
        && tool_rollup.iter().all(|entry| is_analysis_tool(&entry.tool))
}

pub(crate) fn infer_turn_success(
    error_count: usize,
    outcome: &str,
    final_assistant_kind: Option<&str>,
    final_assistant_text_present: bool,
    modified_file_count: usize,
    tool_rollup: &[TurnToolAggregate],
) -> bool {
    if outcome == "redirected" {
        return false;
    }
    if is_analysis_only_turn_shape(modified_file_count, tool_rollup, final_assistant_text_present)
        && matches!(outcome, "answered" | "executed")
    {
        return true;
    }
    if error_count > 0 && final_assistant_kind != Some("deliverable") {
        return false;
    }
    matches!(outcome, "answered" | "executed" | "delegated")
}

pub(crate) fn infer_agent_strategy(
    tool_rollup: &[TurnToolAggregate],
    error_count: usize,
    delegations: &[TurnDelegationPreview],
    modified_file_count: usize,
) -> String {
    if !delegations.is_empty() {
        return String::from("delegate");
    }
    if error_count > 0 {
        return String::from("debug");
    }
    let read_calls = tool_rollup
        .iter()
        .filter(|entry| matches!(entry.tool.as_str(), "read" | "grep" | "glob"))
        .map(|entry| entry.calls)
        .sum::<usize>();
    let patch_calls = tool_rollup
        .iter()
        .filter(|entry| entry.tool == "apply_patch")
        .map(|entry| entry.calls)
        .sum::<usize>();
    let build_calls = tool_rollup
        .iter()
        .filter(|entry| matches!(entry.tool.as_str(), "bash" | "task"))
        .map(|entry| entry.calls)
        .sum::<usize>();
    if modified_file_count > 0 && patch_calls >= read_calls {
        return String::from("implement");
    }
    if patch_calls > 0 && build_calls > 0 {
        return String::from("validate");
    }
    if patch_calls > 0 {
        return String::from("refactor");
    }
    if build_calls > 0 {
        return String::from("validate");
    }
    String::from("explore")
}

pub(crate) fn classify_turn_cost_tier(total_tokens: u64, tool_call_count: usize, response_elapsed_ms: Option<i64>) -> String {
    if total_tokens >= 500_000 || tool_call_count >= 50 || response_elapsed_ms.unwrap_or_default() >= 600_000 {
        return String::from("extreme");
    }
    if total_tokens >= 150_000 || tool_call_count >= 20 || response_elapsed_ms.unwrap_or_default() >= 180_000 {
        return String::from("heavy");
    }
    if total_tokens >= 40_000 || tool_call_count >= 5 || response_elapsed_ms.unwrap_or_default() >= 60_000 {
        return String::from("medium");
    }
    String::from("light")
}

pub(crate) fn classify_turn_effectiveness(turn: &TurnDigest, files_survived_to_end: usize) -> String {
    let retry_ratio = turn.effectiveness_signals.retry_ratio.unwrap_or_default();
    let redundant_read_ratio = turn.effectiveness_signals.redundant_read_ratio.unwrap_or_default();
    if !turn.success && files_survived_to_end == 0 && turn.modified_file_count == 0 {
        return String::from("waste");
    }
    if files_survived_to_end > 0 || (turn.outcome == "delegated" && turn.success) {
        return String::from("high-value");
    }
    if turn.success && turn.modified_file_count > 0
        && (matches!(turn.turn_cost_tier.as_str(), "heavy" | "extreme") || retry_ratio >= 0.5 || redundant_read_ratio >= 0.7) {
            return String::from("moderate");
        }
    if retry_ratio >= 0.5 || redundant_read_ratio >= 0.7 {
        return String::from("low-value");
    }
    if turn.success && (turn.tool_call_count > 0 || turn.modified_file_count > 0) {
        return String::from("moderate");
    }
    if !turn.success {
        return String::from("low-value");
    }
    String::from("moderate")
}

pub(crate) fn recommend_turn_attention(turn: &TurnDigest) -> String {
    match (turn.turn_effectiveness.as_str(), turn.turn_cost_tier.as_str()) {
        ("waste", _) => String::from("skip"),
        ("low-value", "heavy" | "extreme") => String::from("skim"),
        ("high-value", "heavy" | "extreme") => String::from("inspect-artifacts"),
        ("high-value", _) => String::from("read-carefully"),
        _ => String::from("skim"),
    }
}

pub(crate) fn build_failure_narrative(turn: &TurnDigest, files_survived_to_end: usize) -> Option<String> {
    if !matches!(turn.turn_effectiveness.as_str(), "waste" | "low-value") {
        return None;
    }
    if turn.tool_call_count == 0 {
        return Some(String::from("No tool work or durable file change landed in this turn."));
    }
    if turn.effectiveness_signals.redundant_read_ratio.unwrap_or_default() >= 0.6 {
        return Some(format!(
            "Heavy repeated reads with limited durable change; redundant_read_ratio={:.2} across {} tool calls.",
            turn.effectiveness_signals.redundant_read_ratio.unwrap_or_default(),
            turn.tool_call_count
        ));
    }
    if turn.error_count > 0 && files_survived_to_end == 0 {
        return Some(format!(
            "Errors interrupted turn and no durable file changes survived; {} tool errors across {} calls.",
            turn.error_count,
            turn.tool_call_count
        ));
    }
    if turn.modified_file_count > 0 && files_survived_to_end == 0 {
        return Some(format!(
            "Turn changed {} files but none of those changes survived to session end.",
            turn.modified_file_count
        ));
    }
    Some(String::from("Turn consumed budget without strong durable output signal."))
}

pub(crate) fn build_optimization_hints(turn: &TurnDigest) -> Vec<String> {
    let mut hints = Vec::new();
    if matches!(turn.turn_cost_tier.as_str(), "heavy" | "extreme") && turn.turn_effectiveness != "high-value" {
        hints.push(format!(
            "High-cost turn ({} tokens, {} tool calls); cut exploration loops sooner.",
            turn.total_tokens, turn.tool_call_count
        ));
    }
    let redundant_read_ratio = turn.effectiveness_signals.redundant_read_ratio.unwrap_or_default();
    if redundant_read_ratio >= 0.5 {
        hints.push(format!(
            "Repeated reads dominate this turn ({:.0}% redundant); cache file state after first read.",
            redundant_read_ratio * 100.0
        ));
    }
    if turn.error_count >= 3 {
        hints.push(format!(
            "Repeated tool failures ({} errors); pre-validate patch/tool context before retrying.",
            turn.error_count
        ));
    }
    if turn.modified_file_count > 0 && turn.effectiveness_signals.files_survived_to_end == 0 {
        hints.push(format!(
            "Changes were overwritten later ({} files, 0 survived to end); checkpoint or consolidate edits sooner.",
            turn.modified_file_count
        ));
    }
    if turn.delegation_count > 0 && turn.turn_effectiveness == "low-value" {
        hints.push(format!(
            "Delegation looked low-yield ({} subagent calls); narrow task scope before delegating.",
            turn.delegation_count
        ));
    }
    hints
}

pub(crate) fn summarize_turn_change(
    modified_file_count: usize,
    modified_files: &BTreeSet<String>,
    final_assistant_text_preview: Option<&str>,
    outcome: &str,
) -> Option<String> {
    if let Some(text) = final_assistant_text_preview {
        return Some(truncate_text(text, TURN_PREVIEW_LIMIT));
    }
    if modified_file_count > 0 {
        let first = modified_files.iter().next().cloned().unwrap_or_else(|| String::from("files"));
        return Some(if modified_file_count == 1 {
            format!("updated {first}")
        } else {
            format!("updated {modified_file_count} files incl {first}")
        });
    }
    if outcome == "delegated" {
        return Some(String::from("delegated subtask"));
    }
    None
}

pub(crate) fn classify_assistant_text_kind(text: Option<&str>) -> Option<String> {
    let text = text?.trim();
    if text.is_empty() {
        return None;
    }
    let lower = text.to_lowercase();
    if lower.starts_with("done")
        || lower.starts_with("fixed")
        || lower.starts_with("changed:")
        || lower.starts_with("what changed:")
        || lower.starts_with("next steps")
        || lower.starts_with("---")
    {
        return Some(String::from("deliverable"));
    }
    if lower.starts_with("need ")
        || lower.starts_with("inspect ")
        || lower.starts_with("baseline ")
        || lower.starts_with("iteration ")
        || lower.starts_with("freeze ")
        || lower.starts_with("read ")
    {
        return Some(String::from("scratchpad"));
    }
    Some(String::from("mixed"))
}

pub(crate) fn classify_tool_error(tool: &str, status: &str, error_text: &str) -> Option<String> {
    if status != "error" {
        return None;
    }
    let lower = error_text.to_lowercase();
    if lower.contains("aborted") {
        return Some(String::from("aborted"));
    }
    if lower.contains("not found") {
        return Some(String::from("not-found"));
    }
    if lower.contains("timed out") || lower.contains("timeout") {
        return Some(String::from("timeout"));
    }
    if lower.contains("permission") {
        return Some(String::from("permission"));
    }
    if tool == "apply_patch" {
        return Some(String::from("patch-error"));
    }
    Some(String::from("tool-error"))
}

pub(crate) fn extract_reasoning_themes(text: &str) -> Vec<String> {
    let mut themes = Vec::new();
    let mut seen = HashSet::new();
    for line in text.lines().map(str::trim) {
        let normalized = line
            .trim_matches('*')
            .trim_start_matches('#')
            .trim_start_matches('-')
            .trim();
        if normalized.is_empty() {
            continue;
        }
        let is_theme = line.starts_with("**")
            || line.starts_with('#')
            || normalized.split_whitespace().count() <= 6;
        if !is_theme {
            continue;
        }
        let compact = normalized
            .trim_end_matches(':')
            .split_terminator(['.', '!', '?'])
            .next()
            .unwrap_or(normalized)
            .trim();
        if compact.is_empty() {
            continue;
        }
        let label = truncate_text(compact, 48);
        if seen.insert(label.clone()) {
            themes.push(label);
        }
        if themes.len() >= 5 {
            break;
        }
    }
    themes
}

pub(crate) fn summarize_reasoning(text: &str, themes: &[String]) -> Option<String> {
    let lines = text
        .lines()
        .map(str::trim)
        .map(|line| {
            line.trim_matches('*')
                .trim_start_matches('#')
                .trim_start_matches('-')
                .trim()
                .to_string()
        })
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>();
    if lines.is_empty() {
        return None;
    }

    let mut parts = Vec::new();
    if !themes.is_empty() {
        parts.push(format!("themes: {}", themes.iter().take(3).cloned().collect::<Vec<_>>().join(", ")));
    }

    if let Some(first) = lines.first() {
        let first_theme = themes.first().map(|theme| theme.to_lowercase()).unwrap_or_default();
        if first.to_lowercase() != first_theme {
            parts.push(format!("start: {}", truncate_text(first, 120)));
        }
    }
    if lines.len() > 1
        && let Some(last) = lines.iter().rev().find(|line| line.as_str() != lines[0]) {
            parts.push(format!("end: {}", truncate_text(last, 120)));
        }

    Some(truncate_text(&parts.join(" | "), REASONING_SUMMARY_LIMIT))
}

pub(crate) fn classify_message_kind(role: &str, has_text: bool, has_reasoning: bool, has_activity: bool) -> String {
    if role == "user" {
        return String::from("user");
    }
    if has_text && (has_reasoning || has_activity) {
        return String::from("assistant-mixed");
    }
    if has_text {
        return String::from("assistant-text");
    }
    if has_activity {
        return String::from("assistant-tool-only");
    }
    if has_reasoning {
        return String::from("assistant-reasoning-only");
    }
    String::from("assistant-tool-only")
}

pub(crate) fn summarize_activity_items(items: &[String]) -> Option<String> {
    if items.is_empty() {
        return None;
    }
    let mut unique = Vec::new();
    let mut seen = HashSet::new();
    for item in items {
        if seen.insert(item.clone()) {
            unique.push(item.clone());
        }
    }
    let extra = unique.len().saturating_sub(ACTIVITY_SUMMARY_ITEMS_LIMIT);
    let mut summary = unique
        .into_iter()
        .take(ACTIVITY_SUMMARY_ITEMS_LIMIT)
        .collect::<Vec<_>>()
        .join("; ");
    if extra > 0 {
        summary.push_str(&format!("; +{extra} more"));
    }
    Some(truncate_text(&summary, ACTIVITY_SUMMARY_LIMIT))
}

pub(crate) fn summarize_tool_activity(
    tool: &str,
    status: &str,
    task_description: Option<&str>,
    input_value: Option<&Value>,
    patch_summary: Option<&PatchSummary>,
    modified_paths: &[String],
) -> Option<String> {
    if tool == "apply_patch" {
        let mut label = patch_summary
            .map(|summary| format!("apply_patch {}", render_patch_summary(summary)))
            .unwrap_or_else(|| format!("apply_patch {status}"));
        if let Some(path) = modified_paths.first() {
            label.push_str(&format!(" {}", path));
        }
        return Some(truncate_text(&label, TOOL_PART_PREVIEW_LIMIT));
    }
    if tool == "read" {
        return input_value
            .and_then(|value| value.get("filePath"))
            .and_then(Value::as_str)
            .map(|path| format!("read {}", normalize_tool_path(path)));
    }
    if tool == "bash" {
        return task_description
            .map(|desc| format!("bash {desc}"))
            .or_else(|| Some(String::from("bash")));
    }
    if tool == "task" {
        return task_description
            .map(|desc| format!("task {desc}"))
            .or_else(|| Some(String::from("task")));
    }
    if tool == "grep" {
        return input_value
            .and_then(|value| value.get("pattern"))
            .and_then(Value::as_str)
            .map(|pattern| format!("grep {}", truncate_text(pattern, TOOL_PART_PREVIEW_LIMIT)));
    }
    if tool == "glob" {
        return input_value
            .and_then(|value| value.get("pattern"))
            .and_then(Value::as_str)
            .map(|pattern| format!("glob {}", truncate_text(pattern, TOOL_PART_PREVIEW_LIMIT)));
    }
    if status == "error" {
        return Some(format!("{tool} error"));
    }
    Some(tool.to_string())
}

pub(crate) fn classify_tool_call_purpose(
    tool: &str,
    task_description: Option<&str>,
    read_paths: &[String],
    modified_paths: &[String],
) -> Option<String> {
    if tool == "task" {
        return Some(String::from("delegate"));
    }
    if tool == "apply_patch" {
        return Some(String::from("modify"));
    }
    if tool == "read" {
        return Some(String::from("context-gather"));
    }
    if matches!(tool, "grep" | "glob") {
        return Some(String::from("search"));
    }
    if tool == "bash" {
        let desc = task_description.unwrap_or("").to_lowercase();
        if desc.contains("test") || desc.contains("smoke") {
            return Some(String::from("run-test"));
        }
        if desc.contains("compile") || desc.contains("build") || desc.contains("check") {
            return Some(String::from("build"));
        }
        if !modified_paths.is_empty() || !read_paths.is_empty() {
            return Some(String::from("verify-change"));
        }
        return Some(String::from("run-command"));
    }
    None
}
