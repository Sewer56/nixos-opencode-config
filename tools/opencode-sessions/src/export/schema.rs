use serde_json::{Value, json};

use crate::constants::*;
use crate::models::*;

pub(crate) fn build_export_schema() -> Value {
    json!({
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "title": "OpenCode Sessions Export",
        "description": "Machine-first export contract. index.json follows root schema below. summary.json files follow $defs.sessionSummary. turns.jsonl/messages.jsonl/tool_calls.jsonl use one JSON object per line matching $defs.turnDigest/$defs.messageDigest/$defs.toolCallDigest.",
        "type": "object",
        "additionalProperties": false,
        "required": [
            "format",
            "schema_version",
            "schema_file",
            "fields_file",
            "export_id",
            "export_timestamp_ms",
            "iteration_meta",
            "root_session_id",
            "root_title",
            "root_snapshot_completeness",
            "recommended_read_order",
            "totals",
            "tree",
            "session_index",
            "hotspots"
        ],
        "properties": {
            "format": { "const": "opencode-sessions-v1" },
            "schema_version": { "type": "string" },
            "schema_file": { "const": "schema.json" },
            "fields_file": { "const": "fields.json" },
            "export_id": { "type": "string" },
            "export_timestamp_ms": { "type": "integer" },
            "iteration_meta": { "$ref": "#/$defs/iterationMeta" },
            "delta_from_previous": { "type": ["object", "null"], "additionalProperties": true },
            "root_session_id": { "type": "string" },
            "root_title": { "type": "string" },
            "root_session_status": { "type": "string" },
            "root_snapshot_completeness": { "type": "string", "enum": ["final", "live-running-snapshot", "stale-running-snapshot", "partial"] },
            "root_last_activity_ms": { "type": "integer" },
            "root_staleness_ms": { "type": "integer" },
            "root_task_preview": { "type": ["string", "null"] },
            "root_task_file": { "type": ["string", "null"] },
            "schema_changes": { "type": "array", "items": { "type": "string" } },
            "artifact_policy": { "$ref": "#/$defs/artifactPolicy" },
            "classification_policy": { "$ref": "#/$defs/classificationPolicy" },
            "recommended_read_order": { "type": "array", "items": { "type": "string" } },
            "totals": { "$ref": "#/$defs/exportTotals" },
            "token_efficiency": { "$ref": "#/$defs/tokenEfficiency" },
            "tree": { "$ref": "#/$defs/exportTreeNode" },
            "session_index": { "type": "array", "items": { "$ref": "#/$defs/sessionIndexEntry" } },
            "tool_rollup": { "type": "array", "items": { "$ref": "#/$defs/toolAggregate" } },
            "hotspots": { "$ref": "#/$defs/exportHotspots" }
        },
        "$defs": {
            "iterationMeta": {
                "type": "object",
                "additionalProperties": false,
                "required": ["group_key", "iteration_number"],
                "properties": {
                    "group_key": { "type": "string" },
                    "iteration_number": { "type": "integer" },
                    "previous_export_path": { "type": ["string", "null"] }
                }
            },
            "artifactPolicy": {
                "type": "object",
                "additionalProperties": false,
                "required": ["assistant_text_file_chars", "reasoning_file_chars", "tool_input_inline_chars", "tool_output_inline_chars"],
                "properties": {
                    "assistant_text_file_chars": { "type": "integer" },
                    "reasoning_file_chars": { "type": "integer" },
                    "tool_input_inline_chars": { "type": "integer" },
                    "tool_output_inline_chars": { "type": "integer" }
                }
            },
            "confidenceThresholds": {
                "type": "object",
                "additionalProperties": false,
                "required": ["reliable_above", "uncertain_below"],
                "properties": {
                    "reliable_above": { "type": "number" },
                    "uncertain_below": { "type": "number" }
                }
            },
            "classificationPolicy": {
                "type": "object",
                "additionalProperties": false,
                "required": [
                    "version",
                    "user_intent_values",
                    "user_tag_values",
                    "message_kind_values",
                    "outcome_values",
                    "assistant_kind_values",
                    "session_status_values",
                    "agent_strategy_values",
                    "turn_cost_tier_values",
                    "turn_effectiveness_values",
                    "recommended_attention_values",
                    "child_export_reference_status_values",
                    "patch_intent_values",
                    "tool_call_purpose_values",
                    "retry_recovery_values",
                    "intent_confidence_range",
                    "confidence_thresholds"
                ],
                "properties": {
                    "version": { "type": "string" },
                    "user_intent_values": { "type": "array", "items": { "type": "string" } },
                    "user_tag_values": { "type": "array", "items": { "type": "string" } },
                    "message_kind_values": { "type": "array", "items": { "type": "string" } },
                    "outcome_values": { "type": "array", "items": { "type": "string" } },
                    "assistant_kind_values": { "type": "array", "items": { "type": "string" } },
                    "session_status_values": { "type": "array", "items": { "type": "string" } },
                    "agent_strategy_values": { "type": "array", "items": { "type": "string" } },
                    "turn_cost_tier_values": { "type": "array", "items": { "type": "string" } },
                    "turn_effectiveness_values": { "type": "array", "items": { "type": "string" } },
                    "recommended_attention_values": { "type": "array", "items": { "type": "string" } },
                    "child_export_reference_status_values": { "type": "array", "items": { "type": "string" } },
                    "patch_intent_values": { "type": "array", "items": { "type": "string" } },
                    "tool_call_purpose_values": { "type": "array", "items": { "type": "string" } },
                    "retry_recovery_values": { "type": "array", "items": { "type": "string" } },
                    "intent_confidence_range": { "type": "string" },
                    "confidence_thresholds": { "$ref": "#/$defs/confidenceThresholds" }
                }
            },
            "exportTotals": {
                "type": "object",
                "additionalProperties": false,
                "required": ["session_count", "turn_count", "message_count", "user_message_count", "assistant_message_count", "text_chars", "reasoning_chars", "tool_calls", "input_tokens", "output_tokens", "reasoning_tokens"],
                "properties": {
                    "session_count": { "type": "integer" },
                    "turn_count": { "type": "integer" },
                    "message_count": { "type": "integer" },
                    "user_message_count": { "type": "integer" },
                    "assistant_message_count": { "type": "integer" },
                    "text_chars": { "type": "integer" },
                    "reasoning_chars": { "type": "integer" },
                    "tool_calls": { "type": "integer" },
                    "input_tokens": { "type": "integer" },
                    "output_tokens": { "type": "integer" },
                    "reasoning_tokens": { "type": "integer" },
                    "cache_read_tokens": { "type": "integer" },
                    "cache_write_tokens": { "type": "integer" },
                    "cost": { "type": "number" }
                }
            },
            "tokenEfficiency": {
                "type": "object",
                "additionalProperties": false,
                "properties": {
                    "cache_hit_ratio": { "type": ["number", "null"] },
                    "avg_input_tokens_per_turn": { "type": ["number", "null"] },
                    "avg_output_tokens_per_turn": { "type": ["number", "null"] },
                    "avg_reasoning_tokens_per_turn": { "type": ["number", "null"] },
                    "avg_tool_calls_per_turn": { "type": ["number", "null"] },
                    "avg_input_tokens_per_tool_call": { "type": ["number", "null"] }
                }
            },
            "sessionIndexEntry": {
                "type": "object",
                "additionalProperties": false,
                "required": ["session_path", "title", "summary_file", "turns_file", "snapshot_completeness"],
                "properties": {
                    "session_path": { "type": "string" },
                    "parent_session_path": { "type": ["string", "null"] },
                    "title": { "type": "string" },
                    "agent": { "type": ["string", "null"] },
                    "runtime": { "$ref": "#/$defs/sessionRuntime" },
                    "session_status": { "type": "string" },
                    "snapshot_completeness": { "type": "string", "enum": ["final", "live-running-snapshot", "stale-running-snapshot", "partial"] },
                    "duration_ms": { "type": "integer" },
                    "turn_count": { "type": "integer" },
                    "summary_file": { "type": "string" },
                    "turns_compact_file": { "type": ["string", "null"] },
                    "messages_compact_file": { "type": ["string", "null"] },
                    "turns_file": { "type": "string" },
                    "message_count": { "type": "integer" },
                    "tool_call_count": { "type": "integer" }
                }
            },
            "sessionSummary": {
                "type": "object",
                "additionalProperties": false,
                "required": ["session", "session_status", "snapshot_completeness", "turns_file", "messages_file", "tool_calls_file", "totals", "token_efficiency"],
                "properties": {
                    "session": { "$ref": "#/$defs/sessionFileMeta" },
                    "runtime": { "$ref": "#/$defs/sessionRuntime" },
                    "session_status": { "type": "string" },
                    "snapshot_completeness": { "type": "string", "enum": ["final", "live-running-snapshot", "stale-running-snapshot", "partial"] },
                    "last_activity_ms": { "type": "integer" },
                    "staleness_ms": { "type": "integer" },
                    "session_narrative": { "type": ["string", "null"] },
                    "prompt_preview": { "type": ["string", "null"] },
                    "prompt_file": { "type": ["string", "null"] },
                    "artifacts_dir": { "type": ["string", "null"] },
                    "artifacts_manifest_file": { "type": ["string", "null"] },
                    "turns_compact_file": { "type": ["string", "null"] },
                    "messages_compact_file": { "type": ["string", "null"] },
                    "turns_file": { "type": "string" },
                    "messages_file": { "type": "string" },
                    "tool_calls_file": { "type": "string" },
                    "artifact_count": { "type": "integer" },
                    "largest_artifacts": { "type": "array", "items": { "$ref": "#/$defs/artifactManifestEntry" } },
                    "totals": { "$ref": "#/$defs/sessionTotals" },
                    "hot_turns": { "type": "array", "items": { "$ref": "#/$defs/turnHotspot" } },
                    "pivotal_turns": { "type": "array", "items": { "type": "integer" } },
                    "hot_messages": { "type": "array", "items": { "$ref": "#/$defs/messageHotspot" } },
                    "tool_rollup": { "type": "array", "items": { "$ref": "#/$defs/toolAggregate" } },
                    "token_efficiency": { "$ref": "#/$defs/tokenEfficiency" },
                    "file_access_rollup": { "type": "array", "items": { "$ref": "#/$defs/fileAccessRollupEntry" } },
                    "error_patterns": { "type": "array", "items": { "$ref": "#/$defs/errorPatternEntry" } },
                    "retry_chains": { "type": "array", "items": { "$ref": "#/$defs/retryChainEntry" } },
                    "file_transition_rollup": { "type": "array", "items": { "$ref": "#/$defs/fileTransitionEntry" } },
                    "session_deliverables": { "type": "array", "items": { "$ref": "#/$defs/sessionDeliverable" } },
                    "turn_dependency_edges": { "type": "array", "items": { "$ref": "#/$defs/turnDependencyEdge" } },
                    "children": { "type": "array", "items": { "$ref": "#/$defs/childLink" } }
                }
            },
            "turnCompactDigest": {
                "type": "object",
                "additionalProperties": false,
                "required": ["turn_index", "user_message_index", "message_index_end", "user_intent", "agent_strategy", "outcome", "success", "turn_cost_tier", "turn_effectiveness", "recommended_attention"],
                "properties": {
                    "turn_index": { "type": "integer" },
                    "user_message_index": { "type": "integer" },
                    "message_index_end": { "type": "integer" },
                    "user_intent": { "type": "string", "enum": ["task", "continuation", "redirect", "followup-request", "scope-change", "approval"] },
                    "response_elapsed_ms": { "type": ["integer", "null"] },
                    "total_tokens": { "type": "integer" },
                    "tool_call_count": { "type": "integer" },
                    "modified_file_count": { "type": "integer" },
                    "agent_strategy": { "type": "string", "enum": ["explore", "implement", "debug", "refactor", "validate", "delegate"] },
                    "outcome": { "type": "string", "enum": ["answered", "executed", "delegated", "redirected", "followup-needed"] },
                    "success": { "type": "boolean" },
                    "turn_cost_tier": { "type": "string", "enum": ["light", "medium", "heavy", "extreme"] },
                    "turn_effectiveness": { "type": "string", "enum": ["high-value", "moderate", "low-value", "waste"] },
                    "recommended_attention": { "type": "string", "enum": ["skip", "skim", "read-carefully", "inspect-artifacts"] },
                    "optimization_hints": { "type": "array", "items": { "type": "string" } },
                    "failure_narrative": { "type": ["string", "null"] },
                    "reasoning_summary": { "type": ["string", "null"] },
                    "turn_change_summary": { "type": ["string", "null"] },
                    "key_diff_preview": { "type": ["string", "null"] }
                }
            },
            "messageCompactDigest": {
                "type": "object",
                "additionalProperties": false,
                "required": ["message_index", "role", "message_kind", "time_ms"],
                "properties": {
                    "message_index": { "type": "integer" },
                    "turn_index": { "type": ["integer", "null"] },
                    "role": { "type": "string", "enum": ["user", "assistant"] },
                    "message_kind": { "type": "string", "enum": ["user", "assistant-text", "assistant-tool-only", "assistant-mixed", "assistant-reasoning-only"] },
                    "time_ms": { "type": "integer" },
                    "wall_gap_ms": { "type": ["integer", "null"] },
                    "duration_ms": { "type": ["integer", "null"] },
                    "total_tokens": { "type": ["integer", "null"] },
                    "tool_count": { "type": "integer" },
                    "tool_error_count": { "type": "integer" },
                    "text_preview": { "type": ["string", "null"] },
                    "activity_summary": { "type": ["string", "null"] },
                    "reasoning_summary": { "type": ["string", "null"] }
                }
            },
            "sessionDeliverable": {
                "type": "object",
                "additionalProperties": false,
                "required": ["path", "write_count", "final_turn_index"],
                "properties": {
                    "path": { "type": "string" },
                    "write_count": { "type": "integer" },
                    "final_turn_index": { "type": "integer" },
                    "final_patch_intent": { "type": ["string", "null"], "enum": [null, "feature", "fix", "refactor", "config", "test", "docs"] },
                    "snapshot_file": { "type": ["string", "null"] },
                    "content_sha256": { "type": ["string", "null"] },
                    "line_count": { "type": ["integer", "null"] },
                    "snapshot_source": { "type": ["string", "null"], "enum": [null, "workspace-current"] }
                }
            },
            "artifactManifest": {
                "type": "object",
                "additionalProperties": false,
                "required": ["artifacts_dir", "entries"],
                "properties": {
                    "artifacts_dir": { "type": "string" },
                    "total_size_bytes": { "type": "integer" },
                    "entries": { "type": "array", "items": { "$ref": "#/$defs/artifactManifestEntry" } }
                }
            },
            "artifactManifestEntry": {
                "type": "object",
                "additionalProperties": false,
                "required": ["path", "category", "size_bytes"],
                "properties": {
                    "path": { "type": "string" },
                    "category": { "type": "string" },
                    "size_bytes": { "type": "integer" },
                    "message_index": { "type": ["integer", "null"] },
                    "tool_index": { "type": ["integer", "null"] }
                }
            },
            "turnDigest": {
                "type": "object",
                "additionalProperties": false,
                "required": ["turn_index", "user_message_index", "user_intent", "turn_cost_tier", "turn_effectiveness", "recommended_attention", "outcome"],
                "properties": {
                    "turn_index": { "type": "integer" },
                    "user_message_index": { "type": "integer" },
                    "user_intent": { "type": "string", "enum": ["task", "continuation", "redirect", "followup-request", "scope-change", "approval"] },
                    "turn_cost_tier": { "type": "string", "enum": ["light", "medium", "heavy", "extreme"] },
                    "turn_effectiveness": { "type": "string", "enum": ["high-value", "moderate", "low-value", "waste"] },
                    "recommended_attention": { "type": "string", "enum": ["skip", "skim", "read-carefully", "inspect-artifacts"] },
                    "outcome": { "type": "string", "enum": ["answered", "executed", "delegated", "redirected", "followup-needed"] }
                }
            },
            "messageDigest": {
                "type": "object",
                "additionalProperties": false,
                "required": ["message_index", "role", "message_kind", "time_ms"],
                "properties": {
                    "message_index": { "type": "integer" },
                    "role": { "type": "string", "enum": ["user", "assistant"] },
                    "message_kind": { "type": "string", "enum": ["user", "assistant-text", "assistant-tool-only", "assistant-mixed", "assistant-reasoning-only"] },
                    "time_ms": { "type": "integer" },
                    "text_preview": { "type": ["string", "null"] },
                    "text_file": { "type": ["string", "null"] }
                }
            },
            "toolCallDigest": {
                "type": "object",
                "additionalProperties": false,
                "required": ["message_index", "tool_index", "tool"],
                "properties": {
                    "message_index": { "type": "integer" },
                    "turn_index": { "type": ["integer", "null"] },
                    "tool_index": { "type": "integer" },
                    "tool": { "type": "string" },
                    "status": { "type": "string" },
                    "call_purpose": { "type": ["string", "null"], "enum": [null, "context-gather", "search", "verify-change", "run-test", "build", "run-command", "modify", "delegate"] },
                    "patch_file": { "type": ["string", "null"] },
                    "output_file": { "type": ["string", "null"] },
                    "error_file": { "type": ["string", "null"] }
                }
            },
            "sessionRuntime": {
                "type": "object",
                "additionalProperties": false,
                "properties": {
                    "models": { "type": "array", "items": { "type": "string" } },
                    "providers": { "type": "array", "items": { "type": "string" } }
                }
            },
            "sessionFileMeta": {
                "type": "object",
                "additionalProperties": false,
                "required": ["session_path", "session_id", "title", "created_ms", "updated_ms", "duration_ms"],
                "properties": {
                    "session_path": { "type": "string" },
                    "session_id": { "type": "string" },
                    "title": { "type": "string" },
                    "agent": { "type": ["string", "null"] },
                    "created_ms": { "type": "integer" },
                    "updated_ms": { "type": "integer" },
                    "duration_ms": { "type": "integer" }
                }
            },
            "sessionTotals": {
                "type": "object",
                "additionalProperties": false,
                "required": ["turn_count", "message_count", "user_message_count", "assistant_message_count", "text_chars", "reasoning_chars", "tool_calls", "input_tokens", "output_tokens", "reasoning_tokens"],
                "properties": {
                    "turn_count": { "type": "integer" },
                    "message_count": { "type": "integer" },
                    "user_message_count": { "type": "integer" },
                    "assistant_message_count": { "type": "integer" },
                    "child_session_count": { "type": "integer" },
                    "text_chars": { "type": "integer" },
                    "reasoning_chars": { "type": "integer" },
                    "tool_calls": { "type": "integer" },
                    "input_tokens": { "type": "integer" },
                    "output_tokens": { "type": "integer" },
                    "reasoning_tokens": { "type": "integer" },
                    "cache_read_tokens": { "type": "integer" },
                    "cache_write_tokens": { "type": "integer" },
                    "cost": { "type": "number" }
                }
            },
            "toolAggregate": {
                "type": "object",
                "additionalProperties": false,
                "required": ["tool", "calls", "total_duration_ms", "max_duration_ms"],
                "properties": {
                    "tool": { "type": "string" },
                    "calls": { "type": "integer" },
                    "error_calls": { "type": "integer" },
                    "total_duration_ms": { "type": "integer" },
                    "max_duration_ms": { "type": "integer" },
                    "total_output_chars": { "type": "integer" },
                    "total_input_tokens_proxy": { "type": "integer" },
                    "avg_input_tokens_proxy": { "type": ["number", "null"] }
                }
            },
            "fileAccessRollupEntry": {
                "type": "object",
                "additionalProperties": false,
                "required": ["path"],
                "properties": {
                    "path": { "type": "string" },
                    "read_count": { "type": "integer" },
                    "modified_count": { "type": "integer" },
                    "total_output_chars": { "type": "integer" },
                    "turn_indexes": { "type": "array", "items": { "type": "integer" } }
                }
            },
            "errorPatternEntry": {
                "type": "object",
                "additionalProperties": false,
                "required": ["tool", "error_type", "count", "sample_message_index"],
                "properties": {
                    "tool": { "type": "string" },
                    "error_type": { "type": "string" },
                    "count": { "type": "integer" },
                    "turn_indexes": { "type": "array", "items": { "type": "integer" } },
                    "sample_message_index": { "type": "integer" },
                    "sample_error_preview": { "type": ["string", "null"] }
                }
            },
            "retryChainEntry": {
                "type": "object",
                "additionalProperties": false,
                "required": ["turn_index", "tool", "error_type", "attempts", "start_message_index", "end_message_index"],
                "properties": {
                    "turn_index": { "type": "integer" },
                    "tool": { "type": "string" },
                    "error_type": { "type": "string" },
                    "attempts": { "type": "integer" },
                    "start_message_index": { "type": "integer" },
                    "end_message_index": { "type": "integer" },
                    "recovery_strategy": { "type": ["string", "null"] },
                    "sample_error_preview": { "type": ["string", "null"] }
                }
            },
            "fileSupersessionEntry": {
                "type": "object",
                "additionalProperties": false,
                "required": ["written_in_turn", "superseded_by_turn"],
                "properties": {
                    "written_in_turn": { "type": "integer" },
                    "superseded_by_turn": { "type": "integer" }
                }
            },
            "fileTransitionEntry": {
                "type": "object",
                "additionalProperties": false,
                "required": ["path", "write_count"],
                "properties": {
                    "path": { "type": "string" },
                    "write_count": { "type": "integer" },
                    "write_turns": { "type": "array", "items": { "type": "integer" } },
                    "reread_in_turns": { "type": "array", "items": { "type": "integer" } },
                    "rewritten_in_turns": { "type": "array", "items": { "type": "integer" } },
                    "supersession_chain": { "type": "array", "items": { "$ref": "#/$defs/fileSupersessionEntry" } },
                    "survives_to_end": { "type": ["boolean", "null"] }
                }
            },
            "turnDependencyEdge": {
                "type": "object",
                "additionalProperties": false,
                "required": ["from_turn", "to_turn", "relation", "file_count"],
                "properties": {
                    "from_turn": { "type": "integer" },
                    "to_turn": { "type": "integer" },
                    "relation": { "type": "string" },
                    "file_count": { "type": "integer" },
                    "sample_paths": { "type": "array", "items": { "type": "string" } }
                }
            },
            "messageHotspot": {
                "type": "object",
                "additionalProperties": false,
                "required": ["session_path", "start_message_index", "message_count", "role"],
                "properties": {
                    "session_path": { "type": "string" },
                    "start_message_index": { "type": "integer" },
                    "end_message_index": { "type": ["integer", "null"] },
                    "turn_index": { "type": ["integer", "null"] },
                    "message_count": { "type": "integer" },
                    "role": { "type": "string" },
                    "hot_reasons": { "type": "array", "items": { "type": "string" } },
                    "duration_ms": { "type": ["integer", "null"] },
                    "total_tokens": { "type": "integer" },
                    "tool_calls": { "type": "integer" },
                    "pattern": { "type": ["string", "null"] },
                    "sample_text_preview": { "type": ["string", "null"] }
                }
            },
            "toolHotspot": {
                "type": "object",
                "additionalProperties": false,
                "required": ["session_path", "message_index", "tool_index", "tool"],
                "properties": {
                    "session_path": { "type": "string" },
                    "message_index": { "type": "integer" },
                    "tool_index": { "type": "integer" },
                    "tool": { "type": "string" },
                    "hot_reasons": { "type": "array", "items": { "type": "string" } },
                    "status": { "type": "string" },
                    "duration_ms": { "type": ["integer", "null"] },
                    "output_chars": { "type": ["integer", "null"] },
                    "output_preview": { "type": ["string", "null"] },
                    "output_file": { "type": ["string", "null"] },
                    "error_type": { "type": ["string", "null"] },
                    "error_file": { "type": ["string", "null"] }
                }
            },
            "turnHotspot": {
                "type": "object",
                "additionalProperties": false,
                "required": ["session_path", "turn_index", "user_message_index", "user_intent", "user_intent_confidence", "wall_to_next_user_ms", "agent_strategy", "outcome", "turn_cost_tier", "turn_effectiveness", "recommended_attention"],
                "properties": {
                    "session_path": { "type": "string" },
                    "turn_index": { "type": "integer" },
                    "user_message_index": { "type": "integer" },
                    "user_intent": { "type": "string" },
                    "user_intent_confidence": { "type": "number" },
                    "hot_reasons": { "type": "array", "items": { "type": "string" } },
                    "response_elapsed_ms": { "type": ["integer", "null"] },
                    "wall_to_next_user_ms": { "type": "integer" },
                    "total_tokens": { "type": "integer" },
                    "tool_calls": { "type": "integer" },
                    "error_count": { "type": "integer" },
                    "delegation_count": { "type": "integer" },
                    "cache_hit_ratio": { "type": ["number", "null"] },
                    "tokens_per_tool_call": { "type": ["number", "null"] },
                    "agent_strategy": { "type": "string" },
                    "outcome": { "type": "string" },
                    "turn_cost_tier": { "type": "string" },
                    "turn_effectiveness": { "type": "string" },
                    "recommended_attention": { "type": "string" },
                    "user_text_preview": { "type": ["string", "null"] },
                    "final_assistant_kind": { "type": ["string", "null"] },
                    "final_assistant_text_preview": { "type": ["string", "null"] }
                }
            },
            "sessionHotspot": {
                "type": "object",
                "additionalProperties": false,
                "required": ["session_path", "duration_ms", "message_count"],
                "properties": {
                    "session_path": { "type": "string" },
                    "duration_ms": { "type": "integer" },
                    "message_count": { "type": "integer" },
                    "tool_call_count": { "type": "integer" },
                    "input_tokens": { "type": "integer" },
                    "output_tokens": { "type": "integer" },
                    "reasoning_tokens": { "type": "integer" }
                }
            },
            "exportHotspots": {
                "type": "object",
                "additionalProperties": false,
                "properties": {
                    "slowest_sessions": { "type": "array", "items": { "$ref": "#/$defs/sessionHotspot" } },
                    "turns": { "type": "array", "items": { "$ref": "#/$defs/turnHotspot" } },
                    "messages": { "type": "array", "items": { "$ref": "#/$defs/messageHotspot" } },
                    "tools": { "type": "array", "items": { "$ref": "#/$defs/toolHotspot" } }
                }
            },
            "childLink": {
                "type": "object",
                "additionalProperties": false,
                "required": ["session_path", "session_id", "title", "summary_file", "duration_ms", "turn_count", "message_count"],
                "properties": {
                    "session_path": { "type": "string" },
                    "session_id": { "type": "string" },
                    "title": { "type": "string" },
                    "agent": { "type": ["string", "null"] },
                    "summary_file": { "type": "string" },
                    "duration_ms": { "type": "integer" },
                    "turn_count": { "type": "integer" },
                    "message_count": { "type": "integer" },
                    "tool_call_count": { "type": "integer" },
                    "input_tokens": { "type": "integer" },
                    "output_tokens": { "type": "integer" },
                    "reasoning_tokens": { "type": "integer" },
                    "parent_message_index": { "type": ["integer", "null"] },
                    "parent_tool_index": { "type": ["integer", "null"] },
                    "delegation_description": { "type": ["string", "null"] },
                    "delegation_prompt_preview": { "type": ["string", "null"] },
                    "delegation_prompt_preview_resolved": { "type": ["string", "null"] },
                    "delegation_prompt_export_paths": { "type": "array", "items": { "type": "string" } },
                    "resolved_current_export_paths": { "type": "array", "items": { "type": "string" } },
                    "delegation_export_reference_status": { "type": ["string", "null"] },
                    "current_export_path_hint": { "type": ["string", "null"] },
                    "delegation_input_file": { "type": ["string", "null"] },
                    "delegation_result_preview": { "type": ["string", "null"] },
                    "delegation_result_file": { "type": ["string", "null"] },
                    "child_outcome": { "type": ["string", "null"] },
                    "child_final_assistant_kind": { "type": ["string", "null"] }
                }
            },
            "exportTreeNode": {
                "type": "object",
                "additionalProperties": false,
                "required": ["session_path", "summary_file"],
                "properties": {
                    "session_path": { "type": "string" },
                    "summary_file": { "type": "string" },
                    "children": { "type": "array", "items": { "$ref": "#/$defs/exportTreeNode" } }
                }
            }
        }
    })
}

pub(crate) fn build_export_fields_catalog() -> Value {
    json!({
        "format": "opencode-sessions-fields-v1",
        "schema_version": SCHEMA_VERSION,
        "files": {
            "index.json": {
                "description": "Bundle root. Start here for totals, tree, policies, read order, and iteration metadata.",
                "fields": [
                    { "name": "schema_file", "type": "string", "description": "Path to formal JSON Schema for bundle files." },
                    { "name": "fields_file", "type": "string", "description": "Path to machine-readable field dictionary." },
                    { "name": "recommended_read_order", "type": "array<string>", "description": "Cheap-to-expensive traversal order for analyzers." },
                    { "name": "root_snapshot_completeness", "type": "enum", "description": "Whether root snapshot is final, live-running, stale-running, or partial." },
                    { "name": "delta_from_previous", "type": "object?", "description": "Sibling-export diff summary for benchmark iteration tracking." },
                    { "name": "classification_policy", "type": "object", "description": "Heuristic enum vocabularies and confidence semantics." },
                    { "name": "artifact_policy", "type": "object", "description": "Thresholds that decide inline previews versus sidecar artifacts." }
                ]
            },
            "summary.json": {
                "description": "Session-local rollups, file churn, children, and deliverables.",
                "fields": [
                    { "name": "session_narrative", "type": "string?", "description": "Short machine-targeted storyline for session." },
                    { "name": "snapshot_completeness", "type": "enum", "description": "Whether session snapshot is final, live-running, stale-running, or partial." },
                    { "name": "turns_compact_file", "type": "string?", "description": "Pointer to compact turn digest file for cheapest turn scan." },
                    { "name": "messages_compact_file", "type": "string?", "description": "Pointer to compact message digest file for cheap chronological scan." },
                    { "name": "largest_artifacts", "type": "array<object>", "description": "Largest sidecars for quick artifact triage by size and category." },
                    { "name": "artifacts_manifest_file", "type": "string?", "description": "Pointer to artifacts/index.json catalog." },
                    { "name": "session_deliverables", "type": "array<object>", "description": "Final modified files with last-write turn, patch intent, hashes, and optional snapshots." },
                    { "name": "file_transition_rollup", "type": "array<object>", "description": "Per-file rewrite history with supersession chains." },
                    { "name": "turn_dependency_edges", "type": "array<object>", "description": "Cross-turn overwrite graph edges." }
                ]
            },
            "turns.compact.jsonl": {
                "description": "Compact turn digest for cheapest bulk triage before opening full turns.jsonl.",
                "fields": [
                    { "name": "turn_index", "type": "integer", "description": "Zero-based turn id within session." },
                    { "name": "message_index_end", "type": "integer", "description": "Last message index covered by this turn span." },
                    { "name": "user_intent", "type": "enum", "description": "Heuristic task class for user request." },
                    { "name": "agent_strategy", "type": "enum", "description": "High-level phase label for turn." },
                    { "name": "turn_cost_tier", "type": "enum", "description": "Coarse spend bucket independent from value." },
                    { "name": "turn_effectiveness", "type": "enum", "description": "Durable value estimate for turn output." },
                    { "name": "recommended_attention", "type": "enum", "description": "Cheap skip/read guidance for analyzers." },
                    { "name": "failure_narrative", "type": "string?", "description": "Compact why-low-value summary for waste/low-value turns." },
                    { "name": "reasoning_summary", "type": "string?", "description": "Turn-level reasoning rollup from assistant messages in turn." },
                    { "name": "turn_change_summary", "type": "string?", "description": "Compact what-changed summary for turn." },
                    { "name": "key_diff_preview", "type": "string?", "description": "Strongest edit preview for turn." }
                ]
            },
            "turns.jsonl": {
                "description": "One user turn digest per line. Primary optimization layer.",
                "fields": [
                    { "name": "turn_index", "type": "integer", "description": "Zero-based turn id within session." },
                    { "name": "user_intent", "type": "enum", "description": "Heuristic task class for user request." },
                    { "name": "turn_cost_tier", "type": "enum", "description": "Coarse spend bucket independent from outcome quality." },
                    { "name": "turn_effectiveness", "type": "enum", "description": "Durable value estimate for turn output." },
                    { "name": "recommended_attention", "type": "enum", "description": "Cheap skip/read guidance for analyzers." },
                    { "name": "effectiveness_signals", "type": "object", "description": "Supporting metrics like retry ratio and redundant reads." },
                    { "name": "change_stats", "type": "object", "description": "Patch-count and line-delta summary across turn tool calls." },
                    { "name": "call_purpose_rollup", "type": "array<object>", "description": "Why tools were used in this turn." },
                    { "name": "optimization_hints", "type": "array<string>", "description": "Heuristic coaching strings for future optimization." }
                ]
            },
            "messages.compact.jsonl": {
                "description": "Cheap chronological message scan before opening full messages.jsonl.",
                "fields": [
                    { "name": "message_index", "type": "integer", "description": "Chronological message id in session." },
                    { "name": "turn_index", "type": "integer?", "description": "Owning turn id when message belongs to a user turn span." },
                    { "name": "message_kind", "type": "enum", "description": "User / assistant text / tool-only / mixed / reasoning-only." },
                    { "name": "total_tokens", "type": "integer?", "description": "Cheap token signal for prioritizing heavy messages." },
                    { "name": "activity_summary", "type": "string?", "description": "Compact tool/activity rollup for sparse assistant messages." },
                    { "name": "reasoning_summary", "type": "string?", "description": "Cheap reasoning rollup before opening full message digest." }
                ]
            },
            "messages.jsonl": {
                "description": "Chronological full message digests with previews, metadata, and pointers.",
                "fields": [
                    { "name": "message_index", "type": "integer", "description": "Chronological message id in session." },
                    { "name": "turn_index", "type": "integer?", "description": "Owning turn id when message belongs to a user turn span." },
                    { "name": "message_kind", "type": "enum", "description": "User / assistant text / tool-only / mixed / reasoning-only." },
                    { "name": "text_preview", "type": "string?", "description": "Inline text preview before opening sidecar." },
                    { "name": "reasoning_summary", "type": "string?", "description": "Cheap middle layer for long reasoning content." },
                    { "name": "text_file", "type": "string?", "description": "Sidecar pointer when full text lives outside digest." }
                ]
            },
            "tool_calls.jsonl": {
                "description": "One tool invocation per line with direct turn linkage.",
                "fields": [
                    { "name": "message_index", "type": "integer", "description": "Hosting message id." },
                    { "name": "turn_index", "type": "integer?", "description": "Direct join back to turns.jsonl." },
                    { "name": "tool_index", "type": "integer", "description": "Tool ordinal within message." },
                    { "name": "call_purpose", "type": "enum?", "description": "Heuristic why-label for tool usage." },
                    { "name": "patch_summary", "type": "object?", "description": "Structured edit scope for apply_patch." },
                    { "name": "patch_file", "type": "string?", "description": "Full diff sidecar when patch text is large." },
                    { "name": "output_file", "type": "string?", "description": "Large tool output sidecar pointer." }
                ]
            },
            "artifacts/index.json": {
                "description": "Catalog of sidecar files with size, category, and message/tool linkage.",
                "fields": [
                    { "name": "total_size_bytes", "type": "integer", "description": "Total bytes across all listed artifacts." },
                    { "name": "category", "type": "string", "description": "Artifact class such as message-reasoning or tool-output." },
                    { "name": "size_bytes", "type": "integer", "description": "Artifact size for triage and largest-artifact ranking." },
                    { "name": "message_index", "type": "integer?", "description": "Owning message when artifact came from message or tool under message." },
                    { "name": "tool_index", "type": "integer?", "description": "Owning tool ordinal for tool artifacts." }
                ]
            },
            "deliverables/*": {
                "description": "Embedded final snapshots copied from current workspace when file still exists locally.",
                "fields": [
                    { "name": "content_sha256", "type": "string", "description": "Hash for cheap comparison without opening snapshot file." },
                    { "name": "line_count", "type": "integer", "description": "Final line count for deliverable snapshot." },
                    { "name": "snapshot_source", "type": "string", "description": "Origin of embedded snapshot bytes." }
                ]
            }
        }
    })
}

pub(crate) fn render_export_readme(index: &ExportIndexFile) -> String {
    let mut out = String::new();
    out.push_str("# OpenCode Session Export\n\n");
    out.push_str("LLM-first bundle. Read files in order below. Do not ingest whole `artifacts/` tree unless needed.\n\n");
    out.push_str("## Read order\n\n");
    for path in &index.recommended_read_order {
        out.push_str(&format!("1. `{path}`\n"));
    }
    out.push_str("\n## How to use\n\n");
    out.push_str("- Start with `index.json` for totals, tree, hot spots, session pointers.\n");
    out.push_str("- Read `schema.json` for machine-validatable bundle contract before writing strict parsers.\n");
    out.push_str("- Read `fields.json` for machine-readable field descriptions when mapping bundle fields into downstream pipelines.\n");
    out.push_str("- Read root `summary.json` next for session-local totals, runtime, narrative, pivotal turns, tool rollup, child links.\n");
    out.push_str("- Read `turns.compact.jsonl` for cheapest turn-level optimization scan, then `turns.jsonl` when compact layer is not enough.\n");
    out.push_str("- Read `messages.compact.jsonl` for cheap chronological message scan, then `messages.jsonl` when compact layer is not enough.\n");
    out.push_str("- Read `tool_calls.jsonl` for tool chronology and heavy I/O pointers.\n");
    out.push_str("- Use `classification_policy` in `index.json` to interpret heuristic tags and confidence.\n");
    out.push_str("- Open files in `artifacts/` only when digest has `*_file` pointer and preview is not enough; prefer `artifacts/index.json` when browsing sidecars.\n");
    out.push_str("- Follow child `summary.json` links for subagents.\n");
    out.push_str("\n## Semantics\n\n");
    out.push_str("- `time_ms` = message creation time.\n");
    out.push_str("- `wall_gap_ms` = time until next message in same session chronology.\n");
    out.push_str("- `turns.compact.jsonl` = compact turn scan layer with key cost/value/change/reasoning fields only.\n");
    out.push_str("- `message_index_end` on compact turns gives direct turn span end without opening full turns file.\n");
    out.push_str("- `turns.jsonl` = full user turn digest with response cost, tools, delegations, and next-user outcome signal.\n");
    out.push_str("- `messages.compact.jsonl` = compact chronological message scan with timing, token, activity, and reasoning hints only.\n");
    out.push_str("- `user_message_index`..`message_index_end` spans full message range for each turn.\n");
    out.push_str("- `user_intent_confidence` = 0..1 heuristic confidence for `user_intent`; `alternative_user_intents` gives fallback labels when confidence is low.\n");
    out.push_str("- `user_tags` values are documented in `classification_policy.user_tag_values`.\n");
    out.push_str("- `message_kind` tells whether message is user, assistant-text, tool-only, mixed, or reasoning-only.\n");
    out.push_str("- `duration_ms` = elapsed model/tool duration when present.\n");
    out.push_str("- `runtime.models` / `runtime.providers` = session-level runtime set; per-message `model` / `provider` appear only when session mixes runtimes.\n");
    out.push_str("- `session_status` = `completed` | `running` | `abandoned` | `error`.\n");
    out.push_str("- `root_snapshot_completeness` / `snapshot_completeness` distinguish final snapshots from live or stale running captures.\n");
    out.push_str("- `agent_strategy` on turns = heuristic phase label (`explore`, `implement`, `debug`, `refactor`, `validate`, `delegate`).\n");
    out.push_str("- `turn_cost_tier` separates cost (`light` | `medium` | `heavy` | `extreme`) from value labels; `recommended_attention` is coarse skip/read guidance.\n");
    out.push_str("- `turn_effectiveness` / `effectiveness_signals` estimate whether turn produced durable value versus churn.\n");
    out.push_str("- `text_preview` / `reasoning_preview` / `output_preview` are truncated previews.\n");
    out.push_str("- `activity_summary` = compact summary for tool-only or sparse assistant messages.\n");
    out.push_str("- `reasoning_summary` = compact middle layer before opening full reasoning sidecar; `reasoning_themes` give cheap topic hints.\n");
    out.push_str("- `read_files` / `modified_files` on turns are unique path samples; counts show full scope.\n");
    out.push_str("- `change_stats` on turns aggregates patch counts and line churn across tool calls in that turn; `change_intents` summarizes semantic edit classes seen in that turn.\n");
    out.push_str("- `call_purpose_rollup` on turns summarizes why tools were used (`modify`, `search`, `context-gather`, ...).\n");
    out.push_str("- `key_diff_preview` on turns is compact patch summary for strongest edit in that turn.\n");
    out.push_str("- `file_transition_rollup` in `summary.json` is aggregated per file; `write_turns` show rewrite history, `supersession_chain` links which turn overwrote which, and `survives_to_end` is omitted while session is still running.\n");
    out.push_str("- `session_deliverables` in `summary.json` highlights final modified files with last-write turn, coarse patch intent, and optional final snapshot metadata.\n");
    out.push_str("- `turn_dependency_edges` in `summary.json` aggregates cross-turn rewrite edges, so analyzers can jump from overwritten turn to overwriting turn.\n");
    out.push_str("- `session_narrative` / `pivotal_turns` in `summary.json` give fast storyline and strategic jump points.\n");
    out.push_str("- `last_activity_ms` / `staleness_ms` in summaries and root index help distinguish active `running` sessions from stale ones.\n");
    out.push_str("- `hot_messages` may collapse consecutive same-signal messages into spans via `start_message_index` / `end_message_index`.\n");
    out.push_str("- `file_access_rollup` / `error_patterns` in `summary.json` surface repeated file churn and failures.\n");
    out.push_str("- `retry_chains` in `summary.json` groups repeated same-tool failures inside turns.\n");
    out.push_str("- `token_efficiency` gives normalized per-turn/per-tool token ratios.\n");
    out.push_str("- `tool_rollup.total_input_tokens_proxy` / `avg_input_tokens_proxy` are per-tool token approximations split evenly across tool parts inside same hosting message.\n");
    out.push_str("- `iteration_meta` in `index.json` links sibling benchmark exports in same series.\n");
    out.push_str("- `delta_from_previous` summarizes high-level index changes plus turn/tool/totals deltas versus prior sibling export when present.\n");
    out.push_str("- `children[]` may include `delegation_result_preview` / `child_outcome` for subagent result triage from parent summary.\n");
    out.push_str("- `children[].delegation_export_reference_status` warns when child prompt references stale export paths; `current_export_path_hint` points at current bundle series root, `resolved_current_export_paths` rewrites stale bundle paths onto current export root when possible, and `delegation_prompt_preview_resolved` gives compact rewritten prompt preview when stale root was detected.\n");
    out.push_str("- `patch_summary` / `patch_intent` / `patch_file` on `apply_patch` calls expose structured edit scope, coarse edit class, and full diff text; `input_file` may be omitted when redundant with `patchText`.\n");
    out.push_str("- `call_purpose` on tool calls gives coarse per-call intent (`context-gather`, `search`, `build`, ...).\n");
    out.push_str("- `turn_index` on tool calls gives direct join back to `turns.jsonl` without recomputing message spans.\n");
    out.push_str("- `retry_chains[].recovery_strategy` summarizes what agent tried after repeated same-tool failures.\n");
    out.push_str("- `optimization_hints` / `failure_narrative` on turns are heuristic machine-targeted coaching signals.\n");
    out.push_str("- `error_type` on tool calls is heuristic (`aborted`, `not-found`, `timeout`, `permission`, ...).\n");
    out.push_str("- `artifacts_manifest_file` in `summary.json` points at `artifacts/index.json`, which catalogs sidecars with category, size, and message/tool linkage.\n");
    out.push_str("- `largest_artifacts` in `summary.json` surfaces biggest sidecars first so analyzers can target expensive files without scanning whole manifest.\n");
    out.push_str("- `snapshot_file` / `content_sha256` / `line_count` on `session_deliverables` describe current workspace snapshot when source file is still available locally.\n");
    out.push_str("- `*_file` pointers contain full text/json sidecars when preview omitted or insufficient.\n");
    out.push_str("- `artifact_policy` in `index.json` tells which thresholds decide sidecar emission.\n");
    out.push_str("- Missing field means absent or intentionally omitted as redundant.\n");
    out.push_str("\n## Root summary\n\n");
    out.push_str(&format!("- schema_version: `{}`\n", index.schema_version));
    out.push_str(&format!("- schema_file: `{}`\n", index.schema_file));
    out.push_str(&format!("- fields_file: `{}`\n", index.fields_file));
    out.push_str(&format!("- export_id: `{}`\n", index.export_id));
    out.push_str(&format!("- export_timestamp_ms: `{}`\n", index.export_timestamp_ms));
    out.push_str(&format!("- root_session_id: `{}`\n", index.root_session_id));
    out.push_str(&format!("- root_title: `{}`\n", index.root_title));
    out.push_str(&format!("- root_session_status: `{}`\n", index.root_session_status));
    out.push_str(&format!("- root_snapshot_completeness: `{}`\n", index.root_snapshot_completeness));
    out.push_str(&format!("- root_last_activity_ms: `{}`\n", index.root_last_activity_ms));
    if index.root_staleness_ms > 0 {
        out.push_str(&format!("- root_staleness_ms: `{}`\n", index.root_staleness_ms));
    }
    out.push_str(&format!("- session_count: {}\n", index.totals.session_count));
    out.push_str(&format!("- turn_count: {}\n", index.totals.turn_count));
    out.push_str(&format!("- message_count: {}\n", index.totals.message_count));
    out.push_str(&format!("- tool_calls: {}\n", index.totals.tool_calls));
    if let Some(path) = &index.root_task_file {
        out.push_str(&format!("- root_task_file: `{path}`\n"));
    }
    out
}
