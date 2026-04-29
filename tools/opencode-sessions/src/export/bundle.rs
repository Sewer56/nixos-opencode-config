use anyhow::{Context, Result};
use chrono::Utc;
use rusqlite::{Connection, params};
use serde_json::Value;
use std::cmp::Reverse;
use std::collections::HashMap;
use std::fs::{self};
use std::path::{Path, PathBuf};
use uuid::Uuid;

use crate::constants::*;
use crate::format::*;
use crate::models::*;
use crate::export::session_output::*;
use crate::export::turn::*;
use crate::export::classify::*;
use crate::export::hotspot::*;
use crate::export::rollup::*;
use crate::export::io::*;
use crate::export::delta::*;
use crate::export::schema::*;

pub(crate) fn export_bundle(
    conn: &Connection,
    index: &OverviewIndex,
    root_session_id: &str,
    out_dir: Option<PathBuf>,
) -> Result<PathBuf> {
    let loaded = load_session_tree(conn, index, root_session_id)?;
    let base_dir = out_dir.unwrap_or_else(default_export_base_dir);
    let export_id = Uuid::now_v7().to_string();
    let export_timestamp_ms = Utc::now().timestamp_millis();
    fs::create_dir_all(&base_dir).with_context(|| format!("create {}", base_dir.display()))?;

    let root_name = format!(
        "{}__{}__{}",
        format_timestamp_slug(loaded.meta.time_updated),
        sanitize_filename(&loaded.meta.title),
        short_id(&loaded.meta.id),
    );
    let export_root = unique_child_dir(&base_dir, &root_name)?;
    fs::create_dir_all(export_root.join("sessions"))?;

    let mut acc = ExportAccumulator::new();
    let tree = write_session_bundle(
        &loaded,
        &export_root,
        Path::new("sessions"),
        "0",
        0,
        true,
        export_timestamp_ms,
        &mut acc,
    )?;

    let mut tool_rollup = rollup_tools(&acc.tool_calls);
    tool_rollup.sort_by(|left, right| {
        right
            .calls
            .cmp(&left.calls)
            .then_with(|| right.total_duration_ms.cmp(&left.total_duration_ms))
            .then_with(|| left.tool.cmp(&right.tool))
    });

    acc.session_index.sort_by(|left, right| left.depth.cmp(&right.depth).then_with(|| left.session_path.cmp(&right.session_path)));
    acc.session_hotspots.sort_by_key(|entry| Reverse(entry.duration_ms));

    let hotspots = build_hotspots(&acc.session_hotspots, &acc.turns, &acc.message_digests, &acc.tool_calls);
    let root_entry = acc
        .session_index
        .iter()
        .find(|entry| entry.depth == 0)
        .or_else(|| acc.session_index.first());
    let iteration_meta = build_iteration_meta(&base_dir, &root_name, &export_root)?;
    let totals = acc.totals();
    let token_efficiency = build_token_efficiency(
        totals.turn_count,
        totals.tool_calls,
        totals.input_tokens,
        totals.output_tokens,
        totals.reasoning_tokens,
        totals.cache_read_tokens,
    );
    let mut index_file = ExportIndexFile {
        format: "opencode-sessions-v1",
        schema_version: SCHEMA_VERSION,
        schema_file: String::from("schema.json"),
        fields_file: String::from("fields.json"),
        export_id,
        export_timestamp_ms,
        iteration_meta,
        delta_from_previous: None,
        root_session_id: loaded.meta.id.clone(),
        root_title: loaded.meta.title.clone(),
        root_session_status: root_entry
            .map(|root| root.session_status.clone())
            .unwrap_or_else(|| String::from("abandoned")),
        root_snapshot_completeness: root_entry
            .map(|root| root.snapshot_completeness.clone())
            .unwrap_or_else(|| String::from("partial")),
        root_last_activity_ms: loaded.meta.time_updated,
        root_staleness_ms: export_timestamp_ms.saturating_sub(loaded.meta.time_updated),
        root_task_preview: acc.root_task_preview.clone(),
        root_task_file: acc.root_task_file.clone(),
        schema_changes: vec![
            "export identity plus root/session status for cross-bundle comparison",
            "turn intent confidence plus fallback intents and cost/effectiveness/attention labels with documented classification policy",
            "formal schema file with typed enums, stricter top-level validation, compact turn plus message scan layers, deliverable snapshots, sharper optimization hints, snapshot completeness labels, direct tool-call turn_index joins, message-span pointers, aggregated per-file transition rollups, dependency edges, and resolved stale-child export paths",
        ],
        artifact_policy: ArtifactPolicy {
            assistant_text_file_chars: ASSISTANT_TEXT_ARTIFACT_CHARS_THRESHOLD,
            reasoning_file_chars: REASONING_ARTIFACT_CHARS_THRESHOLD,
            tool_input_inline_chars: TOOL_INPUT_INLINE_CHARS_THRESHOLD,
            tool_output_inline_chars: TOOL_CALLS_EMBEDDED_IO_LIMIT,
        },
        classification_policy: ClassificationPolicy {
            version: "heuristic-v6",
            user_intent_values: vec![
                "task",
                "continuation",
                "redirect",
                "followup-request",
                "scope-change",
                "approval",
            ],
            user_tag_values: vec!["subagents", "tui", "cli", "machine-optimization", "metrics"],
            message_kind_values: vec!["user", "assistant-text", "assistant-tool-only", "assistant-mixed", "assistant-reasoning-only"],
            outcome_values: vec!["answered", "executed", "delegated", "redirected", "followup-needed"],
            assistant_kind_values: vec!["deliverable", "scratchpad", "mixed"],
            session_status_values: vec!["completed", "running", "abandoned", "error"],
            agent_strategy_values: vec!["explore", "implement", "debug", "refactor", "validate", "delegate"],
            turn_cost_tier_values: vec!["light", "medium", "heavy", "extreme"],
            turn_effectiveness_values: vec!["high-value", "moderate", "low-value", "waste"],
            recommended_attention_values: vec!["skip", "skim", "read-carefully", "inspect-artifacts"],
            child_export_reference_status_values: vec!["current-export", "mixed-export", "stale-export"],
            patch_intent_values: vec!["feature", "fix", "refactor", "config", "test", "docs"],
            tool_call_purpose_values: vec![
                "context-gather",
                "search",
                "verify-change",
                "run-test",
                "build",
                "run-command",
                "modify",
                "delegate",
            ],
            retry_recovery_values: vec!["retry", "re-read-and-retry", "verify-or-build", "change-approach", "abandon"],
            intent_confidence_range: "0..1 heuristic confidence",
            confidence_thresholds: ConfidenceThresholds {
                reliable_above: 0.75,
                uncertain_below: 0.5,
            },
        },
        recommended_read_order: root_entry
            .map(|root| {
                vec![
                    String::from("index.json"),
                    String::from("schema.json"),
                    String::from("fields.json"),
                    root.summary_file.clone(),
                    root.turns_compact_file.clone().unwrap_or_else(|| root.turns_file.clone()),
                    root.turns_file.clone(),
                    root.messages_compact_file.clone().unwrap_or_else(|| root.messages_file.clone()),
                    root.messages_file.clone(),
                    root.tool_calls_file.clone(),
                ]
            })
            .unwrap_or_else(|| vec![String::from("index.json"), String::from("schema.json"), String::from("fields.json")]),
        totals,
        token_efficiency,
        tree,
        session_index: acc.session_index.clone(),
        tool_rollup,
        hotspots,
    };
    index_file.delta_from_previous = build_delta_from_previous(&base_dir, &export_root, &index_file)?;

    write_json_pretty(export_root.join("index.json"), &index_file)?;
    write_json_pretty(export_root.join("schema.json"), &build_export_schema())?;
    write_json_pretty(export_root.join("fields.json"), &build_export_fields_catalog())?;
    write_text(
        export_root.join("README.md"),
        &render_export_readme(&index_file),
    )?;

    Ok(export_root)
}

pub(crate) fn load_session_tree(conn: &Connection, index: &OverviewIndex, session_id: &str) -> Result<LoadedSession> {
    let meta = index.get(session_id)?.clone();
    let messages = load_messages(conn, session_id)?;
    let children = index
        .children_of(session_id)
        .iter()
        .map(|child_id| load_session_tree(conn, index, child_id))
        .collect::<Result<Vec<_>>>()?;

    Ok(LoadedSession { meta, messages, children })
}

pub(crate) fn load_messages(conn: &Connection, session_id: &str) -> Result<Vec<LoadedMessage>> {
    let mut messages = Vec::new();
    let mut stmt = conn.prepare(
        r#"
        select id, session_id, time_created, time_updated, data
        from message
        where session_id = ?1
        order by time_created asc, id asc
        "#,
    )?;
    let mut rows = stmt.query(params![session_id])?;
    while let Some(row) = rows.next()? {
        let id: String = row.get(0)?;
        let time_created: i64 = row.get(2)?;
        let raw_json: String = row.get(4)?;
        let info: MessageInfo = serde_json::from_str(&raw_json)
            .with_context(|| format!("parse message json for {id}"))?;
        messages.push(LoadedMessage {
            id,
            time_created,
            info,
            parts: Vec::new(),
        });
    }

    let mut parts_by_message: HashMap<String, Vec<LoadedPart>> = HashMap::new();
    let mut stmt = conn.prepare(
        r#"
        select id, message_id, session_id, time_created, time_updated, data
        from part
        where session_id = ?1
        order by time_created asc, id asc
        "#,
    )?;
    let mut rows = stmt.query(params![session_id])?;
    while let Some(row) = rows.next()? {
        let message_id: String = row.get(1)?;
        let raw_json: String = row.get(5)?;
        let raw: Value = serde_json::from_str(&raw_json)
            .with_context(|| format!("parse part json for message {message_id}"))?;
        parts_by_message
            .entry(message_id.clone())
            .or_default()
            .push(LoadedPart { raw });
    }

    for message in &mut messages {
        message.parts = parts_by_message.remove(&message.id).unwrap_or_default();
    }

    Ok(messages)
}
#[allow(clippy::too_many_arguments)]
pub(crate) fn write_session_bundle(
    session: &LoadedSession,
    bundle_root: &Path,
    relative_parent_dir: &Path,
    session_path: &str,
    depth: usize,
    is_root: bool,
    export_timestamp_ms: i64,
    acc: &mut ExportAccumulator,
) -> Result<ExportTreeNode> {
    let agent = session.agent();
    let folder_name = session_folder_name(is_root, session_path, agent.as_deref(), &session.meta.title, &session.meta.id);
    let relative_session_dir = relative_parent_dir.join(&folder_name);
    let session_dir = bundle_root.join(&relative_session_dir);
    fs::create_dir_all(&session_dir).with_context(|| format!("create {}", session_dir.display()))?;

    let compact_messages = session
        .messages
        .iter()
        .enumerate()
        .map(|(message_index, message)| compact_message(message, session_path, depth, message_index))
        .collect::<Result<Vec<_>>>()?;

    let mut child_links = Vec::new();
    let mut child_tree_nodes = Vec::new();
    let children_relative_dir = relative_session_dir.join("children");
    let children_dir = bundle_root.join(&children_relative_dir);
    if !session.children.is_empty() {
        fs::create_dir_all(&children_dir)?;
    }

    let current_export_name = bundle_root
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or_default();
    let child_delegations = map_child_delegations(session, &compact_messages, current_export_name);

    for (child_index, child) in session.children.iter().enumerate() {
        let child_path = format!("{session_path}.{child_index}");
        let child_tree = write_session_bundle(
            child,
            bundle_root,
            &children_relative_dir,
            &child_path,
            depth + 1,
            false,
            export_timestamp_ms,
            acc,
        )?;
        let delegation = child_delegations.get(&child.meta.id);
        let child_stats = acc.session_stats.iter().find(|stats| stats.session_id == child.meta.id);
        let child_turn = acc
            .turns
            .iter()
            .filter(|turn| turn.session_path == child_path)
            .max_by_key(|turn| turn.turn_index);
        child_links.push(ChildLink {
            session_path: child_path,
            session_id: child.meta.id.clone(),
            title: child.meta.title.clone(),
            agent: child.agent(),
            summary_file: child_tree.summary_file.clone(),
            duration_ms: child.meta.duration_ms(),
            turn_count: child_stats.map(|stats| stats.turn_count).unwrap_or_default(),
            message_count: child_stats.map(|stats| stats.message_count).unwrap_or(child.meta.message_count),
            tool_call_count: child_stats.map(|stats| stats.tool_calls).unwrap_or_default(),
            input_tokens: child_stats.map(|stats| stats.input_tokens).unwrap_or_default(),
            output_tokens: child_stats.map(|stats| stats.output_tokens).unwrap_or_default(),
            reasoning_tokens: child_stats.map(|stats| stats.reasoning_tokens).unwrap_or_default(),
            parent_message_index: delegation.map(|item| item.message_index),
            parent_tool_index: delegation.map(|item| item.tool_index),
            delegation_description: delegation.and_then(|item| item.description.clone()),
            delegation_prompt_preview: delegation.and_then(|item| item.prompt_preview.clone()),
            delegation_prompt_preview_resolved: delegation.and_then(|item| item.prompt_preview_resolved.clone()),
            delegation_prompt_export_paths: delegation
                .map(|item| item.prompt_export_paths.clone())
                .unwrap_or_default(),
            resolved_current_export_paths: delegation
                .map(|item| resolve_current_export_paths(&item.prompt_export_paths, current_export_name))
                .unwrap_or_default(),
            delegation_export_reference_status: delegation.and_then(|item| item.export_reference_status.clone()),
            current_export_path_hint: delegation
                .and_then(|item| item.export_reference_status.as_deref())
                .filter(|status| matches!(*status, "stale-export" | "mixed-export"))
                .map(|_| format!("exports/{current_export_name}")),
            delegation_input_file: delegation.and_then(|item| item.input_file.clone()),
            delegation_result_preview: child_turn.and_then(|turn| turn.final_assistant_text_preview.clone()),
            delegation_result_file: child_turn.and_then(|turn| turn.final_assistant_text_file.clone()),
            child_outcome: child_turn.map(|turn| turn.outcome.clone()),
            child_final_assistant_kind: child_turn.and_then(|turn| turn.final_assistant_kind.clone()),
        });
        child_tree_nodes.push(child_tree);
    }

    let session_meta = SessionFileMeta {
        session_path: session_path.to_string(),
        session_id: session.meta.id.clone(),
        title: session.meta.title.clone(),
        agent: agent.clone(),
        created_ms: session.meta.time_created,
        updated_ms: session.meta.time_updated,
        duration_ms: session.meta.duration_ms(),
    };

    let session_stats = compute_session_stats(session, &compact_messages, session_path, depth, agent.clone());

    let turns_file = path_string(&relative_session_dir.join("turns.jsonl"));
    let turns_compact_file = path_string(&relative_session_dir.join("turns.compact.jsonl"));
    let messages_compact_file = path_string(&relative_session_dir.join("messages.compact.jsonl"));
    let messages_file = path_string(&relative_session_dir.join("messages.jsonl"));
    let tool_calls_file = path_string(&relative_session_dir.join("tool_calls.jsonl"));
    let summary_file = path_string(&relative_session_dir.join("summary.json"));
    let out = build_session_machine_output(
        session,
        &compact_messages,
        &child_links,
        &session_dir,
        &relative_session_dir,
        session_path,
    )?;
    let (turn_digests, message_digests, tool_digests, runtime, prompt_preview, prompt_file, artifacts_dir, artifact_count) =
        (out.turn_digests, out.message_digests, out.tool_digests, out.runtime, out.prompt_preview, out.prompt_file, out.artifacts_dir, out.artifact_count);
    let (artifacts_manifest_file, mut artifact_entries) = write_artifacts_manifest(&session_dir, &relative_session_dir)?;
    artifact_entries.sort_by(|left, right| right.size_bytes.cmp(&left.size_bytes).then_with(|| left.path.cmp(&right.path)));
    let largest_artifacts = artifact_entries.into_iter().take(10).collect::<Vec<_>>();
    let session_status = infer_session_status(session, &compact_messages, &tool_digests);

    let mut tool_rollup = rollup_tools(&tool_digests);
    tool_rollup.sort_by(|left, right| {
        right
            .calls
            .cmp(&left.calls)
            .then_with(|| right.total_duration_ms.cmp(&left.total_duration_ms))
            .then_with(|| left.tool.cmp(&right.tool))
    });

    let hot_turns = build_session_hot_turns(&turn_digests);
    let pivotal_turns = build_pivotal_turns(&turn_digests);
    let hot_messages = build_session_hot_messages(&message_digests);
    let file_access_rollup = build_file_access_rollup(&turn_digests, &tool_digests);
    let error_patterns = build_error_patterns(&turn_digests, &tool_digests);
    let retry_chains = build_retry_chains(&turn_digests, &tool_digests);
    let file_transition_rollup = build_file_transition_rollup(&turn_digests, &tool_digests, &session_status);
    let session_deliverables = build_session_deliverables(&turn_digests, &tool_digests, &session_dir, &relative_session_dir)?;
    let turn_dependency_edges = build_turn_dependency_edges(&file_transition_rollup);
    let session_narrative = build_session_narrative(
        &session.meta.title,
        &session_status,
        export_timestamp_ms.saturating_sub(session.meta.time_updated),
        &turn_digests,
        &session_deliverables,
        &pivotal_turns,
        &child_links,
    );
    let snapshot_completeness = infer_snapshot_completeness(
        &session_status,
        export_timestamp_ms.saturating_sub(session.meta.time_updated),
    );
    let session_totals_value = session_totals(&session_stats);
    let token_efficiency = build_token_efficiency(
        session_totals_value.turn_count,
        session_totals_value.tool_calls,
        session_totals_value.input_tokens,
        session_totals_value.output_tokens,
        session_totals_value.reasoning_tokens,
        session_totals_value.cache_read_tokens,
    );

    let summary = SessionSummaryFile {
        session: session_meta.clone(),
        runtime: runtime.clone(),
        session_status: session_status.clone(),
        snapshot_completeness: snapshot_completeness.clone(),
        last_activity_ms: session.meta.time_updated,
        staleness_ms: export_timestamp_ms.saturating_sub(session.meta.time_updated),
        session_narrative,
        prompt_preview: prompt_preview.clone(),
        prompt_file: prompt_file.clone(),
        turns_compact_file: Some(turns_compact_file.clone()),
        messages_compact_file: Some(messages_compact_file.clone()),
        turns_file: turns_file.clone(),
        messages_file: messages_file.clone(),
        tool_calls_file: tool_calls_file.clone(),
        artifacts_dir: artifacts_dir.clone(),
        artifacts_manifest_file,
        artifact_count,
        largest_artifacts,
        totals: session_totals_value,
        hot_turns: hot_turns.clone(),
        pivotal_turns,
        hot_messages: hot_messages.clone(),
        tool_rollup: tool_rollup.clone(),
        token_efficiency,
        file_access_rollup,
        error_patterns,
        retry_chains,
        file_transition_rollup,
        session_deliverables,
        turn_dependency_edges,
        children: child_links.clone(),
    };

    acc.session_stats.push(session_stats.clone());
    acc.turns.extend(turn_digests.clone());
    acc.message_digests.extend(message_digests.clone());
    acc.tool_calls.extend(tool_digests.clone());
    acc.session_hotspots.push(trim_session_hotspot(SessionHotspot {
        session_path: session_path.to_string(),
        duration_ms: session.meta.duration_ms(),
        message_count: message_digests.len(),
        tool_call_count: tool_digests.len(),
        input_tokens: session_stats.input_tokens,
        output_tokens: session_stats.output_tokens,
        reasoning_tokens: session_stats.reasoning_tokens,
    }));
    acc.session_index.push(SessionIndexEntry {
        session_path: session_path.to_string(),
        depth,
        parent_session_path: parent_session_path(session_path),
        title: session.meta.title.clone(),
        agent: agent.clone(),
        runtime,
        session_status,
        snapshot_completeness,
        duration_ms: session.meta.duration_ms(),
        turn_count: turn_digests.len(),
        message_count: message_digests.len(),
        tool_call_count: tool_digests.len(),
        summary_file: summary_file.clone(),
        turns_compact_file: Some(turns_compact_file.clone()),
        messages_compact_file: Some(messages_compact_file.clone()),
        turns_file: turns_file.clone(),
        messages_file: messages_file.clone(),
        tool_calls_file: tool_calls_file.clone(),
    });

    if is_root {
        acc.root_task_preview = prompt_preview.clone();
        acc.root_task_file = prompt_file.clone();
    }

    write_json_pretty(session_dir.join("summary.json"), &summary)?;
    write_jsonl(
        session_dir.join("turns.compact.jsonl"),
        &to_json_values(&build_turn_compact_entries(&turn_digests))?,
    )?;
    write_jsonl(
        session_dir.join("messages.compact.jsonl"),
        &to_json_values(&build_message_compact_entries(&message_digests))?,
    )?;
    write_jsonl(session_dir.join("turns.jsonl"), &to_json_values(&turn_digests)?)?;
    write_jsonl(session_dir.join("messages.jsonl"), &to_json_values(&message_digests)?)?;
    write_jsonl(session_dir.join("tool_calls.jsonl"), &to_json_values(&tool_digests)?)?;

    Ok(ExportTreeNode {
        session_path: session_path.to_string(),
        summary_file,
        children: child_tree_nodes,
    })
}
