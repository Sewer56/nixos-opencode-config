use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

use crate::format::*;

#[derive(Debug, Clone, Serialize)]
pub(crate) struct TokenStatsExport {
    #[serde(skip_serializing)]
    pub(crate) total: Option<u64>,
    #[serde(skip_serializing_if = "is_zero_u64")]
    pub(crate) input: u64,
    #[serde(skip_serializing_if = "is_zero_u64")]
    pub(crate) output: u64,
    #[serde(skip_serializing_if = "is_zero_u64")]
    pub(crate) reasoning: u64,
    #[serde(skip_serializing_if = "is_zero_u64")]
    pub(crate) cache_read: u64,
    #[serde(skip_serializing_if = "is_zero_u64")]
    pub(crate) cache_write: u64,
}

impl TokenStatsExport {
    pub(crate) fn is_empty(&self) -> bool {
        self.total.unwrap_or_default() == 0
            && self.input == 0
            && self.output == 0
            && self.reasoning == 0
            && self.cache_read == 0
            && self.cache_write == 0
    }
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct SessionFileMeta {
    pub(crate) session_path: String,
    pub(crate) session_id: String,
    pub(crate) title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) agent: Option<String>,
    pub(crate) created_ms: i64,
    pub(crate) updated_ms: i64,
    pub(crate) duration_ms: i64,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ChildLink {
    pub(crate) session_path: String,
    pub(crate) session_id: String,
    pub(crate) title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) agent: Option<String>,
    pub(crate) summary_file: String,
    pub(crate) duration_ms: i64,
    pub(crate) turn_count: usize,
    pub(crate) message_count: usize,
    #[serde(skip_serializing_if = "is_zero_usize")]
    pub(crate) tool_call_count: usize,
    #[serde(skip_serializing_if = "is_zero_u64")]
    pub(crate) input_tokens: u64,
    #[serde(skip_serializing_if = "is_zero_u64")]
    pub(crate) output_tokens: u64,
    #[serde(skip_serializing_if = "is_zero_u64")]
    pub(crate) reasoning_tokens: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) parent_message_index: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) parent_tool_index: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) delegation_description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) delegation_prompt_preview: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) delegation_prompt_preview_resolved: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub(crate) delegation_prompt_export_paths: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub(crate) resolved_current_export_paths: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) delegation_export_reference_status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) current_export_path_hint: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) delegation_input_file: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) delegation_result_preview: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) delegation_result_file: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) child_outcome: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) child_final_assistant_kind: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ExportTreeNode {
    pub(crate) session_path: String,
    pub(crate) summary_file: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub(crate) children: Vec<ExportTreeNode>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct SessionStats {
    pub(crate) session_path: String,
    pub(crate) depth: usize,
    pub(crate) session_id: String,
    pub(crate) parent_session_id: Option<String>,
    pub(crate) title: String,
    pub(crate) agent: Option<String>,
    pub(crate) created_ms: i64,
    pub(crate) updated_ms: i64,
    pub(crate) duration_ms: i64,
    pub(crate) turn_count: usize,
    pub(crate) message_count: usize,
    pub(crate) user_message_count: usize,
    pub(crate) assistant_message_count: usize,
    pub(crate) child_session_count: usize,
    pub(crate) text_chars: usize,
    pub(crate) reasoning_chars: usize,
    pub(crate) tool_calls: usize,
    pub(crate) input_tokens: u64,
    pub(crate) output_tokens: u64,
    pub(crate) reasoning_tokens: u64,
    pub(crate) cache_read_tokens: u64,
    pub(crate) cache_write_tokens: u64,
    pub(crate) cost: f64,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ExportTotals {
    pub(crate) session_count: usize,
    pub(crate) turn_count: usize,
    pub(crate) message_count: usize,
    pub(crate) user_message_count: usize,
    pub(crate) assistant_message_count: usize,
    pub(crate) text_chars: usize,
    pub(crate) reasoning_chars: usize,
    pub(crate) tool_calls: usize,
    pub(crate) input_tokens: u64,
    pub(crate) output_tokens: u64,
    pub(crate) reasoning_tokens: u64,
    #[serde(skip_serializing_if = "is_zero_u64")]
    pub(crate) cache_read_tokens: u64,
    #[serde(skip_serializing_if = "is_zero_u64")]
    pub(crate) cache_write_tokens: u64,
    #[serde(skip_serializing_if = "is_zero_f64")]
    pub(crate) cost: f64,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct SessionTotals {
    pub(crate) turn_count: usize,
    pub(crate) message_count: usize,
    pub(crate) user_message_count: usize,
    pub(crate) assistant_message_count: usize,
    #[serde(skip_serializing_if = "is_zero_usize")]
    pub(crate) child_session_count: usize,
    pub(crate) text_chars: usize,
    pub(crate) reasoning_chars: usize,
    pub(crate) tool_calls: usize,
    pub(crate) input_tokens: u64,
    pub(crate) output_tokens: u64,
    pub(crate) reasoning_tokens: u64,
    #[serde(skip_serializing_if = "is_zero_u64")]
    pub(crate) cache_read_tokens: u64,
    #[serde(skip_serializing_if = "is_zero_u64")]
    pub(crate) cache_write_tokens: u64,
    #[serde(skip_serializing_if = "is_zero_f64")]
    pub(crate) cost: f64,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct SessionRuntime {
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub(crate) models: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub(crate) providers: Vec<String>,
}

impl SessionRuntime {
    pub(crate) fn is_empty(&self) -> bool {
        self.models.is_empty() && self.providers.is_empty()
    }
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct MessageDigest {
    #[serde(skip_serializing)]
    pub(crate) session_path: String,
    pub(crate) message_index: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) turn_index: Option<usize>,
    pub(crate) role: String,
    pub(crate) message_kind: String,
    pub(crate) time_ms: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) provider: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) wall_gap_ms: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) duration_ms: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) tokens: Option<TokenStatsExport>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) user_intent: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) user_intent_confidence: Option<f64>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub(crate) user_tags: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub(crate) alternative_user_intents: Vec<String>,
    #[serde(skip_serializing_if = "is_zero_usize")]
    pub(crate) text_chars: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) text_preview: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) text_file: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) activity_summary: Option<String>,
    #[serde(skip_serializing_if = "is_zero_usize")]
    pub(crate) reasoning_chars: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) reasoning_summary: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub(crate) reasoning_themes: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) reasoning_preview: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) reasoning_file: Option<String>,
    #[serde(skip_serializing_if = "is_zero_usize")]
    pub(crate) tool_count: usize,
    #[serde(skip_serializing_if = "is_zero_usize")]
    pub(crate) tool_error_count: usize,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub(crate) tool_names: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub(crate) subagent_calls: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub(crate) subtask_agents: Vec<String>,
    #[serde(skip_serializing_if = "is_zero_usize")]
    pub(crate) patch_file_count: usize,
    #[serde(skip_serializing_if = "is_zero_usize")]
    pub(crate) file_attachment_count: usize,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ToolCallDigest {
    #[serde(skip_serializing)]
    pub(crate) session_path: String,
    pub(crate) message_index: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) turn_index: Option<usize>,
    pub(crate) tool_index: usize,
    pub(crate) tool: String,
    #[serde(skip_serializing_if = "is_completed_status")]
    pub(crate) status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) duration_ms: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) input_preview: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) input_file: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) task_description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) task_prompt_preview: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) delegated_session_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) call_purpose: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) patch_summary: Option<PatchSummary>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) patch_intent: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) patch_file: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) output_preview: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) output_chars: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) output_file: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) error_preview: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) error_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) error_file: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub(crate) read_paths: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub(crate) modified_paths: Vec<String>,
    #[serde(skip_serializing)]
    pub(crate) modified_path_presence: HashMap<String, bool>,
    #[serde(skip_serializing)]
    pub(crate) input_tokens_proxy: u64,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ToolAggregate {
    pub(crate) tool: String,
    pub(crate) calls: usize,
    #[serde(skip_serializing_if = "is_zero_usize")]
    pub(crate) error_calls: usize,
    pub(crate) total_duration_ms: i64,
    pub(crate) max_duration_ms: i64,
    #[serde(skip_serializing_if = "is_zero_usize")]
    pub(crate) total_output_chars: usize,
    #[serde(skip_serializing_if = "is_zero_u64")]
    pub(crate) total_input_tokens_proxy: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) avg_input_tokens_proxy: Option<f64>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct FileAccessRollupEntry {
    pub(crate) path: String,
    #[serde(skip_serializing_if = "is_zero_usize")]
    pub(crate) read_count: usize,
    #[serde(skip_serializing_if = "is_zero_usize")]
    pub(crate) modified_count: usize,
    #[serde(skip_serializing_if = "is_zero_usize")]
    pub(crate) total_output_chars: usize,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub(crate) turn_indexes: Vec<usize>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ErrorPatternEntry {
    pub(crate) tool: String,
    pub(crate) error_type: String,
    pub(crate) count: usize,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub(crate) turn_indexes: Vec<usize>,
    pub(crate) sample_message_index: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) sample_error_preview: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct RetryChainEntry {
    pub(crate) turn_index: usize,
    pub(crate) tool: String,
    pub(crate) error_type: String,
    pub(crate) attempts: usize,
    pub(crate) start_message_index: usize,
    pub(crate) end_message_index: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) recovery_strategy: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) sample_error_preview: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct FileSupersessionEntry {
    pub(crate) written_in_turn: usize,
    pub(crate) superseded_by_turn: usize,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct FileTransitionEntry {
    pub(crate) path: String,
    pub(crate) write_count: usize,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub(crate) write_turns: Vec<usize>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub(crate) reread_in_turns: Vec<usize>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub(crate) rewritten_in_turns: Vec<usize>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub(crate) supersession_chain: Vec<FileSupersessionEntry>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) survives_to_end: Option<bool>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct SessionDeliverableEntry {
    pub(crate) path: String,
    pub(crate) write_count: usize,
    pub(crate) final_turn_index: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) final_patch_intent: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) snapshot_file: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) content_sha256: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) line_count: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) snapshot_source: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ArtifactManifestEntry {
    pub(crate) path: String,
    pub(crate) category: String,
    pub(crate) size_bytes: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) message_index: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) tool_index: Option<usize>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ArtifactManifestFile {
    pub(crate) artifacts_dir: String,
    pub(crate) total_size_bytes: u64,
    pub(crate) entries: Vec<ArtifactManifestEntry>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct TurnCompactEntry {
    pub(crate) turn_index: usize,
    pub(crate) user_message_index: usize,
    pub(crate) message_index_end: usize,
    pub(crate) user_intent: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) response_elapsed_ms: Option<i64>,
    #[serde(skip_serializing_if = "is_zero_u64")]
    pub(crate) total_tokens: u64,
    #[serde(skip_serializing_if = "is_zero_usize")]
    pub(crate) tool_call_count: usize,
    #[serde(skip_serializing_if = "is_zero_usize")]
    pub(crate) modified_file_count: usize,
    pub(crate) agent_strategy: String,
    pub(crate) outcome: String,
    pub(crate) success: bool,
    pub(crate) turn_cost_tier: String,
    pub(crate) turn_effectiveness: String,
    pub(crate) recommended_attention: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub(crate) optimization_hints: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) failure_narrative: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) reasoning_summary: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) turn_change_summary: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) key_diff_preview: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct MessageCompactEntry {
    pub(crate) message_index: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) turn_index: Option<usize>,
    pub(crate) role: String,
    pub(crate) message_kind: String,
    pub(crate) time_ms: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) wall_gap_ms: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) duration_ms: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) total_tokens: Option<u64>,
    #[serde(skip_serializing_if = "is_zero_usize")]
    pub(crate) tool_count: usize,
    #[serde(skip_serializing_if = "is_zero_usize")]
    pub(crate) tool_error_count: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) text_preview: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) activity_summary: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) reasoning_summary: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct TurnDependencyEdge {
    pub(crate) from_turn: usize,
    pub(crate) to_turn: usize,
    pub(crate) relation: String,
    pub(crate) file_count: usize,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub(crate) sample_paths: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct TokenEfficiency {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) cache_hit_ratio: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) avg_input_tokens_per_turn: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) avg_output_tokens_per_turn: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) avg_reasoning_tokens_per_turn: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) avg_tool_calls_per_turn: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) avg_input_tokens_per_tool_call: Option<f64>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct PatchSummary {
    #[serde(skip_serializing_if = "is_zero_usize")]
    pub(crate) files_added: usize,
    #[serde(skip_serializing_if = "is_zero_usize")]
    pub(crate) files_updated: usize,
    #[serde(skip_serializing_if = "is_zero_usize")]
    pub(crate) files_deleted: usize,
    #[serde(skip_serializing_if = "is_zero_usize")]
    pub(crate) files_moved: usize,
    #[serde(skip_serializing_if = "is_zero_usize")]
    pub(crate) hunks: usize,
    #[serde(skip_serializing_if = "is_zero_usize")]
    pub(crate) lines_added: usize,
    #[serde(skip_serializing_if = "is_zero_usize")]
    pub(crate) lines_deleted: usize,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub(crate) sample_paths: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct TurnToolAggregate {
    pub(crate) tool: String,
    pub(crate) calls: usize,
    #[serde(skip_serializing_if = "is_zero_usize")]
    pub(crate) error_calls: usize,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct TurnPurposeAggregate {
    pub(crate) purpose: String,
    pub(crate) calls: usize,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct TurnDelegationPreview {
    pub(crate) session_path: String,
    pub(crate) session_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) agent: Option<String>,
    pub(crate) parent_message_index: usize,
    pub(crate) parent_tool_index: usize,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct TurnDigest {
    #[serde(skip_serializing)]
    pub(crate) session_path: String,
    pub(crate) turn_index: usize,
    pub(crate) user_message_index: usize,
    pub(crate) message_index_end: usize,
    pub(crate) time_ms: i64,
    pub(crate) user_intent: String,
    pub(crate) user_intent_confidence: f64,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub(crate) user_tags: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub(crate) alternative_user_intents: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) user_text_preview: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) user_text_file: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) assistant_message_start: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) assistant_message_end: Option<usize>,
    #[serde(skip_serializing_if = "is_zero_usize")]
    pub(crate) assistant_message_count: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) response_elapsed_ms: Option<i64>,
    pub(crate) wall_to_next_user_ms: i64,
    #[serde(skip_serializing_if = "is_zero_i64")]
    pub(crate) assistant_duration_ms: i64,
    #[serde(skip_serializing_if = "is_zero_i64")]
    pub(crate) tool_duration_ms: i64,
    #[serde(skip_serializing_if = "is_zero_usize")]
    pub(crate) tool_call_count: usize,
    #[serde(skip_serializing_if = "is_zero_usize")]
    pub(crate) error_count: usize,
    #[serde(skip_serializing_if = "is_zero_usize")]
    pub(crate) delegation_count: usize,
    #[serde(skip_serializing_if = "is_zero_u64")]
    pub(crate) input_tokens: u64,
    #[serde(skip_serializing_if = "is_zero_u64")]
    pub(crate) output_tokens: u64,
    #[serde(skip_serializing_if = "is_zero_u64")]
    pub(crate) reasoning_tokens: u64,
    #[serde(skip_serializing_if = "is_zero_u64")]
    pub(crate) cache_read_tokens: u64,
    #[serde(skip_serializing_if = "is_zero_u64")]
    pub(crate) cache_write_tokens: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) cache_hit_ratio: Option<f64>,
    #[serde(skip_serializing_if = "is_zero_u64")]
    pub(crate) total_tokens: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) tokens_per_tool_call: Option<f64>,
    #[serde(skip_serializing_if = "is_zero_usize")]
    pub(crate) read_file_count: usize,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub(crate) read_files: Vec<String>,
    #[serde(skip_serializing_if = "is_zero_usize")]
    pub(crate) modified_file_count: usize,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub(crate) modified_files: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub(crate) tool_rollup: Vec<TurnToolAggregate>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub(crate) call_purpose_rollup: Vec<TurnPurposeAggregate>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub(crate) delegations: Vec<TurnDelegationPreview>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) delegations_file: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) final_assistant_message_index: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) final_assistant_text_preview: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) final_assistant_text_file: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) final_assistant_kind: Option<String>,
    pub(crate) agent_strategy: String,
    pub(crate) outcome: String,
    pub(crate) success: bool,
    pub(crate) turn_cost_tier: String,
    pub(crate) turn_effectiveness: String,
    pub(crate) recommended_attention: String,
    #[serde(skip_serializing_if = "TurnEffectivenessSignals::is_empty")]
    pub(crate) effectiveness_signals: TurnEffectivenessSignals,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) failure_narrative: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub(crate) optimization_hints: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) reasoning_summary: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub(crate) reasoning_themes: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) turn_change_summary: Option<String>,
    #[serde(skip_serializing_if = "TurnChangeStats::is_empty")]
    pub(crate) change_stats: TurnChangeStats,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub(crate) change_intents: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) key_diff_preview: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) next_user_message_index: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) next_user_intent: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) next_user_intent_confidence: Option<f64>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub(crate) next_user_tags: Vec<String>,
    #[serde(skip_serializing)]
    pub(crate) read_files_all: Vec<String>,
    #[serde(skip_serializing)]
    pub(crate) modified_files_all: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct TurnEffectivenessSignals {
    #[serde(skip_serializing_if = "is_zero_usize")]
    pub(crate) files_modified_count: usize,
    pub(crate) files_survived_to_end: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) retry_ratio: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) redundant_read_ratio: Option<f64>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct TurnChangeStats {
    #[serde(skip_serializing_if = "is_zero_usize")]
    pub(crate) patch_calls: usize,
    #[serde(skip_serializing_if = "is_zero_usize")]
    pub(crate) files_added: usize,
    #[serde(skip_serializing_if = "is_zero_usize")]
    pub(crate) files_updated: usize,
    #[serde(skip_serializing_if = "is_zero_usize")]
    pub(crate) files_deleted: usize,
    #[serde(skip_serializing_if = "is_zero_usize")]
    pub(crate) files_moved: usize,
    #[serde(skip_serializing_if = "is_zero_usize")]
    pub(crate) lines_added: usize,
    #[serde(skip_serializing_if = "is_zero_usize")]
    pub(crate) lines_deleted: usize,
}

impl TurnChangeStats {
    pub(crate) fn is_empty(&self) -> bool {
        self.patch_calls == 0
            && self.files_added == 0
            && self.files_updated == 0
            && self.files_deleted == 0
            && self.files_moved == 0
            && self.lines_added == 0
            && self.lines_deleted == 0
    }
}

impl TurnEffectivenessSignals {
    pub(crate) fn is_empty(&self) -> bool {
        self.files_modified_count == 0
            && self.files_survived_to_end == 0
            && self.retry_ratio.is_none()
            && self.redundant_read_ratio.is_none()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct TurnDeltaDigest {
    pub(crate) turn_index: usize,
    #[serde(default)]
    pub(crate) agent_strategy: Option<String>,
    #[serde(default)]
    pub(crate) turn_cost_tier: Option<String>,
    #[serde(default)]
    pub(crate) turn_effectiveness: Option<String>,
    #[serde(default)]
    pub(crate) input_tokens: u64,
    #[serde(default)]
    pub(crate) output_tokens: u64,
    #[serde(default)]
    pub(crate) reasoning_tokens: u64,
    #[serde(default)]
    pub(crate) cache_read_tokens: u64,
    #[serde(default)]
    pub(crate) cache_write_tokens: u64,
    #[serde(default)]
    pub(crate) tool_call_count: usize,
    #[serde(default)]
    pub(crate) error_count: usize,
    #[serde(default)]
    pub(crate) modified_file_count: usize,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct TurnDeltaEntry {
    pub(crate) turn_index: usize,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub(crate) changed_fields: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) previous_agent_strategy: Option<String>,
    pub(crate) current_agent_strategy: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) previous_turn_cost_tier: Option<String>,
    pub(crate) current_turn_cost_tier: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) previous_turn_effectiveness: Option<String>,
    pub(crate) current_turn_effectiveness: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) previous_total_tokens: Option<u64>,
    pub(crate) current_total_tokens: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) previous_tool_call_count: Option<usize>,
    pub(crate) current_tool_call_count: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) previous_error_count: Option<usize>,
    pub(crate) current_error_count: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) previous_modified_file_count: Option<usize>,
    pub(crate) current_modified_file_count: usize,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct MessageHotspot {
    pub(crate) session_path: String,
    pub(crate) start_message_index: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) end_message_index: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) turn_index: Option<usize>,
    #[serde(skip_serializing_if = "is_one_usize")]
    pub(crate) message_count: usize,
    pub(crate) role: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub(crate) hot_reasons: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) duration_ms: Option<i64>,
    #[serde(skip_serializing_if = "is_zero_u64")]
    pub(crate) total_tokens: u64,
    #[serde(skip_serializing_if = "is_zero_usize")]
    pub(crate) tool_calls: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) pattern: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) sample_text_preview: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ToolHotspot {
    pub(crate) session_path: String,
    pub(crate) message_index: usize,
    pub(crate) tool_index: usize,
    pub(crate) tool: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub(crate) hot_reasons: Vec<String>,
    #[serde(skip_serializing_if = "is_completed_status")]
    pub(crate) status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) duration_ms: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) output_chars: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) output_preview: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) output_file: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) error_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) error_file: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct TurnHotspot {
    pub(crate) session_path: String,
    pub(crate) turn_index: usize,
    pub(crate) user_message_index: usize,
    pub(crate) user_intent: String,
    pub(crate) user_intent_confidence: f64,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub(crate) hot_reasons: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) response_elapsed_ms: Option<i64>,
    pub(crate) wall_to_next_user_ms: i64,
    #[serde(skip_serializing_if = "is_zero_u64")]
    pub(crate) total_tokens: u64,
    #[serde(skip_serializing_if = "is_zero_usize")]
    pub(crate) tool_calls: usize,
    #[serde(skip_serializing_if = "is_zero_usize")]
    pub(crate) error_count: usize,
    #[serde(skip_serializing_if = "is_zero_usize")]
    pub(crate) delegation_count: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) cache_hit_ratio: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) tokens_per_tool_call: Option<f64>,
    pub(crate) agent_strategy: String,
    pub(crate) outcome: String,
    pub(crate) turn_cost_tier: String,
    pub(crate) turn_effectiveness: String,
    pub(crate) recommended_attention: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) user_text_preview: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) final_assistant_kind: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) final_assistant_text_preview: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct SessionHotspot {
    pub(crate) session_path: String,
    pub(crate) duration_ms: i64,
    pub(crate) message_count: usize,
    #[serde(skip_serializing_if = "is_zero_usize")]
    pub(crate) tool_call_count: usize,
    #[serde(skip_serializing_if = "is_zero_u64")]
    pub(crate) input_tokens: u64,
    #[serde(skip_serializing_if = "is_zero_u64")]
    pub(crate) output_tokens: u64,
    #[serde(skip_serializing_if = "is_zero_u64")]
    pub(crate) reasoning_tokens: u64,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ExportHotspots {
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub(crate) slowest_sessions: Vec<SessionHotspot>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub(crate) turns: Vec<TurnHotspot>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub(crate) messages: Vec<MessageHotspot>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub(crate) tools: Vec<ToolHotspot>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct SessionIndexEntry {
    pub(crate) session_path: String,
    #[serde(skip_serializing)]
    pub(crate) depth: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) parent_session_path: Option<String>,
    pub(crate) title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) agent: Option<String>,
    #[serde(skip_serializing_if = "SessionRuntime::is_empty")]
    pub(crate) runtime: SessionRuntime,
    pub(crate) session_status: String,
    pub(crate) snapshot_completeness: String,
    pub(crate) duration_ms: i64,
    pub(crate) turn_count: usize,
    pub(crate) message_count: usize,
    #[serde(skip_serializing_if = "is_zero_usize")]
    pub(crate) tool_call_count: usize,
    pub(crate) summary_file: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) turns_compact_file: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) messages_compact_file: Option<String>,
    #[serde(skip_serializing)]
    pub(crate) messages_file: String,
    pub(crate) turns_file: String,
    #[serde(skip_serializing)]
    pub(crate) tool_calls_file: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct SessionSummaryFile {
    pub(crate) session: SessionFileMeta,
    #[serde(skip_serializing_if = "SessionRuntime::is_empty")]
    pub(crate) runtime: SessionRuntime,
    pub(crate) session_status: String,
    pub(crate) snapshot_completeness: String,
    pub(crate) last_activity_ms: i64,
    #[serde(skip_serializing_if = "is_zero_i64")]
    pub(crate) staleness_ms: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) session_narrative: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) prompt_preview: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) prompt_file: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) turns_compact_file: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) messages_compact_file: Option<String>,
    pub(crate) turns_file: String,
    pub(crate) messages_file: String,
    pub(crate) tool_calls_file: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) artifacts_dir: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) artifacts_manifest_file: Option<String>,
    #[serde(skip_serializing_if = "is_zero_usize")]
    pub(crate) artifact_count: usize,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub(crate) largest_artifacts: Vec<ArtifactManifestEntry>,
    pub(crate) totals: SessionTotals,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub(crate) hot_turns: Vec<TurnHotspot>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub(crate) pivotal_turns: Vec<usize>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub(crate) hot_messages: Vec<MessageHotspot>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub(crate) tool_rollup: Vec<ToolAggregate>,
    pub(crate) token_efficiency: TokenEfficiency,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub(crate) file_access_rollup: Vec<FileAccessRollupEntry>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub(crate) error_patterns: Vec<ErrorPatternEntry>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub(crate) retry_chains: Vec<RetryChainEntry>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub(crate) file_transition_rollup: Vec<FileTransitionEntry>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub(crate) session_deliverables: Vec<SessionDeliverableEntry>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub(crate) turn_dependency_edges: Vec<TurnDependencyEdge>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub(crate) children: Vec<ChildLink>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ArtifactPolicy {
    pub(crate) assistant_text_file_chars: usize,
    pub(crate) reasoning_file_chars: usize,
    pub(crate) tool_input_inline_chars: usize,
    pub(crate) tool_output_inline_chars: usize,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ClassificationPolicy {
    pub(crate) version: &'static str,
    pub(crate) user_intent_values: Vec<&'static str>,
    pub(crate) user_tag_values: Vec<&'static str>,
    pub(crate) message_kind_values: Vec<&'static str>,
    pub(crate) outcome_values: Vec<&'static str>,
    pub(crate) assistant_kind_values: Vec<&'static str>,
    pub(crate) session_status_values: Vec<&'static str>,
    pub(crate) agent_strategy_values: Vec<&'static str>,
    pub(crate) turn_cost_tier_values: Vec<&'static str>,
    pub(crate) turn_effectiveness_values: Vec<&'static str>,
    pub(crate) recommended_attention_values: Vec<&'static str>,
    pub(crate) child_export_reference_status_values: Vec<&'static str>,
    pub(crate) patch_intent_values: Vec<&'static str>,
    pub(crate) tool_call_purpose_values: Vec<&'static str>,
    pub(crate) retry_recovery_values: Vec<&'static str>,
    pub(crate) intent_confidence_range: &'static str,
    pub(crate) confidence_thresholds: ConfidenceThresholds,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ConfidenceThresholds {
    pub(crate) reliable_above: f64,
    pub(crate) uncertain_below: f64,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct TotalsDelta {
    #[serde(skip_serializing_if = "is_zero_i64")]
    pub(crate) session_count: i64,
    #[serde(skip_serializing_if = "is_zero_i64")]
    pub(crate) turn_count: i64,
    #[serde(skip_serializing_if = "is_zero_i64")]
    pub(crate) message_count: i64,
    #[serde(skip_serializing_if = "is_zero_i64")]
    pub(crate) user_message_count: i64,
    #[serde(skip_serializing_if = "is_zero_i64")]
    pub(crate) assistant_message_count: i64,
    #[serde(skip_serializing_if = "is_zero_i64")]
    pub(crate) text_chars: i64,
    #[serde(skip_serializing_if = "is_zero_i64")]
    pub(crate) reasoning_chars: i64,
    #[serde(skip_serializing_if = "is_zero_i64")]
    pub(crate) tool_calls: i64,
    #[serde(skip_serializing_if = "is_zero_i64")]
    pub(crate) input_tokens: i64,
    #[serde(skip_serializing_if = "is_zero_i64")]
    pub(crate) output_tokens: i64,
    #[serde(skip_serializing_if = "is_zero_i64")]
    pub(crate) reasoning_tokens: i64,
    #[serde(skip_serializing_if = "is_zero_i64")]
    pub(crate) cache_read_tokens: i64,
    #[serde(skip_serializing_if = "is_zero_i64")]
    pub(crate) cache_write_tokens: i64,
    #[serde(skip_serializing_if = "is_zero_f64")]
    pub(crate) cost: f64,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ToolRollupDelta {
    pub(crate) tool: String,
    #[serde(skip_serializing_if = "is_zero_i64")]
    pub(crate) calls_delta: i64,
    #[serde(skip_serializing_if = "is_zero_i64")]
    pub(crate) error_calls_delta: i64,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct IterationMeta {
    pub(crate) group_key: String,
    pub(crate) iteration_number: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) previous_export_path: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct DeltaFromPrevious {
    pub(crate) previous_export_path: String,
    pub(crate) previous_schema_version: String,
    pub(crate) current_schema_version: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub(crate) added_index_fields: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub(crate) removed_index_fields: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub(crate) changed_index_fields: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) totals_delta: Option<TotalsDelta>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub(crate) tool_rollup_deltas: Vec<ToolRollupDelta>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub(crate) turn_deltas: Vec<TurnDeltaEntry>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ExportIndexFile {
    pub(crate) format: &'static str,
    pub(crate) schema_version: &'static str,
    pub(crate) schema_file: String,
    pub(crate) fields_file: String,
    pub(crate) export_id: String,
    pub(crate) export_timestamp_ms: i64,
    pub(crate) iteration_meta: IterationMeta,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) delta_from_previous: Option<DeltaFromPrevious>,
    pub(crate) root_session_id: String,
    pub(crate) root_title: String,
    pub(crate) root_session_status: String,
    pub(crate) root_snapshot_completeness: String,
    pub(crate) root_last_activity_ms: i64,
    #[serde(skip_serializing_if = "is_zero_i64")]
    pub(crate) root_staleness_ms: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) root_task_preview: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) root_task_file: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub(crate) schema_changes: Vec<&'static str>,
    pub(crate) artifact_policy: ArtifactPolicy,
    pub(crate) classification_policy: ClassificationPolicy,
    pub(crate) recommended_read_order: Vec<String>,
    pub(crate) totals: ExportTotals,
    pub(crate) token_efficiency: TokenEfficiency,
    pub(crate) tree: ExportTreeNode,
    pub(crate) session_index: Vec<SessionIndexEntry>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub(crate) tool_rollup: Vec<ToolAggregate>,
    pub(crate) hotspots: ExportHotspots,
}

#[derive(Debug, Clone)]
pub(crate) struct ExportAccumulator {
    pub(crate) session_stats: Vec<SessionStats>,
    pub(crate) turns: Vec<TurnDigest>,
    pub(crate) message_digests: Vec<MessageDigest>,
    pub(crate) tool_calls: Vec<ToolCallDigest>,
    pub(crate) session_index: Vec<SessionIndexEntry>,
    pub(crate) session_hotspots: Vec<SessionHotspot>,
    pub(crate) root_task_file: Option<String>,
    pub(crate) root_task_preview: Option<String>,
}

#[derive(Debug, Clone)]
pub(crate) struct ChildDelegationInfo {
    pub(crate) message_index: usize,
    pub(crate) tool_index: usize,
    pub(crate) description: Option<String>,
    pub(crate) prompt_preview: Option<String>,
    pub(crate) prompt_preview_resolved: Option<String>,
    pub(crate) prompt_export_paths: Vec<String>,
    pub(crate) export_reference_status: Option<String>,
    pub(crate) input_file: Option<String>,
}

impl ExportAccumulator {
    pub(crate) fn new() -> Self {
        Self {
            session_stats: Vec::new(),
            turns: Vec::new(),
            message_digests: Vec::new(),
            tool_calls: Vec::new(),
            session_index: Vec::new(),
            session_hotspots: Vec::new(),
            root_task_file: None,
            root_task_preview: None,
        }
    }

    pub(crate) fn totals(&self) -> ExportTotals {
        let mut totals = ExportTotals {
            session_count: self.session_stats.len(),
            turn_count: self.turns.len(),
            message_count: self.message_digests.len(),
            user_message_count: 0,
            assistant_message_count: 0,
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

        for session in &self.session_stats {
            totals.user_message_count += session.user_message_count;
            totals.assistant_message_count += session.assistant_message_count;
            totals.text_chars += session.text_chars;
            totals.reasoning_chars += session.reasoning_chars;
            totals.tool_calls += session.tool_calls;
            totals.input_tokens += session.input_tokens;
            totals.output_tokens += session.output_tokens;
            totals.reasoning_tokens += session.reasoning_tokens;
            totals.cache_read_tokens += session.cache_read_tokens;
            totals.cache_write_tokens += session.cache_write_tokens;
            totals.cost += session.cost;
        }

        totals
    }
}

#[derive(Debug, Clone)]
pub(crate) struct DeliverableSnapshot {
    pub(crate) snapshot_file: String,
    pub(crate) content_sha256: String,
    pub(crate) line_count: usize,
    pub(crate) snapshot_source: String,
}
