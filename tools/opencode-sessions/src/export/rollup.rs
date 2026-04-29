use anyhow::Result;
use std::collections::{BTreeSet, HashMap};
use std::path::Path;

use crate::constants::*;
use crate::format::*;
use crate::models::*;
use crate::export::turn::*;
use crate::export::io::*;

pub(crate) fn rollup_tools(tools: &[ToolCallDigest]) -> Vec<ToolAggregate> {
    let mut map: HashMap<String, ToolAggregate> = HashMap::new();
    for tool in tools {
        let entry = map.entry(tool.tool.clone()).or_insert_with(|| ToolAggregate {
            tool: tool.tool.clone(),
            calls: 0,
            error_calls: 0,
            total_duration_ms: 0,
            max_duration_ms: 0,
            total_output_chars: 0,
            total_input_tokens_proxy: 0,
            avg_input_tokens_proxy: None,
        });
        entry.calls += 1;
        if tool.status == "error" {
            entry.error_calls += 1;
        }
        entry.total_duration_ms += tool.duration_ms.unwrap_or_default();
        entry.max_duration_ms = entry.max_duration_ms.max(tool.duration_ms.unwrap_or_default());
        entry.total_output_chars += tool.output_chars.unwrap_or_default();
        entry.total_input_tokens_proxy += tool.input_tokens_proxy;
    }
    let mut items = map.into_values().collect::<Vec<_>>();
    for item in &mut items {
        item.avg_input_tokens_proxy = average_u64(item.total_input_tokens_proxy, item.calls);
    }
    items
}

pub(crate) fn build_file_access_rollup(turns: &[TurnDigest], tools: &[ToolCallDigest]) -> Vec<FileAccessRollupEntry> {
    let turn_by_message = build_message_turn_index(turns);
    let mut map: HashMap<String, FileAccessRollupEntry> = HashMap::new();

    for tool in tools {
        let turn_index = turn_by_message.get(&tool.message_index).copied();
        for path in &tool.read_paths {
            let entry = map.entry(path.clone()).or_insert_with(|| FileAccessRollupEntry {
                path: path.clone(),
                read_count: 0,
                modified_count: 0,
                total_output_chars: 0,
                turn_indexes: Vec::new(),
            });
            entry.read_count += 1;
            entry.total_output_chars += tool.output_chars.unwrap_or_default();
            if let Some(turn_index) = turn_index && !entry.turn_indexes.contains(&turn_index) {
                entry.turn_indexes.push(turn_index);
            }
        }
        for path in &tool.modified_paths {
            let entry = map.entry(path.clone()).or_insert_with(|| FileAccessRollupEntry {
                path: path.clone(),
                read_count: 0,
                modified_count: 0,
                total_output_chars: 0,
                turn_indexes: Vec::new(),
            });
            entry.modified_count += 1;
            if let Some(turn_index) = turn_index && !entry.turn_indexes.contains(&turn_index) {
                entry.turn_indexes.push(turn_index);
            }
        }
    }

    let mut items = map.into_values().collect::<Vec<_>>();
    items.sort_by(|left, right| {
        right
            .read_count
            .cmp(&left.read_count)
            .then_with(|| right.modified_count.cmp(&left.modified_count))
            .then_with(|| right.total_output_chars.cmp(&left.total_output_chars))
            .then_with(|| left.path.cmp(&right.path))
    });
    for item in &mut items {
        item.turn_indexes.sort();
        item.turn_indexes.truncate(TURN_FILE_SAMPLE_LIMIT * 2);
    }
    items.truncate(FILE_ACCESS_ROLLUP_LIMIT);
    items
}

pub(crate) fn build_error_patterns(turns: &[TurnDigest], tools: &[ToolCallDigest]) -> Vec<ErrorPatternEntry> {
    let turn_by_message = build_message_turn_index(turns);
    let mut map: HashMap<(String, String), ErrorPatternEntry> = HashMap::new();

    for tool in tools.iter().filter(|tool| tool.status == "error") {
        let error_type = tool.error_type.clone().unwrap_or_else(|| String::from("tool-error"));
        let entry = map
            .entry((tool.tool.clone(), error_type.clone()))
            .or_insert_with(|| ErrorPatternEntry {
                tool: tool.tool.clone(),
                error_type: error_type.clone(),
                count: 0,
                turn_indexes: Vec::new(),
                sample_message_index: tool.message_index,
                sample_error_preview: tool.error_preview.clone(),
            });
        entry.count += 1;
        if let Some(turn_index) = turn_by_message.get(&tool.message_index).copied()
            && !entry.turn_indexes.contains(&turn_index)
        {
            entry.turn_indexes.push(turn_index);
        }
    }

    let mut items = map.into_values().collect::<Vec<_>>();
    items.sort_by(|left, right| {
        right
            .count
            .cmp(&left.count)
            .then_with(|| left.tool.cmp(&right.tool))
            .then_with(|| left.error_type.cmp(&right.error_type))
    });
    items.truncate(ERROR_PATTERN_LIMIT);
    items
}

pub(crate) fn build_retry_chains(turns: &[TurnDigest], tools: &[ToolCallDigest]) -> Vec<RetryChainEntry> {
    let turn_by_message = build_message_turn_index(turns);
    let mut chains = Vec::new();
    let mut current: Option<RetryChainEntry> = None;

    for tool in tools.iter().filter(|tool| tool.status == "error") {
        let turn_index = turn_by_message.get(&tool.message_index).copied().unwrap_or_default();
        let error_type = tool.error_type.clone().unwrap_or_else(|| String::from("tool-error"));
        match &mut current {
            Some(chain)
                if chain.turn_index == turn_index
                    && chain.tool == tool.tool
                    && chain.error_type == error_type
                    && tool.message_index <= chain.end_message_index + 1 =>
            {
                chain.attempts += 1;
                chain.end_message_index = tool.message_index;
            }
            Some(chain) => {
                chains.push(chain.clone());
                current = Some(RetryChainEntry {
                    turn_index,
                    tool: tool.tool.clone(),
                    error_type,
                    attempts: 1,
                    start_message_index: tool.message_index,
                    end_message_index: tool.message_index,
                    recovery_strategy: None,
                    sample_error_preview: tool.error_preview.clone(),
                });
            }
            None => {
                current = Some(RetryChainEntry {
                    turn_index,
                    tool: tool.tool.clone(),
                    error_type,
                    attempts: 1,
                    start_message_index: tool.message_index,
                    end_message_index: tool.message_index,
                    recovery_strategy: None,
                    sample_error_preview: tool.error_preview.clone(),
                });
            }
        }
    }
    if let Some(chain) = current {
        chains.push(chain);
    }
    chains.retain(|chain| chain.attempts > 1);
    for chain in &mut chains {
        chain.recovery_strategy = tools
            .iter()
            .find(|tool| {
                turn_by_message.get(&tool.message_index).copied().unwrap_or_default() == chain.turn_index
                    && tool.message_index > chain.end_message_index
            })
            .map(|tool| match tool.tool.as_str() {
                "read" | "grep" | "glob" => String::from("re-read-and-retry"),
                "apply_patch" if chain.tool != "apply_patch" => String::from("change-approach"),
                "bash" => String::from("verify-or-build"),
                _ if tool.tool == chain.tool => String::from("retry"),
                _ => String::from("change-approach"),
            })
            .or_else(|| Some(String::from("abandon")));
    }
    chains
}

pub(crate) fn build_file_transition_rollup(turns: &[TurnDigest], tools: &[ToolCallDigest], session_status: &str) -> Vec<FileTransitionEntry> {
    let mut writes_by_path: HashMap<String, Vec<usize>> = HashMap::new();
    let mut reads_by_path: HashMap<String, BTreeSet<usize>> = HashMap::new();
    let mut final_presence_by_path: HashMap<String, bool> = HashMap::new();

    for tool in tools {
        for (path, present) in &tool.modified_path_presence {
            final_presence_by_path.insert(path.clone(), *present);
        }
    }

    for turn in turns {
        for path in &turn.modified_files_all {
            writes_by_path
                .entry(path.clone())
                .or_default()
                .push(turn.turn_index);
        }
        for path in &turn.read_files_all {
            reads_by_path
                .entry(path.clone())
                .or_default()
                .insert(turn.turn_index);
        }
    }

    let mut items = writes_by_path
        .into_iter()
        .map(|(path, mut write_turns)| {
            write_turns.sort_unstable();
            write_turns.dedup();
            let first_write_turn = write_turns.first().copied().unwrap_or_default();
            let rewritten_in_turns = write_turns.iter().copied().skip(1).collect::<Vec<_>>();
            let supersession_chain = write_turns
                .windows(2)
                .map(|pair| FileSupersessionEntry {
                    written_in_turn: pair[0],
                    superseded_by_turn: pair[1],
                })
                .collect::<Vec<_>>();
            let survives_to_end = rewritten_in_turns.is_empty()
                && final_presence_by_path
                    .get(&path)
                    .copied()
                    .unwrap_or(true);
            let reread_in_turns = reads_by_path
                .get(&path)
                .map(|turn_indexes| {
                    turn_indexes
                        .iter()
                        .copied()
                        .filter(|turn_index| *turn_index > first_write_turn)
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();

            FileTransitionEntry {
                path,
                write_count: write_turns.len(),
                write_turns,
                reread_in_turns,
                rewritten_in_turns: rewritten_in_turns.clone(),
                supersession_chain,
                survives_to_end: (session_status != "running").then_some(survives_to_end),
            }
        })
        .collect::<Vec<_>>();

    items.sort_by(|left, right| {
        right
            .write_count
            .cmp(&left.write_count)
            .then_with(|| right.reread_in_turns.len().cmp(&left.reread_in_turns.len()))
            .then_with(|| right.rewritten_in_turns.len().cmp(&left.rewritten_in_turns.len()))
            .then_with(|| right.write_turns.last().cmp(&left.write_turns.last()))
            .then_with(|| left.path.cmp(&right.path))
    });
    items.truncate(FILE_ACCESS_ROLLUP_LIMIT);
    items
}

pub(crate) fn build_session_deliverables(
    turns: &[TurnDigest],
    tools: &[ToolCallDigest],
    session_dir: &Path,
    relative_session_dir: &Path,
) -> Result<Vec<SessionDeliverableEntry>> {
    let mut writes_by_path: HashMap<String, Vec<usize>> = HashMap::new();
    let mut final_patch_intent_by_path: HashMap<String, String> = HashMap::new();
    let mut final_presence_by_path: HashMap<String, bool> = HashMap::new();

    for turn in turns {
        for path in &turn.modified_files_all {
            writes_by_path
                .entry(path.clone())
                .or_default()
                .push(turn.turn_index);
        }
    }

    for tool in tools {
        let Some(intent) = tool.patch_intent.as_ref() else {
            for (path, present) in &tool.modified_path_presence {
                final_presence_by_path.insert(path.clone(), *present);
            }
            continue;
        };
        for (path, present) in &tool.modified_path_presence {
            final_presence_by_path.insert(path.clone(), *present);
            final_patch_intent_by_path.insert(path.clone(), intent.clone());
        }
    }

    let mut items = writes_by_path
        .into_iter()
        .filter(|(path, _)| final_presence_by_path.get(path).copied().unwrap_or(true))
        .map(|(path, mut write_turns)| {
            write_turns.sort_unstable();
            write_turns.dedup();
            let snapshot = build_deliverable_snapshot(&path, session_dir, relative_session_dir)?;
            Ok(SessionDeliverableEntry {
                final_turn_index: write_turns.last().copied().unwrap_or_default(),
                write_count: write_turns.len(),
                final_patch_intent: final_patch_intent_by_path.get(&path).cloned(),
                snapshot_file: snapshot.as_ref().map(|item| item.snapshot_file.clone()),
                content_sha256: snapshot.as_ref().map(|item| item.content_sha256.clone()),
                line_count: snapshot.as_ref().map(|item| item.line_count),
                snapshot_source: snapshot.as_ref().map(|item| item.snapshot_source.clone()),
                path,
            })
        })
        .collect::<Result<Vec<_>>>()?;

    items.sort_by(|left, right| {
        right
            .final_turn_index
            .cmp(&left.final_turn_index)
            .then_with(|| right.write_count.cmp(&left.write_count))
            .then_with(|| left.path.cmp(&right.path))
    });
    items.truncate(FILE_ACCESS_ROLLUP_LIMIT);
    Ok(items)
}

pub(crate) fn build_turn_dependency_edges(transitions: &[FileTransitionEntry]) -> Vec<TurnDependencyEdge> {
    let mut by_turn_pair: HashMap<(usize, usize), Vec<String>> = HashMap::new();
    for transition in transitions {
        for edge in &transition.supersession_chain {
            by_turn_pair
                .entry((edge.written_in_turn, edge.superseded_by_turn))
                .or_default()
                .push(transition.path.clone());
        }
    }

    let mut items = by_turn_pair
        .into_iter()
        .map(|((from_turn, to_turn), mut paths)| {
            paths.sort();
            paths.dedup();
            TurnDependencyEdge {
                from_turn,
                to_turn,
                relation: String::from("rewrote-file"),
                file_count: paths.len(),
                sample_paths: sample_strings(paths, PATCH_FILE_SAMPLE_LIMIT),
            }
        })
        .collect::<Vec<_>>();

    items.sort_by(|left, right| {
        right
            .file_count
            .cmp(&left.file_count)
            .then_with(|| right.to_turn.cmp(&left.to_turn))
            .then_with(|| left.from_turn.cmp(&right.from_turn))
    });
    items.truncate(FILE_ACCESS_ROLLUP_LIMIT);
    items
}
