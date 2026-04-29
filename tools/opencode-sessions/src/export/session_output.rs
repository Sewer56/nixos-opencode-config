use anyhow::Result;
use serde_json::Value;
use std::collections::{BTreeSet, HashMap};
use std::path::Path;

use crate::constants::*;
use crate::format::*;
use crate::models::*;
use crate::export::turn::*;
use crate::export::classify::*;

pub(crate) struct SessionOutput {
    pub(crate) turn_digests: Vec<TurnDigest>,
    pub(crate) message_digests: Vec<MessageDigest>,
    pub(crate) tool_digests: Vec<ToolCallDigest>,
    pub(crate) runtime: SessionRuntime,
    pub(crate) prompt_preview: Option<String>,
    pub(crate) prompt_file: Option<String>,
    pub(crate) artifacts_dir: Option<String>,
    pub(crate) artifact_count: usize,
}
use crate::export::io::*;

pub(crate) fn compact_message(
    message: &LoadedMessage,
    session_path: &str,
    depth: usize,
    message_index: usize,
) -> Result<CompactMessage> {
    let parts = message
        .parts
        .iter()
        .filter_map(compact_part)
        .collect::<Vec<_>>();

    Ok(CompactMessage {
        session_path: session_path.to_string(),
        depth,
        message_index,
        message_id: message.id.clone(),
        role: message.info.role.clone(),
        agent: message.info.agent.clone(),
        parent_message_id: message.info.parent_id.clone(),
        model: message.info.model_name(),
        provider: message.info.provider_name(),
        created_ms: message.info.created_ms(message.time_created),
        completed_ms: message.info.completed_ms(),
        duration_ms: message.info.duration_ms(),
        finish: message.info.finish.clone(),
        cost: message.info.cost,
        tokens: message.info.tokens.as_ref().map(|tokens| TokenStatsExport {
            total: tokens.total,
            input: tokens.input,
            output: tokens.output,
            reasoning: tokens.reasoning,
            cache_read: tokens.cache.read,
            cache_write: tokens.cache.write,
        }),
        parts,
    })
}

pub(crate) fn build_session_machine_output(
    session: &LoadedSession,
    compact_messages: &[CompactMessage],
    child_links: &[ChildLink],
    session_dir: &Path,
    relative_session_dir: &Path,
    session_path: &str,
) -> Result<SessionOutput> {
    let artifacts_dir = session_dir.join("artifacts");
    let artifacts_rel_dir = relative_session_dir.join("artifacts");
    let mut artifacts_created = false;
    let mut artifact_count = 0usize;
    let mut aw = ArtifactWriter {
        artifacts_dir: &artifacts_dir,
        artifacts_rel_dir: &artifacts_rel_dir,
        artifacts_created: &mut artifacts_created,
        artifact_count: &mut artifact_count,
    };
    let mut prompt_preview = None;
    let mut prompt_file = None;
    let child_links_by_id: HashMap<&str, &ChildLink> = child_links.iter().map(|item| (item.session_id.as_str(), item)).collect();
    let mut message_digests = Vec::new();
    let mut tool_digests = Vec::new();
    let runtime = SessionRuntime {
        models: compact_messages
            .iter()
            .filter_map(|message| message.model.clone())
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect(),
        providers: compact_messages
            .iter()
            .filter_map(|message| message.provider.clone())
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect(),
    };
    let emit_message_model = runtime.models.len() > 1;
    let emit_message_provider = runtime.providers.len() > 1;

    for (message_index, (message, compact)) in session.messages.iter().zip(compact_messages).enumerate() {
        let text_blocks = compact
            .parts
            .iter()
            .filter_map(|part| match part {
                CompactPart::Text { text, synthetic } if !synthetic => Some(text.as_str()),
                _ => None,
            })
            .collect::<Vec<_>>();
        let reasoning_blocks = compact
            .parts
            .iter()
            .filter_map(|part| match part {
                CompactPart::Reasoning { text } => Some(text.as_str()),
                _ => None,
            })
            .collect::<Vec<_>>();
        let text = join_blocks(&text_blocks);
        let reasoning = join_blocks(&reasoning_blocks);
        let text_chars = text.chars().count();
        let reasoning_chars = reasoning.chars().count();
        let text_preview = (!text.is_empty()).then(|| truncate_text(&text, INLINE_TEXT_PREVIEW_LIMIT));
        let user_classification = (compact.role == "user").then(|| classify_user_intent(text_preview.as_deref()));
        let user_intent = user_classification.as_ref().map(|(intent, _)| intent.clone());
        let user_intent_confidence = user_classification.as_ref().map(|(_, confidence)| *confidence);
        let alternative_user_intents = user_classification
            .as_ref()
            .map(|(intent, confidence)| classify_alternative_user_intents(text_preview.as_deref(), intent, *confidence))
            .unwrap_or_default();
        let user_tags = if compact.role == "user" { classify_user_tags(text_preview.as_deref()) } else { Default::default() };
        let mut message_activity_items = Vec::new();
        let reasoning_themes = extract_reasoning_themes(&reasoning);
        let reasoning_summary = (reasoning_chars > REASONING_PREVIEW_LIMIT)
            .then(|| summarize_reasoning(&reasoning, &reasoning_themes))
            .flatten();
        let reasoning_preview = (!reasoning.is_empty()).then(|| truncate_text(&reasoning, REASONING_PREVIEW_LIMIT));

        let mut text_file = if text_chars > MESSAGES_EMBEDDED_TEXT_LIMIT
            || (compact.role == "assistant" && text_chars > ASSISTANT_TEXT_ARTIFACT_CHARS_THRESHOLD)
        {
            write_text_artifact(
                &mut aw,
                &format!("message-{message_index:03}-text.txt"),
                &text,
                true,
                0,
            )?
        } else {
            None
        };

        if compact.role == "user" && !text.is_empty() && text_file.is_none() {
            text_file = write_text_artifact(
                &mut aw,
                &format!("message-{message_index:03}-user.txt"),
                &text,
                true,
                0,
            )?;
        }

        if prompt_preview.is_none() && compact.role == "user" && !text.is_empty() {
            prompt_preview = Some(truncate_text(&text, TEXT_PREVIEW_LIMIT));
            if text_file.is_none() {
                text_file = write_text_artifact(
                    &mut aw,
                    &format!("message-{message_index:03}-prompt.txt"),
                    &text,
                    true,
                    0,
                )?;
            }
            prompt_file = text_file.clone();
        }

        let reasoning_file = if reasoning_chars > REASONING_ARTIFACT_CHARS_THRESHOLD {
            write_text_artifact(
                &mut aw,
                &format!("message-{message_index:03}-reasoning.txt"),
                &reasoning,
                true,
                0,
            )?
        } else {
            None
        };

        let tool_names = compact
            .parts
            .iter()
            .filter_map(|part| match part {
                CompactPart::Tool { tool, .. } => Some(tool.clone()),
                _ => None,
            })
            .collect::<Vec<_>>();
        let subagent_calls = compact
            .parts
            .iter()
            .filter_map(|part| match part {
                CompactPart::Agent { name } => Some(name.clone()),
                _ => None,
            })
            .collect::<Vec<_>>();
        let subtask_agents = compact
            .parts
            .iter()
            .filter_map(|part| match part {
                CompactPart::Subtask { agent, .. } => Some(agent.clone()),
                _ => None,
            })
            .collect::<Vec<_>>();
        let patch_file_count = compact
            .parts
            .iter()
            .map(|part| match part {
                CompactPart::Patch { file_count, .. } => *file_count,
                _ => 0,
            })
            .sum();
        let file_attachment_count = compact
            .parts
            .iter()
            .filter(|part| matches!(part, CompactPart::File { .. }))
            .count();
        let tool_error_count = compact
            .parts
            .iter()
            .filter(|part| matches!(part, CompactPart::Tool { status, .. } if status == "error"))
            .count();

        let next_message_time = compact_messages.get(message_index + 1).map(|next| next.created_ms);
        let wall_gap_ms = next_message_time.map(|next| next.saturating_sub(compact.created_ms));

        let tool_parts_count = message
            .parts
            .iter()
            .filter(|part| part.raw.get("type").and_then(Value::as_str) == Some("tool"))
            .count();
        let input_tokens_proxy_per_tool = compact
            .tokens
            .as_ref()
            .map(|tokens| tokens.input / tool_parts_count.max(1) as u64)
            .unwrap_or_default();

        for (tool_index, part) in message
            .parts
            .iter()
            .filter(|part| part.raw.get("type").and_then(Value::as_str) == Some("tool"))
            .enumerate()
        {
            let Some(state) = part.raw.get("state") else {
                continue;
            };
            let Some(tool) = part.raw.get("tool").and_then(Value::as_str) else {
                continue;
            };
            let status = state
                .get("status")
                .and_then(Value::as_str)
                .unwrap_or("unknown")
                .to_string();
            let input_value = state.get("input");
            let input_text = input_value
                .and_then(|value| serde_json::to_string(value).ok());
            let input_preview = input_value
                .map(|value| {
                    shrink_json(
                        value,
                        TOOL_INPUT_STRING_LIMIT,
                        TOOL_INPUT_ITEMS_LIMIT,
                        TOOL_INPUT_DEPTH_LIMIT,
                    )
                })
                .map(|value| normalize_tool_input_preview(tool, value));
            let patch_text = if tool == "apply_patch" {
                input_value
                    .and_then(|value| value.get("patchText"))
                    .and_then(Value::as_str)
            } else {
                None
            };
            let input_file = (tool != "apply_patch")
                .then_some(input_value)
                .flatten()
                .filter(|_| {
                    input_text
                        .as_ref()
                        .map(|text| text.len() > TOOL_INPUT_INLINE_CHARS_THRESHOLD)
                        .unwrap_or(true)
                })
                .map(|value| {
                    write_json_artifact(
                        &mut aw,
                        &format!("tool-{message_index:03}-{tool_index:02}-input.json"),
                        value,
                        true,
                        0,
                    )
                })
                .transpose()?
                .flatten();

            let task_description = input_value
                .and_then(|value| value.get("description"))
                .and_then(Value::as_str)
                .map(|value| truncate_text(value.trim(), TOOL_PART_PREVIEW_LIMIT));
            let task_prompt_preview = input_value
                .and_then(|value| value.get("prompt"))
                .and_then(Value::as_str)
                .map(|value| truncate_text(value.trim(), SUBTASK_PREVIEW_LIMIT));

            let output_text = state.get("output").and_then(Value::as_str).unwrap_or("");

            let delegated_session_id = if tool == "task" {
                extract_task_id(output_text).and_then(|task_id| {
                    child_links_by_id
                        .contains_key(task_id.as_str())
                        .then_some(task_id)
                })
            } else {
                None
            };
            let patch_summary = patch_text.and_then(summarize_patch_text);
            let read_paths = tool_read_paths(tool, input_value);
            let modified_paths = patch_text.map(patch_paths_from_text).unwrap_or_default();
            let modified_path_presence = if let Some(p) = patch_text {
                patch_path_presence_from_text(p)
            } else {
                HashMap::new()
            };
            let effective_read_paths = if status != "error" { read_paths.clone() } else { Vec::new() };
            let effective_modified_paths = if status != "error" { modified_paths.clone() } else { Vec::new() };
            let call_purpose = classify_tool_call_purpose(tool, task_description.as_deref(), &effective_read_paths, &effective_modified_paths);
            let patch_intent = patch_summary
                .as_ref()
                .and_then(|summary| classify_patch_intent(&modified_paths, summary));
            if let Some(activity) = summarize_tool_activity(
                tool,
                &status,
                task_description.as_deref(),
                input_value,
                patch_summary.as_ref(),
                &modified_paths,
            ) {
                message_activity_items.push(activity);
            }
            let patch_file = patch_text
                .filter(|text| text.chars().count() > TOOL_INPUT_STRING_LIMIT)
                .map(|text| {
                    write_text_artifact(
                        &mut aw,
                        &format!("tool-{message_index:03}-{tool_index:02}-patch.diff"),
                        text,
                        true,
                        0,
                    )
                })
                .transpose()?
                .flatten();

            let output_chars = (!output_text.is_empty()).then_some(output_text.chars().count());
            let output_preview = (!output_text.is_empty()).then(|| truncate_text(output_text, TOOL_TEXT_PREVIEW_LIMIT));
            let output_file = if output_chars.unwrap_or_default() > TOOL_CALLS_EMBEDDED_IO_LIMIT {
                write_text_artifact(
                    &mut aw,
                    &format!("tool-{message_index:03}-{tool_index:02}-output.txt"),
                    output_text,
                    true,
                    0,
                )?
            } else {
                None
            };

            let error_text = state.get("error").and_then(Value::as_str).unwrap_or("");
            let error_preview = (!error_text.is_empty()).then(|| truncate_text(error_text, TOOL_TEXT_PREVIEW_LIMIT));
            let error_type = classify_tool_error(tool, &status, error_text);
            let error_file = if error_text.chars().count() > TOOL_CALLS_EMBEDDED_IO_LIMIT {
                write_text_artifact(
                    &mut aw,
                    &format!("tool-{message_index:03}-{tool_index:02}-error.txt"),
                    error_text,
                    true,
                    0,
                )?
            } else {
                None
            };

            let duration_ms = state
                .get("time")
                .and_then(Value::as_object)
                .and_then(|time| {
                    time.get("start")
                        .and_then(Value::as_i64)
                        .zip(time.get("end").and_then(Value::as_i64))
                        .map(|(start, end)| end.saturating_sub(start))
                });

            tool_digests.push(ToolCallDigest {
                session_path: session_path.to_string(),
                message_index,
                turn_index: None,
                tool_index,
                tool: tool.to_string(),
                status,
                title: non_empty_owned(state.get("title").and_then(Value::as_str)),
                duration_ms,
                input_preview,
                input_file,
                task_description,
                task_prompt_preview,
                delegated_session_id,
                call_purpose,
                patch_summary,
                patch_intent,
                patch_file,
                output_preview,
                output_chars,
                output_file,
                error_preview,
                error_type,
                error_file,
                read_paths: effective_read_paths,
                modified_paths: effective_modified_paths,
                modified_path_presence,
                input_tokens_proxy: input_tokens_proxy_per_tool,
            });
        }

        message_digests.push(MessageDigest {
            session_path: session_path.to_string(),
            message_index,
            turn_index: None,
            role: compact.role.clone(),
            message_kind: classify_message_kind(
                compact.role.as_str(),
                !text.is_empty(),
                !reasoning.is_empty(),
                !message_activity_items.is_empty(),
            ),
            time_ms: compact.created_ms,
            model: emit_message_model.then(|| compact.model.clone()).flatten(),
            provider: emit_message_provider.then(|| compact.provider.clone()).flatten(),
            wall_gap_ms,
            duration_ms: compact.duration_ms,
            tokens: compact.tokens.clone().filter(|tokens| !tokens.is_empty()),
            user_intent,
            user_intent_confidence,
            user_tags,
            alternative_user_intents,
            text_chars,
            text_preview,
            text_file,
            activity_summary: summarize_activity_items(&message_activity_items),
            reasoning_chars,
            reasoning_summary,
            reasoning_themes,
            reasoning_preview,
            reasoning_file,
            tool_count: tool_names.len(),
            tool_error_count,
            tool_names,
            subagent_calls,
            subtask_agents,
            patch_file_count,
            file_attachment_count,
        });
    }

    let turn_digests = build_turn_digests(
        session_path,
        &message_digests,
        &tool_digests,
        child_links,
    );

    for tool in &mut tool_digests {
        tool.turn_index = turn_digests.iter().find_map(|turn| {
            (tool.message_index >= turn.user_message_index && tool.message_index <= turn.message_index_end).then_some(turn.turn_index)
        });
    }

    for message in &mut message_digests {
        message.turn_index = turn_digests.iter().find_map(|turn| {
            if turn.user_message_index == message.message_index {
                return Some(turn.turn_index);
            }
            turn.assistant_message_start
                .zip(turn.assistant_message_end)
                .filter(|(start, end)| message.message_index >= *start && message.message_index <= *end)
                .map(|_| turn.turn_index)
        });
    }

    Ok(SessionOutput {
        turn_digests,
        message_digests,
        tool_digests,
        runtime,
        prompt_preview,
        prompt_file,
        artifacts_dir: (artifact_count > 0).then(|| path_string(&artifacts_rel_dir)),
        artifact_count,
    })
}

pub(crate) fn build_turn_compact_entries(turns: &[TurnDigest]) -> Vec<TurnCompactEntry> {
    turns
        .iter()
        .map(|turn| TurnCompactEntry {
            turn_index: turn.turn_index,
            user_message_index: turn.user_message_index,
            message_index_end: turn.message_index_end,
            user_intent: turn.user_intent.clone(),
            response_elapsed_ms: turn.response_elapsed_ms,
            total_tokens: turn.total_tokens,
            tool_call_count: turn.tool_call_count,
            modified_file_count: turn.modified_file_count,
            agent_strategy: turn.agent_strategy.clone(),
            outcome: turn.outcome.clone(),
            success: turn.success,
            turn_cost_tier: turn.turn_cost_tier.clone(),
            turn_effectiveness: turn.turn_effectiveness.clone(),
            recommended_attention: turn.recommended_attention.clone(),
            optimization_hints: turn.optimization_hints.clone(),
            failure_narrative: turn.failure_narrative.clone(),
            reasoning_summary: turn.reasoning_summary.clone(),
            turn_change_summary: turn.turn_change_summary.clone(),
            key_diff_preview: turn.key_diff_preview.clone(),
        })
        .collect()
}

pub(crate) fn build_message_compact_entries(messages: &[MessageDigest]) -> Vec<MessageCompactEntry> {
    messages
        .iter()
        .map(|message| MessageCompactEntry {
            message_index: message.message_index,
            turn_index: message.turn_index,
            role: message.role.clone(),
            message_kind: message.message_kind.clone(),
            time_ms: message.time_ms,
            wall_gap_ms: message.wall_gap_ms,
            duration_ms: message.duration_ms,
            total_tokens: message.tokens.as_ref().and_then(|tokens| tokens.total),
            tool_count: message.tool_count,
            tool_error_count: message.tool_error_count,
            text_preview: message.text_preview.clone(),
            activity_summary: message.activity_summary.clone(),
            reasoning_summary: message.reasoning_summary.clone(),
        })
        .collect()
}

pub(crate) fn infer_session_status(
    session: &LoadedSession,
    compact_messages: &[CompactMessage],
    tools: &[ToolCallDigest],
) -> String {
    if tools.last().map(|tool| tool.status.as_str()) == Some("running") {
        return String::from("running");
    }

    let Some(last_loaded) = session.messages.last() else {
        return String::from("abandoned");
    };
    let last_compact = compact_messages.last();

    if last_loaded.info.error.is_some() || matches!(last_compact.and_then(|message| message.finish.as_deref()), Some("error")) {
        return String::from("error");
    }
    if last_loaded.info.role == "assistant" {
        if let Some(finish) = last_compact.and_then(|message| message.finish.as_deref()) {
            if finish == "tool-calls" || finish == "unknown" {
                return String::from("abandoned");
            }
            return String::from("completed");
        }
        if tools.last().map(|tool| tool.status.as_str()) == Some("error") {
            return String::from("error");
        }
        return String::from("abandoned");
    }
    if last_loaded.info.role == "user" {
        return String::from("abandoned");
    }
    String::from("abandoned")
}

pub(crate) fn compact_part(part: &LoadedPart) -> Option<CompactPart> {
    let part_type = part.raw.get("type")?.as_str()?;
    match part_type {
        "text" => {
            let text = part.raw.get("text")?.as_str()?.trim().to_string();
            if text.is_empty() {
                return None;
            }
            Some(CompactPart::Text {
                text,
                synthetic: part.raw.get("synthetic").and_then(Value::as_bool).unwrap_or(false),
            })
        }
        "reasoning" => Some(CompactPart::Reasoning {
            text: part.raw.get("text")?.as_str()?.trim().to_string(),
        }),
        "tool" => {
            let tool = part.raw.get("tool")?.as_str()?.to_string();
            let state = part.raw.get("state")?;
            let status = state.get("status")?.as_str()?.to_string();
            let input = state
                .get("input")
                .map(|value| {
                    shrink_json(
                        value,
                        TOOL_INPUT_STRING_LIMIT,
                        TOOL_INPUT_ITEMS_LIMIT,
                        TOOL_INPUT_DEPTH_LIMIT + 1,
                    )
                });
            let output_preview = state
                .get("output")
                .and_then(Value::as_str)
                .map(|value| truncate_text(value.trim(), TOOL_PART_PREVIEW_LIMIT));
            let output_chars = state
                .get("output")
                .and_then(Value::as_str)
                .map(str::len);
            let error = state
                .get("error")
                .and_then(Value::as_str)
                .map(|value| truncate_text(value.trim(), TOOL_PART_PREVIEW_LIMIT));
            let title = state
                .get("title")
                .and_then(Value::as_str)
                .map(str::to_string);
            let duration_ms = state
                .get("time")
                .and_then(Value::as_object)
                .and_then(|time| {
                    time.get("start")
                        .and_then(Value::as_i64)
                        .zip(time.get("end").and_then(Value::as_i64))
                        .map(|(start, end)| end.saturating_sub(start))
                });

            Some(CompactPart::Tool {
                tool,
                status,
                title,
                duration_ms,
                input,
                output_preview,
                output_chars,
                error,
            })
        }
        "agent" => Some(CompactPart::Agent {
            name: part.raw.get("name")?.as_str()?.to_string(),
        }),
        "subtask" => Some(CompactPart::Subtask {
            agent: part.raw.get("agent")?.as_str()?.to_string(),
            description: truncate_text(part.raw.get("description")?.as_str()?.trim(), SUBTASK_PREVIEW_LIMIT),
            prompt: truncate_text(part.raw.get("prompt")?.as_str()?.trim(), SUBTASK_PREVIEW_LIMIT),
            command: part
                .raw
                .get("command")
                .and_then(Value::as_str)
                .map(|value| truncate_text(value.trim(), TOOL_PART_PREVIEW_LIMIT)),
        }),
        "file" => Some(CompactPart::File {
            filename: part
                .raw
                .get("filename")
                .and_then(Value::as_str)
                .map(str::to_string),
            mime: part.raw.get("mime").and_then(Value::as_str).map(str::to_string),
        }),
        "patch" => {
            let files = part
                .raw
                .get("files")
                .and_then(Value::as_array)
                .cloned()
                .unwrap_or_default();
            let collected = files
                .iter()
                .take(PATCH_FILE_SAMPLE_LIMIT)
                .filter_map(Value::as_str)
                .map(|value| truncate_text(value, TOOL_PART_PREVIEW_LIMIT))
                .collect::<Vec<_>>();
            Some(CompactPart::Patch {
                file_count: files.len(),
                files: collected,
            })
        }
        "retry" => Some(CompactPart::Retry {
            attempt: part.raw.get("attempt").and_then(Value::as_i64),
            error: part
                .raw
                .get("error")
                .and_then(Value::as_str)
                .map(|value| truncate_text(value, TOOL_PART_PREVIEW_LIMIT)),
        }),
        "step-start" | "step-finish" | "snapshot" | "compaction" => None,
        _ => None,
    }
}

pub(crate) fn compute_session_stats(
    session: &LoadedSession,
    compact_messages: &[CompactMessage],
    session_path: &str,
    depth: usize,
    agent: Option<String>,
) -> SessionStats {
    let mut stats = SessionStats {
        session_path: session_path.to_string(),
        depth,
        session_id: session.meta.id.clone(),
        parent_session_id: session.meta.parent_id.clone(),
        title: session.meta.title.clone(),
        agent,
        created_ms: session.meta.time_created,
        updated_ms: session.meta.time_updated,
        duration_ms: session.meta.duration_ms(),
        turn_count: 0,
        message_count: compact_messages.len(),
        user_message_count: 0,
        assistant_message_count: 0,
        child_session_count: session.children.len(),
        text_chars: 0,
        reasoning_chars: 0,
        tool_calls: 0,
        input_tokens: 0,
        output_tokens: 0,
        reasoning_tokens: 0,
        cache_read_tokens: 0,
        cache_write_tokens: 0,
        cost: 0.0,
    };

    for message in compact_messages {
        if message.role == "user" {
            stats.turn_count += 1;
            stats.user_message_count += 1;
        }
        if message.role == "assistant" {
            stats.assistant_message_count += 1;
        }

        for part in &message.parts {
            match part {
                CompactPart::Text { text, .. } => stats.text_chars += text.chars().count(),
                CompactPart::Reasoning { text } => stats.reasoning_chars += text.chars().count(),
                CompactPart::Tool { .. } => stats.tool_calls += 1,
                _ => {}
            }
        }

        if let Some(tokens) = &message.tokens {
            stats.input_tokens += tokens.input;
            stats.output_tokens += tokens.output;
            stats.reasoning_tokens += tokens.reasoning;
            stats.cache_read_tokens += tokens.cache_read;
            stats.cache_write_tokens += tokens.cache_write;
        }

        stats.cost += message.cost.unwrap_or(0.0);
    }

    stats
}

pub(crate) fn session_totals(stats: &SessionStats) -> SessionTotals {
    SessionTotals {
        turn_count: stats.turn_count,
        message_count: stats.message_count,
        user_message_count: stats.user_message_count,
        assistant_message_count: stats.assistant_message_count,
        child_session_count: stats.child_session_count,
        text_chars: stats.text_chars,
        reasoning_chars: stats.reasoning_chars,
        tool_calls: stats.tool_calls,
        input_tokens: stats.input_tokens,
        output_tokens: stats.output_tokens,
        reasoning_tokens: stats.reasoning_tokens,
        cache_read_tokens: stats.cache_read_tokens,
        cache_write_tokens: stats.cache_write_tokens,
        cost: stats.cost,
    }
}

pub(crate) fn parent_session_path(session_path: &str) -> Option<String> {
    session_path.rsplit_once('.').map(|(parent, _)| parent.to_string())
}
