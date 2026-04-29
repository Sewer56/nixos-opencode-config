#![recursion_limit = "512"]

use anyhow::{Context, Result, bail};
use chrono::{DateTime, Local, TimeZone, Utc};
use clap::{Args, Parser, Subcommand};
use crossterm::ExecutableCommand;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use crossterm::terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap};
use rusqlite::{Connection, OpenFlags, params};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};
use sha2::{Digest, Sha256};
use std::collections::{BTreeSet, HashMap, HashSet};
use std::fs::{self, File};
use std::io::{self, BufWriter, IsTerminal, Write};
use std::path::{Path, PathBuf};
use std::time::Duration;
use uuid::Uuid;

const TEXT_PREVIEW_LIMIT: usize = 160;
const INLINE_TEXT_PREVIEW_LIMIT: usize = 400;
const REASONING_PREVIEW_LIMIT: usize = 160;
const REASONING_SUMMARY_LIMIT: usize = 320;
const ACTIVITY_SUMMARY_LIMIT: usize = 220;
const ACTIVITY_SUMMARY_ITEMS_LIMIT: usize = 3;
const TURN_PREVIEW_LIMIT: usize = 220;
const TOOL_TEXT_PREVIEW_LIMIT: usize = 160;
const TOOL_INPUT_STRING_LIMIT: usize = 120;
const TOOL_INPUT_ITEMS_LIMIT: usize = 6;
const TOOL_INPUT_DEPTH_LIMIT: usize = 3;
const TOOL_INPUT_INLINE_CHARS_THRESHOLD: usize = 2_500;
const TOOL_PART_PREVIEW_LIMIT: usize = 140;
const SUBTASK_PREVIEW_LIMIT: usize = 200;
const PATCH_FILE_SAMPLE_LIMIT: usize = 6;
const SESSION_HOT_MESSAGES_LIMIT: usize = 5;
const EXPORT_HOTSPOT_LIMIT: usize = 3;
const TOOL_ROLLUP_DELTA_LIMIT: usize = 12;
const TURN_DELTA_LIMIT: usize = 12;
const TURN_FILE_SAMPLE_LIMIT: usize = 1;
const FILE_ACCESS_ROLLUP_LIMIT: usize = 16;
const ERROR_PATTERN_LIMIT: usize = 12;
const PATH_FALLBACK_COMPONENTS: usize = 6;
const MESSAGES_EMBEDDED_TEXT_LIMIT: usize = 2_000;
const ASSISTANT_TEXT_ARTIFACT_CHARS_THRESHOLD: usize = 400;
const REASONING_ARTIFACT_CHARS_THRESHOLD: usize = 6_000;
const TOOL_CALLS_EMBEDDED_IO_LIMIT: usize = 4_000;
const HOT_SESSION_TOKEN_THRESHOLD: u64 = 100_000;
const HOT_TURN_TOKEN_THRESHOLD: u64 = 500_000;
const HOT_MESSAGE_TOKEN_THRESHOLD: u64 = 50_000;
const HOT_TURN_TOOL_COUNT_THRESHOLD: usize = 3;
const HOT_MESSAGE_TOOL_COUNT_THRESHOLD: usize = 3;
const HOT_TURN_SLOW_MS_THRESHOLD: i64 = 60_000;
const HOT_MESSAGE_SLOW_MS_THRESHOLD: i64 = 30_000;
const HOT_TOOL_SLOW_MS_THRESHOLD: i64 = 1_000;
const HOT_TOOL_OUTPUT_CHARS_THRESHOLD: usize = 5_000;
const SCHEMA_VERSION: &str = "1.18";

#[derive(Parser)]
#[command(name = "opencode-sessions")]
#[command(version)]
#[command(about = "Browse and export OpenCode conversations from local SQLite")]
struct Cli {
    #[arg(long, global = true, value_name = "PATH")]
    db: Option<PathBuf>,

    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    /// Interactive ratatui browser
    Tui(TuiArgs),
    /// Print recent conversation tree
    Tree(TreeArgs),
    /// Export one conversation subtree into folder bundle
    Export(ExportArgs),
    /// List discovered OpenCode sqlite files
    Dbs,
}

#[derive(Args, Clone)]
struct TuiArgs {
    #[arg(long)]
    search: Option<String>,

    #[arg(long)]
    limit: Option<usize>,
}

#[derive(Args, Clone)]
struct TreeArgs {
    #[arg(long)]
    search: Option<String>,

    #[arg(long)]
    limit: Option<usize>,

    #[arg(long)]
    json: bool,
}

#[derive(Args, Clone)]
struct ExportArgs {
    /// Session id, or search text if exact id not found
    target: Option<String>,

    #[arg(long)]
    search: Option<String>,

    /// Base output dir. Tool creates one bundle folder inside.
    #[arg(long, value_name = "DIR")]
    out: Option<PathBuf>,

    #[arg(long)]
    latest: bool,
}

#[derive(Debug, Clone)]
struct SessionOverview {
    id: String,
    project_id: String,
    project_name: Option<String>,
    project_worktree: Option<String>,
    parent_id: Option<String>,
    directory: String,
    title: String,
    time_created: i64,
    time_updated: i64,
    message_count: usize,
}

impl SessionOverview {
    fn duration_ms(&self) -> i64 {
        self.time_updated.saturating_sub(self.time_created)
    }

    fn agent_hint(&self) -> Option<String> {
        extract_subagent_from_title(&self.title)
    }
}

#[derive(Debug, Clone)]
struct OverviewIndex {
    ordered_ids: Vec<String>,
    roots: Vec<String>,
    sessions: HashMap<String, SessionOverview>,
    children: HashMap<String, Vec<String>>,
}

impl OverviewIndex {
    fn get(&self, session_id: &str) -> Result<&SessionOverview> {
        self.sessions
            .get(session_id)
            .with_context(|| format!("session not found in overview: {session_id}"))
    }

    fn children_of(&self, session_id: &str) -> &[String] {
        self.children
            .get(session_id)
            .map(Vec::as_slice)
            .unwrap_or(&[])
    }

    fn latest_root(&self) -> Result<&str> {
        self.roots
            .first()
            .map(String::as_str)
            .context("no root sessions found")
    }

    fn all_expandable_ids(&self) -> HashSet<String> {
        self.children.keys().cloned().collect()
    }

    fn root_id(&self, session_id: &str) -> Result<String> {
        let mut current = session_id.to_string();
        let mut seen = HashSet::new();

        loop {
            if !seen.insert(current.clone()) {
                bail!("cycle detected while resolving root for {session_id}");
            }

            let session = self.get(&current)?;
            let Some(parent_id) = &session.parent_id else {
                return Ok(current);
            };
            current = parent_id.clone();
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct MessageTime {
    #[serde(default)]
    created: Option<i64>,
    #[serde(default)]
    completed: Option<i64>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct ModelRef {
    #[serde(rename = "providerID", default)]
    provider_id: Option<String>,
    #[serde(rename = "modelID", default)]
    model_id: Option<String>,
    #[serde(default)]
    variant: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct CacheTokens {
    #[serde(default)]
    read: u64,
    #[serde(default)]
    write: u64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct Tokens {
    #[serde(default)]
    total: Option<u64>,
    #[serde(default)]
    input: u64,
    #[serde(default)]
    output: u64,
    #[serde(default)]
    reasoning: u64,
    #[serde(default)]
    cache: CacheTokens,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct MessageInfo {
    #[serde(default)]
    role: String,
    #[serde(default)]
    time: MessageTime,
    #[serde(rename = "parentID", default)]
    parent_id: Option<String>,
    #[serde(default)]
    agent: Option<String>,
    #[serde(rename = "modelID", default)]
    model_id: Option<String>,
    #[serde(rename = "providerID", default)]
    provider_id: Option<String>,
    #[serde(default)]
    model: Option<ModelRef>,
    #[serde(default)]
    cost: Option<f64>,
    #[serde(default)]
    tokens: Option<Tokens>,
    #[serde(default)]
    error: Option<Value>,
    #[serde(default)]
    finish: Option<String>,
    #[serde(default)]
    mode: Option<String>,
    #[serde(default)]
    variant: Option<String>,
}

impl MessageInfo {
    fn created_ms(&self, fallback: i64) -> i64 {
        self.time.created.unwrap_or(fallback)
    }

    fn completed_ms(&self) -> Option<i64> {
        self.time.completed
    }

    fn duration_ms(&self) -> Option<i64> {
        self.time
            .created
            .zip(self.time.completed)
            .map(|(start, end)| end.saturating_sub(start))
    }

    fn model_name(&self) -> Option<String> {
        if let Some(model_id) = &self.model_id && !model_id.is_empty() {
            return Some(model_id.clone());
        }
        self.model
            .as_ref()
            .and_then(|model| model.model_id.clone())
            .filter(|value| !value.is_empty())
    }

    fn provider_name(&self) -> Option<String> {
        if let Some(provider_id) = &self.provider_id && !provider_id.is_empty() {
            return Some(provider_id.clone());
        }
        self.model
            .as_ref()
            .and_then(|model| model.provider_id.clone())
            .filter(|value| !value.is_empty())
    }
}

#[derive(Debug, Clone)]
struct LoadedPart {
    raw: Value,
}

#[derive(Debug, Clone)]
struct LoadedMessage {
    id: String,
    time_created: i64,
    info: MessageInfo,
    parts: Vec<LoadedPart>,
}

#[derive(Debug, Clone)]
struct LoadedSession {
    meta: SessionOverview,
    messages: Vec<LoadedMessage>,
    children: Vec<LoadedSession>,
}

impl LoadedSession {
    fn agent(&self) -> Option<String> {
        self.meta
            .agent_hint()
            .or_else(|| self.messages.iter().find_map(|message| message.info.agent.clone()))
    }
}

#[derive(Debug, Clone, Serialize)]
struct TreeNode {
    session_id: String,
    parent_session_id: Option<String>,
    title: String,
    agent: Option<String>,
    directory: String,
    project_name: Option<String>,
    project_worktree: Option<String>,
    created_ms: i64,
    updated_ms: i64,
    duration_ms: i64,
    message_count: usize,
    child_count: usize,
    children: Vec<TreeNode>,
}

#[derive(Debug, Clone)]
struct VisibleRow {
    session_id: String,
    depth: usize,
}

#[derive(Debug, Clone, Serialize)]
struct TokenStatsExport {
    #[serde(skip_serializing)]
    total: Option<u64>,
    #[serde(skip_serializing_if = "is_zero_u64")]
    input: u64,
    #[serde(skip_serializing_if = "is_zero_u64")]
    output: u64,
    #[serde(skip_serializing_if = "is_zero_u64")]
    reasoning: u64,
    #[serde(skip_serializing_if = "is_zero_u64")]
    cache_read: u64,
    #[serde(skip_serializing_if = "is_zero_u64")]
    cache_write: u64,
}

impl TokenStatsExport {
    fn is_empty(&self) -> bool {
        self.total.unwrap_or_default() == 0
            && self.input == 0
            && self.output == 0
            && self.reasoning == 0
            && self.cache_read == 0
            && self.cache_write == 0
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type")]
enum CompactPart {
    Text { text: String, synthetic: bool },
    Reasoning { text: String },
    Tool {
        tool: String,
        status: String,
        title: Option<String>,
        duration_ms: Option<i64>,
        input: Option<Value>,
        output_preview: Option<String>,
        output_chars: Option<usize>,
        error: Option<String>,
    },
    Agent { name: String },
    Subtask {
        agent: String,
        description: String,
        prompt: String,
        command: Option<String>,
    },
    File { filename: Option<String>, mime: Option<String> },
    Patch { file_count: usize, files: Vec<String> },
    Retry { attempt: Option<i64>, error: Option<String> },
}

#[derive(Debug, Clone, Serialize)]
struct CompactMessage {
    session_path: String,
    depth: usize,
    message_index: usize,
    message_id: String,
    role: String,
    agent: Option<String>,
    parent_message_id: Option<String>,
    model: Option<String>,
    provider: Option<String>,
    created_ms: i64,
    completed_ms: Option<i64>,
    duration_ms: Option<i64>,
    finish: Option<String>,
    cost: Option<f64>,
    tokens: Option<TokenStatsExport>,
    parts: Vec<CompactPart>,
}

#[derive(Debug, Clone, Serialize)]
struct SessionFileMeta {
    session_path: String,
    session_id: String,
    title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    agent: Option<String>,
    created_ms: i64,
    updated_ms: i64,
    duration_ms: i64,
}

#[derive(Debug, Clone, Serialize)]
struct ChildLink {
    session_path: String,
    session_id: String,
    title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    agent: Option<String>,
    summary_file: String,
    duration_ms: i64,
    turn_count: usize,
    message_count: usize,
    #[serde(skip_serializing_if = "is_zero_usize")]
    tool_call_count: usize,
    #[serde(skip_serializing_if = "is_zero_u64")]
    input_tokens: u64,
    #[serde(skip_serializing_if = "is_zero_u64")]
    output_tokens: u64,
    #[serde(skip_serializing_if = "is_zero_u64")]
    reasoning_tokens: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    parent_message_index: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    parent_tool_index: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    delegation_description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    delegation_prompt_preview: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    delegation_prompt_preview_resolved: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    delegation_prompt_export_paths: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    resolved_current_export_paths: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    delegation_export_reference_status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    current_export_path_hint: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    delegation_input_file: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    delegation_result_preview: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    delegation_result_file: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    child_outcome: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    child_final_assistant_kind: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
struct ExportTreeNode {
    session_path: String,
    summary_file: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    children: Vec<ExportTreeNode>,
}

#[derive(Debug, Clone, Serialize)]
struct SessionStats {
    session_path: String,
    depth: usize,
    session_id: String,
    parent_session_id: Option<String>,
    title: String,
    agent: Option<String>,
    created_ms: i64,
    updated_ms: i64,
    duration_ms: i64,
    turn_count: usize,
    message_count: usize,
    user_message_count: usize,
    assistant_message_count: usize,
    child_session_count: usize,
    text_chars: usize,
    reasoning_chars: usize,
    tool_calls: usize,
    input_tokens: u64,
    output_tokens: u64,
    reasoning_tokens: u64,
    cache_read_tokens: u64,
    cache_write_tokens: u64,
    cost: f64,
}

#[derive(Debug, Clone, Serialize)]
struct ExportTotals {
    session_count: usize,
    turn_count: usize,
    message_count: usize,
    user_message_count: usize,
    assistant_message_count: usize,
    text_chars: usize,
    reasoning_chars: usize,
    tool_calls: usize,
    input_tokens: u64,
    output_tokens: u64,
    reasoning_tokens: u64,
    #[serde(skip_serializing_if = "is_zero_u64")]
    cache_read_tokens: u64,
    #[serde(skip_serializing_if = "is_zero_u64")]
    cache_write_tokens: u64,
    #[serde(skip_serializing_if = "is_zero_f64")]
    cost: f64,
}

#[derive(Debug, Clone, Serialize)]
struct SessionTotals {
    turn_count: usize,
    message_count: usize,
    user_message_count: usize,
    assistant_message_count: usize,
    #[serde(skip_serializing_if = "is_zero_usize")]
    child_session_count: usize,
    text_chars: usize,
    reasoning_chars: usize,
    tool_calls: usize,
    input_tokens: u64,
    output_tokens: u64,
    reasoning_tokens: u64,
    #[serde(skip_serializing_if = "is_zero_u64")]
    cache_read_tokens: u64,
    #[serde(skip_serializing_if = "is_zero_u64")]
    cache_write_tokens: u64,
    #[serde(skip_serializing_if = "is_zero_f64")]
    cost: f64,
}

#[derive(Debug, Clone, Serialize)]
struct SessionRuntime {
    #[serde(skip_serializing_if = "Vec::is_empty")]
    models: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    providers: Vec<String>,
}

impl SessionRuntime {
    fn is_empty(&self) -> bool {
        self.models.is_empty() && self.providers.is_empty()
    }
}

#[derive(Debug, Clone, Serialize)]
struct MessageDigest {
    #[serde(skip_serializing)]
    session_path: String,
    message_index: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    turn_index: Option<usize>,
    role: String,
    message_kind: String,
    time_ms: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    provider: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    wall_gap_ms: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    duration_ms: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tokens: Option<TokenStatsExport>,
    #[serde(skip_serializing_if = "Option::is_none")]
    user_intent: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    user_intent_confidence: Option<f64>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    user_tags: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    alternative_user_intents: Vec<String>,
    #[serde(skip_serializing_if = "is_zero_usize")]
    text_chars: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    text_preview: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    text_file: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    activity_summary: Option<String>,
    #[serde(skip_serializing_if = "is_zero_usize")]
    reasoning_chars: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    reasoning_summary: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    reasoning_themes: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    reasoning_preview: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    reasoning_file: Option<String>,
    #[serde(skip_serializing_if = "is_zero_usize")]
    tool_count: usize,
    #[serde(skip_serializing_if = "is_zero_usize")]
    tool_error_count: usize,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    tool_names: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    subagent_calls: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    subtask_agents: Vec<String>,
    #[serde(skip_serializing_if = "is_zero_usize")]
    patch_file_count: usize,
    #[serde(skip_serializing_if = "is_zero_usize")]
    file_attachment_count: usize,
}

#[derive(Debug, Clone, Serialize)]
struct ToolCallDigest {
    #[serde(skip_serializing)]
    session_path: String,
    message_index: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    turn_index: Option<usize>,
    tool_index: usize,
    tool: String,
    #[serde(skip_serializing_if = "is_completed_status")]
    status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    duration_ms: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    input_preview: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    input_file: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    task_description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    task_prompt_preview: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    delegated_session_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    call_purpose: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    patch_summary: Option<PatchSummary>,
    #[serde(skip_serializing_if = "Option::is_none")]
    patch_intent: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    patch_file: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    output_preview: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    output_chars: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    output_file: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error_preview: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error_file: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    read_paths: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    modified_paths: Vec<String>,
    #[serde(skip_serializing)]
    modified_path_presence: HashMap<String, bool>,
    #[serde(skip_serializing)]
    input_tokens_proxy: u64,
}

#[derive(Debug, Clone, Serialize)]
struct ToolAggregate {
    tool: String,
    calls: usize,
    #[serde(skip_serializing_if = "is_zero_usize")]
    error_calls: usize,
    total_duration_ms: i64,
    max_duration_ms: i64,
    #[serde(skip_serializing_if = "is_zero_usize")]
    total_output_chars: usize,
    #[serde(skip_serializing_if = "is_zero_u64")]
    total_input_tokens_proxy: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    avg_input_tokens_proxy: Option<f64>,
}

#[derive(Debug, Clone, Serialize)]
struct FileAccessRollupEntry {
    path: String,
    #[serde(skip_serializing_if = "is_zero_usize")]
    read_count: usize,
    #[serde(skip_serializing_if = "is_zero_usize")]
    modified_count: usize,
    #[serde(skip_serializing_if = "is_zero_usize")]
    total_output_chars: usize,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    turn_indexes: Vec<usize>,
}

#[derive(Debug, Clone, Serialize)]
struct ErrorPatternEntry {
    tool: String,
    error_type: String,
    count: usize,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    turn_indexes: Vec<usize>,
    sample_message_index: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    sample_error_preview: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
struct RetryChainEntry {
    turn_index: usize,
    tool: String,
    error_type: String,
    attempts: usize,
    start_message_index: usize,
    end_message_index: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    recovery_strategy: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    sample_error_preview: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
struct FileSupersessionEntry {
    written_in_turn: usize,
    superseded_by_turn: usize,
}

#[derive(Debug, Clone, Serialize)]
struct FileTransitionEntry {
    path: String,
    write_count: usize,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    write_turns: Vec<usize>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    reread_in_turns: Vec<usize>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    rewritten_in_turns: Vec<usize>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    supersession_chain: Vec<FileSupersessionEntry>,
    #[serde(skip_serializing_if = "Option::is_none")]
    survives_to_end: Option<bool>,
}

#[derive(Debug, Clone, Serialize)]
struct SessionDeliverableEntry {
    path: String,
    write_count: usize,
    final_turn_index: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    final_patch_intent: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    snapshot_file: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    content_sha256: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    line_count: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    snapshot_source: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
struct ArtifactManifestEntry {
    path: String,
    category: String,
    size_bytes: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    message_index: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_index: Option<usize>,
}

#[derive(Debug, Clone, Serialize)]
struct ArtifactManifestFile {
    artifacts_dir: String,
    total_size_bytes: u64,
    entries: Vec<ArtifactManifestEntry>,
}

#[derive(Debug, Clone, Serialize)]
struct TurnCompactEntry {
    turn_index: usize,
    user_message_index: usize,
    message_index_end: usize,
    user_intent: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    response_elapsed_ms: Option<i64>,
    #[serde(skip_serializing_if = "is_zero_u64")]
    total_tokens: u64,
    #[serde(skip_serializing_if = "is_zero_usize")]
    tool_call_count: usize,
    #[serde(skip_serializing_if = "is_zero_usize")]
    modified_file_count: usize,
    agent_strategy: String,
    outcome: String,
    success: bool,
    turn_cost_tier: String,
    turn_effectiveness: String,
    recommended_attention: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    optimization_hints: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    failure_narrative: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    reasoning_summary: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    turn_change_summary: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    key_diff_preview: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
struct MessageCompactEntry {
    message_index: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    turn_index: Option<usize>,
    role: String,
    message_kind: String,
    time_ms: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    wall_gap_ms: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    duration_ms: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    total_tokens: Option<u64>,
    #[serde(skip_serializing_if = "is_zero_usize")]
    tool_count: usize,
    #[serde(skip_serializing_if = "is_zero_usize")]
    tool_error_count: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    text_preview: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    activity_summary: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    reasoning_summary: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
struct TurnDependencyEdge {
    from_turn: usize,
    to_turn: usize,
    relation: String,
    file_count: usize,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    sample_paths: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
struct TokenEfficiency {
    #[serde(skip_serializing_if = "Option::is_none")]
    cache_hit_ratio: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    avg_input_tokens_per_turn: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    avg_output_tokens_per_turn: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    avg_reasoning_tokens_per_turn: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    avg_tool_calls_per_turn: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    avg_input_tokens_per_tool_call: Option<f64>,
}

#[derive(Debug, Clone, Serialize)]
struct PatchSummary {
    #[serde(skip_serializing_if = "is_zero_usize")]
    files_added: usize,
    #[serde(skip_serializing_if = "is_zero_usize")]
    files_updated: usize,
    #[serde(skip_serializing_if = "is_zero_usize")]
    files_deleted: usize,
    #[serde(skip_serializing_if = "is_zero_usize")]
    files_moved: usize,
    #[serde(skip_serializing_if = "is_zero_usize")]
    hunks: usize,
    #[serde(skip_serializing_if = "is_zero_usize")]
    lines_added: usize,
    #[serde(skip_serializing_if = "is_zero_usize")]
    lines_deleted: usize,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    sample_paths: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
struct TurnToolAggregate {
    tool: String,
    calls: usize,
    #[serde(skip_serializing_if = "is_zero_usize")]
    error_calls: usize,
}

#[derive(Debug, Clone, Serialize)]
struct TurnPurposeAggregate {
    purpose: String,
    calls: usize,
}

#[derive(Debug, Clone, Serialize)]
struct TurnDelegationPreview {
    session_path: String,
    session_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    agent: Option<String>,
    parent_message_index: usize,
    parent_tool_index: usize,
}

#[derive(Debug, Clone, Serialize)]
struct TurnDigest {
    #[serde(skip_serializing)]
    session_path: String,
    turn_index: usize,
    user_message_index: usize,
    message_index_end: usize,
    time_ms: i64,
    user_intent: String,
    user_intent_confidence: f64,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    user_tags: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    alternative_user_intents: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    user_text_preview: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    user_text_file: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    assistant_message_start: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    assistant_message_end: Option<usize>,
    #[serde(skip_serializing_if = "is_zero_usize")]
    assistant_message_count: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    response_elapsed_ms: Option<i64>,
    wall_to_next_user_ms: i64,
    #[serde(skip_serializing_if = "is_zero_i64")]
    assistant_duration_ms: i64,
    #[serde(skip_serializing_if = "is_zero_i64")]
    tool_duration_ms: i64,
    #[serde(skip_serializing_if = "is_zero_usize")]
    tool_call_count: usize,
    #[serde(skip_serializing_if = "is_zero_usize")]
    error_count: usize,
    #[serde(skip_serializing_if = "is_zero_usize")]
    delegation_count: usize,
    #[serde(skip_serializing_if = "is_zero_u64")]
    input_tokens: u64,
    #[serde(skip_serializing_if = "is_zero_u64")]
    output_tokens: u64,
    #[serde(skip_serializing_if = "is_zero_u64")]
    reasoning_tokens: u64,
    #[serde(skip_serializing_if = "is_zero_u64")]
    cache_read_tokens: u64,
    #[serde(skip_serializing_if = "is_zero_u64")]
    cache_write_tokens: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    cache_hit_ratio: Option<f64>,
    #[serde(skip_serializing_if = "is_zero_u64")]
    total_tokens: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    tokens_per_tool_call: Option<f64>,
    #[serde(skip_serializing_if = "is_zero_usize")]
    read_file_count: usize,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    read_files: Vec<String>,
    #[serde(skip_serializing_if = "is_zero_usize")]
    modified_file_count: usize,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    modified_files: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    tool_rollup: Vec<TurnToolAggregate>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    call_purpose_rollup: Vec<TurnPurposeAggregate>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    delegations: Vec<TurnDelegationPreview>,
    #[serde(skip_serializing_if = "Option::is_none")]
    delegations_file: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    final_assistant_message_index: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    final_assistant_text_preview: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    final_assistant_text_file: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    final_assistant_kind: Option<String>,
    agent_strategy: String,
    outcome: String,
    success: bool,
    turn_cost_tier: String,
    turn_effectiveness: String,
    recommended_attention: String,
    #[serde(skip_serializing_if = "TurnEffectivenessSignals::is_empty")]
    effectiveness_signals: TurnEffectivenessSignals,
    #[serde(skip_serializing_if = "Option::is_none")]
    failure_narrative: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    optimization_hints: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    reasoning_summary: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    reasoning_themes: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    turn_change_summary: Option<String>,
    #[serde(skip_serializing_if = "TurnChangeStats::is_empty")]
    change_stats: TurnChangeStats,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    change_intents: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    key_diff_preview: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    next_user_message_index: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    next_user_intent: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    next_user_intent_confidence: Option<f64>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    next_user_tags: Vec<String>,
    #[serde(skip_serializing)]
    read_files_all: Vec<String>,
    #[serde(skip_serializing)]
    modified_files_all: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
struct TurnEffectivenessSignals {
    #[serde(skip_serializing_if = "is_zero_usize")]
    files_modified_count: usize,
    files_survived_to_end: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    retry_ratio: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    redundant_read_ratio: Option<f64>,
}

#[derive(Debug, Clone, Serialize)]
struct TurnChangeStats {
    #[serde(skip_serializing_if = "is_zero_usize")]
    patch_calls: usize,
    #[serde(skip_serializing_if = "is_zero_usize")]
    files_added: usize,
    #[serde(skip_serializing_if = "is_zero_usize")]
    files_updated: usize,
    #[serde(skip_serializing_if = "is_zero_usize")]
    files_deleted: usize,
    #[serde(skip_serializing_if = "is_zero_usize")]
    files_moved: usize,
    #[serde(skip_serializing_if = "is_zero_usize")]
    lines_added: usize,
    #[serde(skip_serializing_if = "is_zero_usize")]
    lines_deleted: usize,
}

impl TurnChangeStats {
    fn is_empty(&self) -> bool {
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
    fn is_empty(&self) -> bool {
        self.files_modified_count == 0
            && self.files_survived_to_end == 0
            && self.retry_ratio.is_none()
            && self.redundant_read_ratio.is_none()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TurnDeltaDigest {
    turn_index: usize,
    #[serde(default)]
    agent_strategy: Option<String>,
    #[serde(default)]
    turn_cost_tier: Option<String>,
    #[serde(default)]
    turn_effectiveness: Option<String>,
    #[serde(default)]
    input_tokens: u64,
    #[serde(default)]
    output_tokens: u64,
    #[serde(default)]
    reasoning_tokens: u64,
    #[serde(default)]
    cache_read_tokens: u64,
    #[serde(default)]
    cache_write_tokens: u64,
    #[serde(default)]
    tool_call_count: usize,
    #[serde(default)]
    error_count: usize,
    #[serde(default)]
    modified_file_count: usize,
}

#[derive(Debug, Clone, Serialize)]
struct TurnDeltaEntry {
    turn_index: usize,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    changed_fields: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    previous_agent_strategy: Option<String>,
    current_agent_strategy: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    previous_turn_cost_tier: Option<String>,
    current_turn_cost_tier: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    previous_turn_effectiveness: Option<String>,
    current_turn_effectiveness: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    previous_total_tokens: Option<u64>,
    current_total_tokens: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    previous_tool_call_count: Option<usize>,
    current_tool_call_count: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    previous_error_count: Option<usize>,
    current_error_count: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    previous_modified_file_count: Option<usize>,
    current_modified_file_count: usize,
}

#[derive(Debug, Clone, Serialize)]
struct MessageHotspot {
    session_path: String,
    start_message_index: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    end_message_index: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    turn_index: Option<usize>,
    #[serde(skip_serializing_if = "is_one_usize")]
    message_count: usize,
    role: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    hot_reasons: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    duration_ms: Option<i64>,
    #[serde(skip_serializing_if = "is_zero_u64")]
    total_tokens: u64,
    #[serde(skip_serializing_if = "is_zero_usize")]
    tool_calls: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pattern: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    sample_text_preview: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
struct ToolHotspot {
    session_path: String,
    message_index: usize,
    tool_index: usize,
    tool: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    hot_reasons: Vec<String>,
    #[serde(skip_serializing_if = "is_completed_status")]
    status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    duration_ms: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    output_chars: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    output_preview: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    output_file: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error_file: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
struct TurnHotspot {
    session_path: String,
    turn_index: usize,
    user_message_index: usize,
    user_intent: String,
    user_intent_confidence: f64,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    hot_reasons: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    response_elapsed_ms: Option<i64>,
    wall_to_next_user_ms: i64,
    #[serde(skip_serializing_if = "is_zero_u64")]
    total_tokens: u64,
    #[serde(skip_serializing_if = "is_zero_usize")]
    tool_calls: usize,
    #[serde(skip_serializing_if = "is_zero_usize")]
    error_count: usize,
    #[serde(skip_serializing_if = "is_zero_usize")]
    delegation_count: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    cache_hit_ratio: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tokens_per_tool_call: Option<f64>,
    agent_strategy: String,
    outcome: String,
    turn_cost_tier: String,
    turn_effectiveness: String,
    recommended_attention: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    user_text_preview: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    final_assistant_kind: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    final_assistant_text_preview: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
struct SessionHotspot {
    session_path: String,
    duration_ms: i64,
    message_count: usize,
    #[serde(skip_serializing_if = "is_zero_usize")]
    tool_call_count: usize,
    #[serde(skip_serializing_if = "is_zero_u64")]
    input_tokens: u64,
    #[serde(skip_serializing_if = "is_zero_u64")]
    output_tokens: u64,
    #[serde(skip_serializing_if = "is_zero_u64")]
    reasoning_tokens: u64,
}

#[derive(Debug, Clone, Serialize)]
struct ExportHotspots {
    #[serde(skip_serializing_if = "Vec::is_empty")]
    slowest_sessions: Vec<SessionHotspot>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    turns: Vec<TurnHotspot>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    messages: Vec<MessageHotspot>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    tools: Vec<ToolHotspot>,
}

#[derive(Debug, Clone, Serialize)]
struct SessionIndexEntry {
    session_path: String,
    #[serde(skip_serializing)]
    depth: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    parent_session_path: Option<String>,
    title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    agent: Option<String>,
    #[serde(skip_serializing_if = "SessionRuntime::is_empty")]
    runtime: SessionRuntime,
    session_status: String,
    snapshot_completeness: String,
    duration_ms: i64,
    turn_count: usize,
    message_count: usize,
    #[serde(skip_serializing_if = "is_zero_usize")]
    tool_call_count: usize,
    summary_file: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    turns_compact_file: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    messages_compact_file: Option<String>,
    #[serde(skip_serializing)]
    messages_file: String,
    turns_file: String,
    #[serde(skip_serializing)]
    tool_calls_file: String,
}

#[derive(Debug, Clone, Serialize)]
struct SessionSummaryFile {
    session: SessionFileMeta,
    #[serde(skip_serializing_if = "SessionRuntime::is_empty")]
    runtime: SessionRuntime,
    session_status: String,
    snapshot_completeness: String,
    last_activity_ms: i64,
    #[serde(skip_serializing_if = "is_zero_i64")]
    staleness_ms: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    session_narrative: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    prompt_preview: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    prompt_file: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    turns_compact_file: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    messages_compact_file: Option<String>,
    turns_file: String,
    messages_file: String,
    tool_calls_file: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    artifacts_dir: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    artifacts_manifest_file: Option<String>,
    #[serde(skip_serializing_if = "is_zero_usize")]
    artifact_count: usize,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    largest_artifacts: Vec<ArtifactManifestEntry>,
    totals: SessionTotals,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    hot_turns: Vec<TurnHotspot>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pivotal_turns: Vec<usize>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    hot_messages: Vec<MessageHotspot>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    tool_rollup: Vec<ToolAggregate>,
    token_efficiency: TokenEfficiency,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    file_access_rollup: Vec<FileAccessRollupEntry>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    error_patterns: Vec<ErrorPatternEntry>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    retry_chains: Vec<RetryChainEntry>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    file_transition_rollup: Vec<FileTransitionEntry>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    session_deliverables: Vec<SessionDeliverableEntry>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    turn_dependency_edges: Vec<TurnDependencyEdge>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    children: Vec<ChildLink>,
}

#[derive(Debug, Clone, Serialize)]
struct ArtifactPolicy {
    assistant_text_file_chars: usize,
    reasoning_file_chars: usize,
    tool_input_inline_chars: usize,
    tool_output_inline_chars: usize,
}

#[derive(Debug, Clone, Serialize)]
struct ClassificationPolicy {
    version: &'static str,
    user_intent_values: Vec<&'static str>,
    user_tag_values: Vec<&'static str>,
    message_kind_values: Vec<&'static str>,
    outcome_values: Vec<&'static str>,
    assistant_kind_values: Vec<&'static str>,
    session_status_values: Vec<&'static str>,
    agent_strategy_values: Vec<&'static str>,
    turn_cost_tier_values: Vec<&'static str>,
    turn_effectiveness_values: Vec<&'static str>,
    recommended_attention_values: Vec<&'static str>,
    child_export_reference_status_values: Vec<&'static str>,
    patch_intent_values: Vec<&'static str>,
    tool_call_purpose_values: Vec<&'static str>,
    retry_recovery_values: Vec<&'static str>,
    intent_confidence_range: &'static str,
    confidence_thresholds: ConfidenceThresholds,
}

#[derive(Debug, Clone, Serialize)]
struct ConfidenceThresholds {
    reliable_above: f64,
    uncertain_below: f64,
}

#[derive(Debug, Clone, Serialize)]
struct TotalsDelta {
    #[serde(skip_serializing_if = "is_zero_i64")]
    session_count: i64,
    #[serde(skip_serializing_if = "is_zero_i64")]
    turn_count: i64,
    #[serde(skip_serializing_if = "is_zero_i64")]
    message_count: i64,
    #[serde(skip_serializing_if = "is_zero_i64")]
    user_message_count: i64,
    #[serde(skip_serializing_if = "is_zero_i64")]
    assistant_message_count: i64,
    #[serde(skip_serializing_if = "is_zero_i64")]
    text_chars: i64,
    #[serde(skip_serializing_if = "is_zero_i64")]
    reasoning_chars: i64,
    #[serde(skip_serializing_if = "is_zero_i64")]
    tool_calls: i64,
    #[serde(skip_serializing_if = "is_zero_i64")]
    input_tokens: i64,
    #[serde(skip_serializing_if = "is_zero_i64")]
    output_tokens: i64,
    #[serde(skip_serializing_if = "is_zero_i64")]
    reasoning_tokens: i64,
    #[serde(skip_serializing_if = "is_zero_i64")]
    cache_read_tokens: i64,
    #[serde(skip_serializing_if = "is_zero_i64")]
    cache_write_tokens: i64,
    #[serde(skip_serializing_if = "is_zero_f64")]
    cost: f64,
}

#[derive(Debug, Clone, Serialize)]
struct ToolRollupDelta {
    tool: String,
    #[serde(skip_serializing_if = "is_zero_i64")]
    calls_delta: i64,
    #[serde(skip_serializing_if = "is_zero_i64")]
    error_calls_delta: i64,
}

#[derive(Debug, Clone, Serialize)]
struct IterationMeta {
    group_key: String,
    iteration_number: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    previous_export_path: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
struct DeltaFromPrevious {
    previous_export_path: String,
    previous_schema_version: String,
    current_schema_version: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    added_index_fields: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    removed_index_fields: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    changed_index_fields: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    totals_delta: Option<TotalsDelta>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    tool_rollup_deltas: Vec<ToolRollupDelta>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    turn_deltas: Vec<TurnDeltaEntry>,
}

#[derive(Debug, Clone, Serialize)]
struct ExportIndexFile {
    format: &'static str,
    schema_version: &'static str,
    schema_file: String,
    fields_file: String,
    export_id: String,
    export_timestamp_ms: i64,
    iteration_meta: IterationMeta,
    #[serde(skip_serializing_if = "Option::is_none")]
    delta_from_previous: Option<DeltaFromPrevious>,
    root_session_id: String,
    root_title: String,
    root_session_status: String,
    root_snapshot_completeness: String,
    root_last_activity_ms: i64,
    #[serde(skip_serializing_if = "is_zero_i64")]
    root_staleness_ms: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    root_task_preview: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    root_task_file: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    schema_changes: Vec<&'static str>,
    artifact_policy: ArtifactPolicy,
    classification_policy: ClassificationPolicy,
    recommended_read_order: Vec<String>,
    totals: ExportTotals,
    token_efficiency: TokenEfficiency,
    tree: ExportTreeNode,
    session_index: Vec<SessionIndexEntry>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    tool_rollup: Vec<ToolAggregate>,
    hotspots: ExportHotspots,
}

#[derive(Debug, Clone)]
struct ExportAccumulator {
    session_stats: Vec<SessionStats>,
    turns: Vec<TurnDigest>,
    message_digests: Vec<MessageDigest>,
    tool_calls: Vec<ToolCallDigest>,
    session_index: Vec<SessionIndexEntry>,
    session_hotspots: Vec<SessionHotspot>,
    root_task_file: Option<String>,
    root_task_preview: Option<String>,
}

#[derive(Debug, Clone)]
struct ChildDelegationInfo {
    message_index: usize,
    tool_index: usize,
    description: Option<String>,
    prompt_preview: Option<String>,
    prompt_preview_resolved: Option<String>,
    prompt_export_paths: Vec<String>,
    export_reference_status: Option<String>,
    input_file: Option<String>,
}

impl ExportAccumulator {
    fn new() -> Self {
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

    fn totals(&self) -> ExportTotals {
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

struct TuiApp {
    db_path: PathBuf,
    export_base: PathBuf,
    limit: Option<usize>,
    search: String,
    search_mode: bool,
    status: String,
    last_export: Option<PathBuf>,
    index: OverviewIndex,
    expanded: HashSet<String>,
    visible_rows: Vec<VisibleRow>,
    list_state: ListState,
}

impl TuiApp {
    fn new(db_path: PathBuf, export_base: PathBuf, index: OverviewIndex, args: TuiArgs) -> Self {
        let mut app = Self {
            db_path,
            export_base,
            limit: args.limit,
            search: args.search.unwrap_or_default(),
            search_mode: false,
            status: String::from("ready · e export selected · E export root · o open last export"),
            last_export: None,
            index,
            expanded: HashSet::new(),
            visible_rows: Vec::new(),
            list_state: ListState::default(),
        };
        app.refresh_rows();
        app
    }

    fn selected_session_id(&self) -> Option<&str> {
        self.list_state
            .selected()
            .and_then(|index| self.visible_rows.get(index))
            .map(|row| row.session_id.as_str())
    }

    fn refresh_rows(&mut self) {
        self.visible_rows = build_visible_rows(&self.index, self.search.trim(), self.limit, &self.expanded);
        if self.visible_rows.is_empty() {
            self.list_state.select(None);
            return;
        }

        let current = self.list_state.selected().unwrap_or(0);
        self.list_state
            .select(Some(current.min(self.visible_rows.len().saturating_sub(1))));
    }

    fn move_selection(&mut self, delta: isize) {
        if self.visible_rows.is_empty() {
            self.list_state.select(None);
            return;
        }

        let current = self.list_state.selected().unwrap_or(0) as isize;
        let next = (current + delta).clamp(0, self.visible_rows.len().saturating_sub(1) as isize) as usize;
        self.list_state.select(Some(next));
    }

    fn toggle_selected(&mut self) {
        if !self.search.trim().is_empty() {
            self.status = String::from("search mode auto-expands matching branches");
            return;
        }

        let Some(session_id) = self.selected_session_id().map(str::to_owned) else {
            return;
        };

        if self.index.children_of(&session_id).is_empty() {
            return;
        }

        if self.expanded.contains(&session_id) {
            self.expanded.remove(&session_id);
        } else {
            self.expanded.insert(session_id);
        }

        self.refresh_rows();
    }

    fn expand_all(&mut self) {
        self.expanded = self.index.all_expandable_ids();
        self.refresh_rows();
        self.status = String::from("expanded all");
    }

    fn collapse_all(&mut self) {
        self.expanded.clear();
        self.refresh_rows();
        self.status = String::from("collapsed all");
    }
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Some(Command::Dbs) => print_discovered_dbs(cli.db.as_deref()),
        Some(Command::Tree(args)) => {
            let db_path = resolve_db_path(cli.db.as_deref())?;
            let conn = open_db(&db_path)?;
            let index = load_overview(&conn)?;
            run_tree_command(&db_path, &index, args)
        }
        Some(Command::Export(args)) => {
            let db_path = resolve_db_path(cli.db.as_deref())?;
            let conn = open_db(&db_path)?;
            let index = load_overview(&conn)?;
            let session_id = resolve_target_session_id(&index, &args)?;
            let export_root = export_bundle(&conn, &index, &session_id, args.out)?;
            println!("{}", export_root.display());
            Ok(())
        }
        Some(Command::Tui(args)) => {
            let db_path = resolve_db_path(cli.db.as_deref())?;
            let conn = open_db(&db_path)?;
            let index = load_overview(&conn)?;
            run_tui(db_path, index, args)
        }
        None => {
            let db_path = resolve_db_path(cli.db.as_deref())?;
            let conn = open_db(&db_path)?;
            let index = load_overview(&conn)?;

            if io::stdout().is_terminal() {
                run_tui(db_path, index, TuiArgs { search: None, limit: None })
            } else {
                run_tree_command(
                    &db_path,
                    &index,
                    TreeArgs {
                        search: None,
                        limit: None,
                        json: false,
                    },
                )
            }
        }
    }
}

fn print_discovered_dbs(explicit: Option<&Path>) -> Result<()> {
    let discovered = discover_db_paths()?;
    let default = resolve_db_path(explicit).ok();

    if discovered.is_empty() {
        bail!("no OpenCode sqlite files found under {}", opencode_data_dir()?.display());
    }

    for path in discovered {
        let metadata = fs::metadata(&path).with_context(|| format!("read metadata for {}", path.display()))?;
        let modified = metadata.modified().ok().map(format_system_time).unwrap_or_else(|| "unknown".into());
        let mark = if default.as_deref() == Some(path.as_path()) {
            "*"
        } else {
            " "
        };
        println!(
            "{} {}  size={}  modified={}",
            mark,
            path.display(),
            format_bytes(metadata.len()),
            modified
        );
    }

    Ok(())
}

fn run_tree_command(db_path: &Path, index: &OverviewIndex, args: TreeArgs) -> Result<()> {
    let search = args.search.unwrap_or_default();

    if args.json {
        let roots = build_tree_nodes(index, search.trim(), args.limit);
        let payload = json!({
            "db_path": db_path.display().to_string(),
            "root_count": roots.len(),
            "sessions": roots,
        });
        println!("{}", serde_json::to_string_pretty(&payload)?);
        return Ok(());
    }

    let lines = build_text_tree(index, search.trim(), args.limit);
    if lines.is_empty() {
        println!("No matching sessions.");
        return Ok(());
    }

    println!("DB: {}", db_path.display());
    if !search.trim().is_empty() {
        println!("Search: {}", search.trim());
    }
    println!();
    for line in lines {
        println!("{line}");
    }
    Ok(())
}

fn run_tui(db_path: PathBuf, index: OverviewIndex, args: TuiArgs) -> Result<()> {
    let export_base = default_export_base_dir();
    fs::create_dir_all(&export_base).with_context(|| format!("create {}", export_base.display()))?;

    enable_raw_mode().context("enable raw mode")?;
    let mut stdout = io::stdout();
    stdout.execute(EnterAlternateScreen).context("enter alt screen")?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).context("create terminal")?;
    let mut app = TuiApp::new(db_path.clone(), export_base, index, args);

    let result = (|| -> Result<()> {
        loop {
            terminal.draw(|frame| draw_tui(frame, &mut app)).context("draw tui")?;

            if !event::poll(Duration::from_millis(250)).context("poll terminal events")? {
                continue;
            }

            let Event::Key(key) = event::read().context("read terminal event")? else {
                continue;
            };

            if key.kind != KeyEventKind::Press {
                continue;
            }

            if handle_tui_key(&mut app, key)? {
                break;
            }
        }

        Ok(())
    })();

    disable_raw_mode().ok();
    let mut stdout = io::stdout();
    stdout.execute(LeaveAlternateScreen).ok();
    result
}

fn draw_tui(frame: &mut ratatui::Frame<'_>, app: &mut TuiApp) {
    let areas = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(6),
            Constraint::Length(3),
            Constraint::Min(1),
            Constraint::Length(4),
        ])
        .split(frame.area());

    let header = Paragraph::new(vec![
        Line::from(vec![
            Span::styled("OpenCode Sessions", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw("  "),
            Span::styled(
                format!("roots={} visible={}", app.index.roots.len(), app.visible_rows.len()),
                Style::default().fg(Color::Cyan),
            ),
        ]),
        Line::from(vec![
            Span::styled("DB: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(app.db_path.display().to_string()),
        ]),
        Line::from(vec![
            Span::styled("export: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw("e selected subtree  E root conversation  o open last export"),
        ]),
        Line::from(vec![
            Span::styled("browse: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw("↑↓/jk move  enter toggle  / search  esc clear search  a expand all  z collapse all  q quit"),
        ]),
    ])
    .block(Block::default().borders(Borders::ALL));
    frame.render_widget(header, areas[0]);

    let search_title = if app.search_mode { "Search (typing)" } else { "Search (/ to edit)" };
    let search = Paragraph::new(app.search.as_str())
        .block(Block::default().borders(Borders::ALL).title(search_title))
        .wrap(Wrap { trim: false });
    frame.render_widget(search, areas[1]);

    let items = if app.visible_rows.is_empty() {
        vec![ListItem::new(Line::from("No sessions match search."))]
    } else {
        app.visible_rows
            .iter()
            .map(|row| ListItem::new(Line::from(format_row(app, row))))
            .collect()
    };

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("Recent conversations"))
        .highlight_style(Style::default().bg(Color::Rgb(30, 30, 70)).fg(Color::White))
        .highlight_symbol("▶ ");

    frame.render_stateful_widget(list, areas[2], &mut app.list_state);

    let footer = Paragraph::new(vec![
        Line::from(vec![
            Span::styled("selected: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(selected_summary(app)),
        ]),
        Line::from(vec![
            Span::styled("status: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(app.status.as_str()),
        ]),
    ])
    .block(Block::default().borders(Borders::ALL));
    frame.render_widget(footer, areas[3]);
}

fn handle_tui_key(app: &mut TuiApp, key: KeyEvent) -> Result<bool> {
    if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
        return Ok(true);
    }

    if app.search_mode {
        match key.code {
            KeyCode::Esc => {
                app.search_mode = false;
                app.status = String::from("search mode off");
            }
            KeyCode::Enter => {
                app.search_mode = false;
                app.status = format!("search = {}", app.search.trim());
            }
            KeyCode::Backspace => {
                app.search.pop();
                app.refresh_rows();
            }
            KeyCode::Char(ch) => {
                app.search.push(ch);
                app.refresh_rows();
            }
            _ => {}
        }
        return Ok(false);
    }

    match key.code {
        KeyCode::Char('q') => return Ok(true),
        KeyCode::Up | KeyCode::Char('k') => app.move_selection(-1),
        KeyCode::Down | KeyCode::Char('j') => app.move_selection(1),
        KeyCode::Enter | KeyCode::Char(' ') => app.toggle_selected(),
        KeyCode::Char('/') => {
            app.search_mode = true;
            app.status = String::from("search mode on");
        }
        KeyCode::Char('a') => app.expand_all(),
        KeyCode::Char('z') => app.collapse_all(),
        KeyCode::Esc => {
            if !app.search.is_empty() {
                app.search.clear();
                app.refresh_rows();
                app.status = String::from("search cleared");
            }
        }
        KeyCode::Char('e') => {
            let Some(session_id) = app.selected_session_id().map(str::to_owned) else {
                app.status = String::from("nothing selected");
                return Ok(false);
            };
            let conn = open_db(&app.db_path)?;
            let export_root = export_bundle(&conn, &app.index, &session_id, Some(app.export_base.clone()))?;
            app.last_export = Some(export_root.clone());
            app.status = format!("exported selected -> {}", export_root.display());
        }
        KeyCode::Char('E') => {
            let Some(session_id) = app.selected_session_id().map(str::to_owned) else {
                app.status = String::from("nothing selected");
                return Ok(false);
            };
            let root_id = app.index.root_id(&session_id)?;
            let conn = open_db(&app.db_path)?;
            let export_root = export_bundle(&conn, &app.index, &root_id, Some(app.export_base.clone()))?;
            app.last_export = Some(export_root.clone());
            app.status = format!("exported root -> {}", export_root.display());
        }
        KeyCode::Char('o') => {
            let Some(path) = app.last_export.as_ref() else {
                app.status = String::from("no export yet");
                return Ok(false);
            };
            open_path(path)?;
            app.status = format!("opened -> {}", path.display());
        }
        _ => {}
    }

    Ok(false)
}

fn format_row(app: &TuiApp, row: &VisibleRow) -> String {
    let session = match app.index.sessions.get(&row.session_id) {
        Some(session) => session,
        None => return row.session_id.clone(),
    };

    let children = app.index.children_of(&row.session_id);
    let marker = if children.is_empty() {
        "·"
    } else if !app.search.trim().is_empty() || app.expanded.contains(&row.session_id) {
        "▾"
    } else {
        "▸"
    };

    let indent = "  ".repeat(row.depth);
    let kind = if row.depth == 0 {
        String::from("root")
    } else {
        session.agent_hint().unwrap_or_else(|| String::from("subagent"))
    };

    format!(
        "{}{} [{}] {}  {}  {}  {} msgs  {}",
        indent,
        marker,
        kind,
        session.title,
        short_id(&session.id),
        format_local_timestamp(session.time_updated),
        session.message_count,
        format_duration(session.duration_ms()),
    )
}

fn selected_summary(app: &TuiApp) -> String {
    let Some(session_id) = app.selected_session_id() else {
        return String::from("none");
    };
    let Some(session) = app.index.sessions.get(session_id) else {
        return String::from("none");
    };

    let kind = if session.parent_id.is_some() {
        session.agent_hint().unwrap_or_else(|| String::from("subagent"))
    } else {
        String::from("root")
    };

    format!("[{}] {}  {}", kind, session.title, short_id(&session.id))
}

fn open_path(path: &Path) -> Result<()> {
    let opener = if cfg!(target_os = "macos") {
        "open"
    } else {
        "xdg-open"
    };

    std::process::Command::new(opener)
        .arg(path)
        .spawn()
        .with_context(|| format!("launch {opener} for {}", path.display()))?;
    Ok(())
}

fn opencode_data_dir() -> Result<PathBuf> {
    dirs::home_dir()
        .map(|home| home.join(".local/share/opencode"))
        .context("could not resolve home directory")
}

fn discover_db_paths() -> Result<Vec<PathBuf>> {
    let data_dir = opencode_data_dir()?;
    if !data_dir.is_dir() {
        return Ok(Vec::new());
    }

    let mut found = Vec::new();
    for entry in fs::read_dir(&data_dir).with_context(|| format!("read {}", data_dir.display()))? {
        let entry = entry?;
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let Some(name) = path.file_name().and_then(|value| value.to_str()) else {
            continue;
        };
        if name == "opencode.db" || (name.starts_with("opencode-") && name.ends_with(".db")) {
            found.push(path);
        }
    }

    found.sort_by(|left, right| {
        let left_meta = fs::metadata(left).ok();
        let right_meta = fs::metadata(right).ok();
        let left_mtime = left_meta.and_then(|meta| meta.modified().ok());
        let right_mtime = right_meta.and_then(|meta| meta.modified().ok());
        right_mtime
            .cmp(&left_mtime)
            .then_with(|| left.file_name().cmp(&right.file_name()))
    });
    Ok(found)
}

fn resolve_db_path(explicit: Option<&Path>) -> Result<PathBuf> {
    if let Some(path) = explicit {
        let path = if path.is_absolute() {
            path.to_path_buf()
        } else {
            std::env::current_dir()?.join(path)
        };
        if !path.is_file() {
            bail!("db file not found: {}", path.display());
        }
        return Ok(path);
    }

    if let Some(env_db) = std::env::var_os("OPENCODE_DB") {
        let raw = PathBuf::from(env_db);
        if raw.as_os_str() != ":memory:" {
            let resolved = if raw.is_absolute() { raw } else { opencode_data_dir()?.join(raw) };
            if resolved.is_file() {
                return Ok(resolved);
            }
        }
    }

    let discovered = discover_db_paths()?;
    discovered
        .into_iter()
        .next()
        .context("no OpenCode sqlite database found; use --db to point at one")
}

fn open_db(path: &Path) -> Result<Connection> {
    let flags = OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX;
    let conn = Connection::open_with_flags(path, flags)
        .with_context(|| format!("open sqlite db {}", path.display()))?;
    conn.busy_timeout(Duration::from_secs(5))?;
    conn.pragma_update(None, "query_only", true)?;
    Ok(conn)
}

fn load_overview(conn: &Connection) -> Result<OverviewIndex> {
    let sql = r#"
        select
          s.id,
          s.project_id,
          s.parent_id,
          s.directory,
          s.title,
          s.time_created,
          s.time_updated,
          coalesce(m.message_count, 0) as message_count,
          p.worktree,
          p.name
        from session s
        left join project p on p.id = s.project_id
        left join (
          select session_id, count(*) as message_count
          from message
          group by session_id
        ) m on m.session_id = s.id
        where s.time_archived is null
        order by s.time_updated desc, s.id desc
    "#;

    let mut stmt = conn.prepare(sql)?;
    let mut rows = stmt.query([])?;

    let mut ordered_ids = Vec::new();
    let mut sessions = HashMap::new();

    while let Some(row) = rows.next()? {
        let session = SessionOverview {
            id: row.get(0)?,
            project_id: row.get(1)?,
            parent_id: row.get(2)?,
            directory: row.get(3)?,
            title: row.get(4)?,
            time_created: row.get(5)?,
            time_updated: row.get(6)?,
            message_count: usize::try_from(row.get::<_, i64>(7)?).unwrap_or_default(),
            project_worktree: row.get(8)?,
            project_name: row.get(9)?,
        };

        ordered_ids.push(session.id.clone());
        sessions.insert(session.id.clone(), session);
    }

    let mut roots = Vec::new();
    let mut children: HashMap<String, Vec<String>> = HashMap::new();

    for session_id in &ordered_ids {
        let session = sessions
            .get(session_id)
            .with_context(|| format!("missing session after load: {session_id}"))?;
        if let Some(parent_id) = &session.parent_id {
            children.entry(parent_id.clone()).or_default().push(session.id.clone());
        } else {
            roots.push(session.id.clone());
        }
    }

    Ok(OverviewIndex {
        ordered_ids,
        roots,
        sessions,
        children,
    })
}

fn resolve_target_session_id(index: &OverviewIndex, args: &ExportArgs) -> Result<String> {
    if let Some(target) = &args.target {
        if index.sessions.contains_key(target) {
            return Ok(target.clone());
        }

        let matches = search_session_ids(index, target);
        return matches
            .into_iter()
            .next()
            .with_context(|| format!("no session id or search match for {target:?}"));
    }

    if let Some(search) = &args.search {
        let matches = search_session_ids(index, search);
        return matches
            .into_iter()
            .next()
            .with_context(|| format!("no session matches {search:?}"));
    }

    if args.latest || args.target.is_none() {
        return Ok(index.latest_root()?.to_string());
    }

    bail!("unable to resolve target session")
}

fn search_session_ids(index: &OverviewIndex, query: &str) -> Vec<String> {
    let query = query.trim().to_lowercase();
    if query.is_empty() {
        return index.ordered_ids.clone();
    }

    index
        .ordered_ids
        .iter()
        .filter_map(|session_id| {
            let session = index.sessions.get(session_id)?;
            let agent = session.agent_hint().unwrap_or_default();
            let haystacks = [
                session.id.as_str(),
                session.title.as_str(),
                session.directory.as_str(),
                session.project_id.as_str(),
                session.project_name.as_deref().unwrap_or_default(),
                session.project_worktree.as_deref().unwrap_or_default(),
                agent.as_str(),
            ];

            haystacks
                .iter()
                .any(|value| value.to_lowercase().contains(&query))
                .then(|| session_id.clone())
        })
        .collect()
}

fn build_visible_rows(
    index: &OverviewIndex,
    search: &str,
    limit: Option<usize>,
    expanded: &HashSet<String>,
) -> Vec<VisibleRow> {
    let mut rows = Vec::new();
    let roots = limit_roots(&index.roots, limit);

    if search.is_empty() {
        for root in roots {
            push_rows_normal(index, root, 0, expanded, &mut rows);
        }
        return rows;
    }

    for root in roots {
        push_rows_filtered(index, root, 0, search, &mut rows);
    }
    rows
}

fn limit_roots<'a>(roots: &'a [String], limit: Option<usize>) -> &'a [String] {
    match limit {
        Some(limit) => &roots[..roots.len().min(limit)],
        None => roots,
    }
}

fn push_rows_normal(
    index: &OverviewIndex,
    session_id: &str,
    depth: usize,
    expanded: &HashSet<String>,
    rows: &mut Vec<VisibleRow>,
) {
    rows.push(VisibleRow {
        session_id: session_id.to_string(),
        depth,
    });

    if !expanded.contains(session_id) {
        return;
    }

    for child_id in index.children_of(session_id) {
        push_rows_normal(index, child_id, depth + 1, expanded, rows);
    }
}

fn push_rows_filtered(
    index: &OverviewIndex,
    session_id: &str,
    depth: usize,
    search: &str,
    rows: &mut Vec<VisibleRow>,
) -> bool {
    let session = match index.sessions.get(session_id) {
        Some(session) => session,
        None => return false,
    };

    let self_match = session_matches_query(session, search);
    let mut child_matches = false;
    for child_id in index.children_of(session_id) {
        child_matches |= push_rows_filtered(index, child_id, depth + 1, search, rows);
    }

    if self_match || child_matches {
        let insert_at = rows
            .iter()
            .position(|row| row.depth < depth)
            .unwrap_or(rows.len());
        rows.insert(
            insert_at,
            VisibleRow {
                session_id: session_id.to_string(),
                depth,
            },
        );
        return true;
    }

    false
}

fn session_matches_query(session: &SessionOverview, search: &str) -> bool {
    let query = search.trim().to_lowercase();
    if query.is_empty() {
        return true;
    }

    let agent = session.agent_hint().unwrap_or_default();
    [
        session.id.as_str(),
        session.title.as_str(),
        session.directory.as_str(),
        session.project_id.as_str(),
        session.project_name.as_deref().unwrap_or_default(),
        session.project_worktree.as_deref().unwrap_or_default(),
        agent.as_str(),
    ]
    .iter()
    .any(|value| value.to_lowercase().contains(&query))
}

fn build_tree_nodes(index: &OverviewIndex, search: &str, limit: Option<usize>) -> Vec<TreeNode> {
    limit_roots(&index.roots, limit)
        .iter()
        .filter_map(|root_id| build_tree_node(index, root_id, search))
        .collect()
}

fn build_tree_node(index: &OverviewIndex, session_id: &str, search: &str) -> Option<TreeNode> {
    let session = index.sessions.get(session_id)?;
    let children: Vec<TreeNode> = index
        .children_of(session_id)
        .iter()
        .filter_map(|child_id| build_tree_node(index, child_id, search))
        .collect();

    if !search.trim().is_empty() && !session_matches_query(session, search) && children.is_empty() {
        return None;
    }

    Some(TreeNode {
        session_id: session.id.clone(),
        parent_session_id: session.parent_id.clone(),
        title: session.title.clone(),
        agent: session.agent_hint(),
        directory: session.directory.clone(),
        project_name: session.project_name.clone(),
        project_worktree: session.project_worktree.clone(),
        created_ms: session.time_created,
        updated_ms: session.time_updated,
        duration_ms: session.duration_ms(),
        message_count: session.message_count,
        child_count: children.len(),
        children,
    })
}

fn build_text_tree(index: &OverviewIndex, search: &str, limit: Option<usize>) -> Vec<String> {
    let mut lines = Vec::new();
    let roots: Vec<&String> = limit_roots(&index.roots, limit)
        .iter()
        .filter(|root_id| subtree_matches(index, root_id, search))
        .collect();

    for (root_index, root_id) in roots.iter().enumerate() {
        push_text_tree_lines(
            index,
            root_id,
            search,
            String::new(),
            root_index + 1 == roots.len(),
            &mut lines,
        );
    }

    lines
}

fn subtree_matches(index: &OverviewIndex, session_id: &str, search: &str) -> bool {
    if search.trim().is_empty() {
        return true;
    }

    let Some(session) = index.sessions.get(session_id) else {
        return false;
    };

    session_matches_query(session, search)
        || index
            .children_of(session_id)
            .iter()
            .any(|child_id| subtree_matches(index, child_id, search))
}

fn push_text_tree_lines(
    index: &OverviewIndex,
    session_id: &str,
    search: &str,
    prefix: String,
    is_last: bool,
    lines: &mut Vec<String>,
) {
    let Some(session) = index.sessions.get(session_id) else {
        return;
    };

    if !search.trim().is_empty() && !subtree_matches(index, session_id, search) {
        return;
    }

    let branch = if prefix.is_empty() {
        String::new()
    } else if is_last {
        format!("{prefix}└── ")
    } else {
        format!("{prefix}├── ")
    };
    let kind = session
        .agent_hint()
        .map(|agent| format!("@{agent}"))
        .unwrap_or_else(|| String::from("root"));
    lines.push(format!(
        "{}[{}] {}  {}  {}  {} msgs  {}",
        branch,
        kind,
        session.title,
        short_id(&session.id),
        format_local_timestamp(session.time_updated),
        session.message_count,
        format_duration(session.duration_ms()),
    ));

    let filtered_children: Vec<&String> = index
        .children_of(session_id)
        .iter()
        .filter(|child_id| subtree_matches(index, child_id, search))
        .collect();
    let next_prefix = if prefix.is_empty() {
        String::new()
    } else if is_last {
        format!("{prefix}    ")
    } else {
        format!("{prefix}│   ")
    };

    for (child_index, child_id) in filtered_children.iter().enumerate() {
        push_text_tree_lines(
            index,
            child_id,
            search,
            next_prefix.clone(),
            child_index + 1 == filtered_children.len(),
            lines,
        );
    }
}

fn export_bundle(
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
    acc.session_hotspots.sort_by(|left, right| right.duration_ms.cmp(&left.duration_ms));

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

fn load_session_tree(conn: &Connection, index: &OverviewIndex, session_id: &str) -> Result<LoadedSession> {
    let meta = index.get(session_id)?.clone();
    let messages = load_messages(conn, session_id)?;
    let children = index
        .children_of(session_id)
        .iter()
        .map(|child_id| load_session_tree(conn, index, child_id))
        .collect::<Result<Vec<_>>>()?;

    Ok(LoadedSession { meta, messages, children })
}

fn load_messages(conn: &Connection, session_id: &str) -> Result<Vec<LoadedMessage>> {
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

fn write_session_bundle(
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
    let (turn_digests, message_digests, tool_digests, runtime, prompt_preview, prompt_file, artifacts_dir, artifact_count) =
        build_session_machine_output(
            session,
            &compact_messages,
            &child_links,
            &session_dir,
            &relative_session_dir,
            session_path,
        )?;
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

fn compact_message(
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

fn build_session_machine_output(
    session: &LoadedSession,
    compact_messages: &[CompactMessage],
    child_links: &[ChildLink],
    session_dir: &Path,
    relative_session_dir: &Path,
    session_path: &str,
) -> Result<(
    Vec<TurnDigest>,
    Vec<MessageDigest>,
    Vec<ToolCallDigest>,
    SessionRuntime,
    Option<String>,
    Option<String>,
    Option<String>,
    usize,
)> {
    let artifacts_dir = session_dir.join("artifacts");
    let artifacts_rel_dir = relative_session_dir.join("artifacts");
    let mut artifacts_created = false;
    let mut artifact_count = 0usize;
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
        let user_tags = (compact.role == "user")
            .then(|| classify_user_tags(text_preview.as_deref()))
            .unwrap_or_default();
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
                &artifacts_dir,
                &artifacts_rel_dir,
                &mut artifacts_created,
                &mut artifact_count,
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
                &artifacts_dir,
                &artifacts_rel_dir,
                &mut artifacts_created,
                &mut artifact_count,
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
                    &artifacts_dir,
                    &artifacts_rel_dir,
                    &mut artifacts_created,
                    &mut artifact_count,
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
                &artifacts_dir,
                &artifacts_rel_dir,
                &mut artifacts_created,
                &mut artifact_count,
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
                        &artifacts_dir,
                        &artifacts_rel_dir,
                        &mut artifacts_created,
                        &mut artifact_count,
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
            let modified_path_presence = patch_text
                .map(patch_path_presence_from_text)
                .unwrap_or_default();
            let effective_read_paths = (status != "error").then_some(read_paths.clone()).unwrap_or_default();
            let effective_modified_paths = (status != "error")
                .then_some(modified_paths.clone())
                .unwrap_or_default();
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
                        &artifacts_dir,
                        &artifacts_rel_dir,
                        &mut artifacts_created,
                        &mut artifact_count,
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
                    &artifacts_dir,
                    &artifacts_rel_dir,
                    &mut artifacts_created,
                    &mut artifact_count,
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
                    &artifacts_dir,
                    &artifacts_rel_dir,
                    &mut artifacts_created,
                    &mut artifact_count,
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

    Ok((
        turn_digests,
        message_digests,
        tool_digests,
        runtime,
        prompt_preview,
        prompt_file,
        (artifact_count > 0).then(|| path_string(&artifacts_rel_dir)),
        artifact_count,
    ))
}

fn extract_export_reference_paths(text: &str) -> Vec<String> {
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

fn classify_export_reference_status(paths: &[String], current_export_name: &str) -> Option<String> {
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

fn resolve_current_export_paths(paths: &[String], current_export_name: &str) -> Vec<String> {
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

fn build_resolved_prompt_preview(paths: &[String], current_export_name: &str) -> Option<String> {
    let resolved = resolve_current_export_paths(paths, current_export_name);
    if resolved.is_empty() || resolved == paths {
        return None;
    }
    Some(truncate_text(
        &format!("Resolved export refs: {}", resolved.join(", ")),
        SUBTASK_PREVIEW_LIMIT,
    ))
}

fn infer_snapshot_completeness(session_status: &str, staleness_ms: i64) -> String {
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

fn map_child_delegations(
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

fn build_session_hot_turns(turns: &[TurnDigest]) -> Vec<TurnHotspot> {
    let mut items = turns.iter().map(turn_hotspot_from_digest).collect::<Vec<_>>();
    items.sort_by(|left, right| {
        right
            .response_elapsed_ms
            .unwrap_or_default()
            .cmp(&left.response_elapsed_ms.unwrap_or_default())
            .then_with(|| right.total_tokens.cmp(&left.total_tokens))
    });
    items.truncate(SESSION_HOT_MESSAGES_LIMIT);
    items
}

fn build_pivotal_turns(turns: &[TurnDigest]) -> Vec<usize> {
    let mut items = turns
        .iter()
        .enumerate()
        .filter_map(|(idx, turn)| {
            let mut score = 0usize;
            if turn.recommended_attention == "inspect-artifacts" {
                score += 3;
            }
            if turn.delegation_count > 0 {
                score += 2;
            }
            if !turn.change_intents.is_empty() {
                score += 1;
            }
            if idx > 0 && turns[idx - 1].agent_strategy != turn.agent_strategy {
                score += 2;
            }
            (score > 0).then_some((score, turn.turn_index))
        })
        .collect::<Vec<_>>();
    items.sort_by(|left, right| right.0.cmp(&left.0).then_with(|| left.1.cmp(&right.1)));
    items.into_iter().map(|(_, turn_index)| turn_index).take(6).collect()
}

fn build_session_narrative(
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

fn collect_turn_reasoning_themes(messages: &[MessageDigest], range: Option<&std::ops::RangeInclusive<usize>>) -> Vec<String> {
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

fn build_turn_reasoning_summary(
    messages: &[MessageDigest],
    range: Option<&std::ops::RangeInclusive<usize>>,
    themes: &[String],
) -> Option<String> {
    let Some(range) = range else {
        return None;
    };
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

fn build_turn_digests(
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

fn classify_user_intent(text: Option<&str>) -> (String, f64) {
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

fn classify_alternative_user_intents(text: Option<&str>, primary: &str, confidence: f64) -> Vec<String> {
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

fn classify_user_tags(text: Option<&str>) -> Vec<String> {
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

fn infer_turn_outcome(
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

fn is_analysis_tool(tool: &str) -> bool {
    matches!(tool, "read" | "grep" | "glob" | "bash" | "task")
}

fn is_analysis_only_turn_shape(
    modified_file_count: usize,
    tool_rollup: &[TurnToolAggregate],
    final_assistant_text_present: bool,
) -> bool {
    modified_file_count == 0
        && final_assistant_text_present
        && !tool_rollup.is_empty()
        && tool_rollup.iter().all(|entry| is_analysis_tool(&entry.tool))
}

fn infer_turn_success(
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

fn infer_agent_strategy(
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

fn finalize_turn_effectiveness(turns: &mut [TurnDigest]) {
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

fn classify_turn_cost_tier(total_tokens: u64, tool_call_count: usize, response_elapsed_ms: Option<i64>) -> String {
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

fn classify_turn_effectiveness(turn: &TurnDigest, files_survived_to_end: usize) -> String {
    let retry_ratio = turn.effectiveness_signals.retry_ratio.unwrap_or_default();
    let redundant_read_ratio = turn.effectiveness_signals.redundant_read_ratio.unwrap_or_default();
    if !turn.success && files_survived_to_end == 0 && turn.modified_file_count == 0 {
        return String::from("waste");
    }
    if files_survived_to_end > 0 || (turn.outcome == "delegated" && turn.success) {
        return String::from("high-value");
    }
    if turn.success && turn.modified_file_count > 0 {
        if matches!(turn.turn_cost_tier.as_str(), "heavy" | "extreme") || retry_ratio >= 0.5 || redundant_read_ratio >= 0.7 {
            return String::from("moderate");
        }
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

fn recommend_turn_attention(turn: &TurnDigest) -> String {
    match (turn.turn_effectiveness.as_str(), turn.turn_cost_tier.as_str()) {
        ("waste", _) => String::from("skip"),
        ("low-value", "heavy" | "extreme") => String::from("skim"),
        ("high-value", "heavy" | "extreme") => String::from("inspect-artifacts"),
        ("high-value", _) => String::from("read-carefully"),
        _ => String::from("skim"),
    }
}

fn build_failure_narrative(turn: &TurnDigest, files_survived_to_end: usize) -> Option<String> {
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

fn build_optimization_hints(turn: &TurnDigest) -> Vec<String> {
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

fn summarize_turn_change(
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

fn classify_assistant_text_kind(text: Option<&str>) -> Option<String> {
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

fn turn_hotspot_from_digest(turn: &TurnDigest) -> TurnHotspot {
    let mut hot_reasons = Vec::new();
    if turn.response_elapsed_ms.unwrap_or_default() >= HOT_TURN_SLOW_MS_THRESHOLD {
        hot_reasons.push(String::from("slow"));
    }
    if turn.total_tokens >= HOT_TURN_TOKEN_THRESHOLD {
        hot_reasons.push(String::from("token-heavy"));
    }
    if turn.tool_call_count >= HOT_TURN_TOOL_COUNT_THRESHOLD {
        hot_reasons.push(String::from("tool-heavy"));
    }
    if matches!(turn.turn_cost_tier.as_str(), "heavy" | "extreme") {
        hot_reasons.push(String::from("expensive"));
    }
    if turn.delegation_count > 0 {
        hot_reasons.push(String::from("delegation"));
    }
    if turn.error_count > 0 {
        hot_reasons.push(String::from("errors"));
    }
    TurnHotspot {
        session_path: turn.session_path.clone(),
        turn_index: turn.turn_index,
        user_message_index: turn.user_message_index,
        user_intent: turn.user_intent.clone(),
        user_intent_confidence: turn.user_intent_confidence,
        hot_reasons,
        response_elapsed_ms: turn.response_elapsed_ms,
        wall_to_next_user_ms: turn.wall_to_next_user_ms,
        total_tokens: turn.total_tokens,
        tool_calls: turn.tool_call_count,
        error_count: turn.error_count,
        delegation_count: turn.delegation_count,
        cache_hit_ratio: turn.cache_hit_ratio,
        tokens_per_tool_call: turn.tokens_per_tool_call,
        agent_strategy: turn.agent_strategy.clone(),
        outcome: turn.outcome.clone(),
        turn_cost_tier: turn.turn_cost_tier.clone(),
        turn_effectiveness: turn.turn_effectiveness.clone(),
        recommended_attention: turn.recommended_attention.clone(),
        user_text_preview: turn.user_text_preview.clone(),
        final_assistant_kind: turn.final_assistant_kind.clone(),
        final_assistant_text_preview: turn.final_assistant_text_preview.clone(),
    }
}

fn extract_task_id(output: &str) -> Option<String> {
    let prefix = "task_id:";
    output
        .lines()
        .find_map(|line| line.trim().strip_prefix(prefix).map(str::trim))
        .and_then(|value| value.split_whitespace().next())
        .map(str::to_string)
        .filter(|value| !value.is_empty())
}

fn summarize_patch_text(text: &str) -> Option<PatchSummary> {
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

fn render_patch_summary(summary: &PatchSummary) -> String {
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

fn build_key_diff_preview(summary: Option<&PatchSummary>, intent: Option<&str>) -> Option<String> {
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

fn classify_patch_intent(paths: &[String], summary: &PatchSummary) -> Option<String> {
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

fn build_session_hot_messages(messages: &[MessageDigest]) -> Vec<MessageHotspot> {
    let mut items = build_message_hotspots(messages);
    items.sort_by(|left, right| {
        right
            .total_tokens
            .cmp(&left.total_tokens)
            .then_with(|| right.duration_ms.unwrap_or_default().cmp(&left.duration_ms.unwrap_or_default()))
            .then_with(|| right.message_count.cmp(&left.message_count))
    });
    items.truncate(SESSION_HOT_MESSAGES_LIMIT);
    items
}

fn build_hotspots(
    sessions: &[SessionHotspot],
    turns: &[TurnDigest],
    messages: &[MessageDigest],
    tool_calls: &[ToolCallDigest],
) -> ExportHotspots {
    let mut slowest_sessions = sessions.to_vec();
    slowest_sessions.sort_by(|left, right| right.duration_ms.cmp(&left.duration_ms));
    slowest_sessions.truncate(EXPORT_HOTSPOT_LIMIT);

    let mut turn_items = turns.iter().map(turn_hotspot_from_digest).collect::<Vec<_>>();
    turn_items.retain(|item| !item.hot_reasons.is_empty());
    turn_items.sort_by(|left, right| {
        right
            .total_tokens
            .cmp(&left.total_tokens)
            .then_with(|| right.response_elapsed_ms.unwrap_or_default().cmp(&left.response_elapsed_ms.unwrap_or_default()))
            .then_with(|| right.tool_calls.cmp(&left.tool_calls))
            .then_with(|| right.delegation_count.cmp(&left.delegation_count))
    });
    turn_items.truncate(EXPORT_HOTSPOT_LIMIT);

    let mut message_items = build_message_hotspots(messages);
    message_items.sort_by(|left, right| {
        right
            .total_tokens
            .cmp(&left.total_tokens)
            .then_with(|| right.duration_ms.unwrap_or_default().cmp(&left.duration_ms.unwrap_or_default()))
            .then_with(|| right.message_count.cmp(&left.message_count))
            .then_with(|| right.tool_calls.cmp(&left.tool_calls))
    });
    message_items.truncate(EXPORT_HOTSPOT_LIMIT);

    let all_tool_hotspots = tool_calls
        .iter()
        .cloned()
        .map(|tool| {
            let mut hot_reasons = Vec::new();
            if tool.duration_ms.unwrap_or_default() >= HOT_TOOL_SLOW_MS_THRESHOLD {
                hot_reasons.push(String::from("slow"));
            }
            if tool.output_chars.unwrap_or_default() >= HOT_TOOL_OUTPUT_CHARS_THRESHOLD {
                hot_reasons.push(String::from("large-output"));
            }
            if tool.status == "error" {
                hot_reasons.push(String::from("error"));
            }
            ToolHotspot {
                session_path: tool.session_path,
                message_index: tool.message_index,
                tool_index: tool.tool_index,
                tool: tool.tool,
                hot_reasons,
                status: tool.status,
                duration_ms: tool.duration_ms,
                output_chars: tool.output_chars,
                output_preview: tool.output_preview,
                output_file: tool.output_file,
                error_type: tool.error_type,
                error_file: tool.error_file,
            }
        })
        .collect::<Vec<_>>();
    let mut tool_items = all_tool_hotspots;
    tool_items.retain(|item| !item.hot_reasons.is_empty());
    tool_items.sort_by(|left, right| {
        right
            .output_chars
            .unwrap_or_default()
            .cmp(&left.output_chars.unwrap_or_default())
            .then_with(|| right.duration_ms.unwrap_or_default().cmp(&left.duration_ms.unwrap_or_default()))
            .then_with(|| (right.status == "error").cmp(&(left.status == "error")))
    });
    tool_items.truncate(EXPORT_HOTSPOT_LIMIT);

    ExportHotspots {
        slowest_sessions,
        turns: turn_items,
        messages: message_items,
        tools: tool_items,
    }
}

fn build_message_hotspots(messages: &[MessageDigest]) -> Vec<MessageHotspot> {
    let mut items: Vec<MessageHotspot> = Vec::new();
    for message in messages {
        let hotspot = message_hotspot_from_digest(message);
        if hotspot.hot_reasons.is_empty() {
            continue;
        }
        if let Some(previous) = items.last_mut()
            && previous.session_path == hotspot.session_path
            && previous.role == hotspot.role
            && previous.turn_index == hotspot.turn_index
            && previous.hot_reasons == hotspot.hot_reasons
            && previous.end_message_index.unwrap_or(previous.start_message_index) + 1 == hotspot.start_message_index
        {
            previous.end_message_index = Some(hotspot.start_message_index);
            previous.message_count += 1;
            previous.duration_ms = Some(previous.duration_ms.unwrap_or_default() + hotspot.duration_ms.unwrap_or_default());
            previous.total_tokens += hotspot.total_tokens;
            previous.tool_calls += hotspot.tool_calls;
            if previous.sample_text_preview.is_none() {
                previous.sample_text_preview = hotspot.sample_text_preview.clone();
            }
            previous.pattern = message_hotspot_pattern(previous, message.activity_summary.as_deref());
            continue;
        }
        items.push(hotspot);
    }
    items
}

fn message_hotspot_pattern(hotspot: &MessageHotspot, activity_summary: Option<&str>) -> Option<String> {
    if hotspot.message_count <= 1 {
        return None;
    }
    if activity_summary.map(|activity| activity.starts_with("read ")).unwrap_or(false)
        && hotspot.hot_reasons.iter().any(|reason| reason == "token-heavy")
    {
        return Some(String::from("same-file-read-loop"));
    }
    Some(String::from("multi-message-hot-span"))
}

fn message_hotspot_from_digest(message: &MessageDigest) -> MessageHotspot {
    let mut hot_reasons = Vec::new();
    if message.duration_ms.unwrap_or_default() >= HOT_MESSAGE_SLOW_MS_THRESHOLD {
        hot_reasons.push(String::from("slow"));
    }
    if token_total(message.tokens.as_ref()) >= HOT_MESSAGE_TOKEN_THRESHOLD {
        hot_reasons.push(String::from("token-heavy"));
    }
    if message.tool_count >= HOT_MESSAGE_TOOL_COUNT_THRESHOLD {
        hot_reasons.push(String::from("tool-heavy"));
    }
    MessageHotspot {
        session_path: message.session_path.clone(),
        start_message_index: message.message_index,
        end_message_index: None,
        turn_index: message.turn_index,
        message_count: 1,
        role: message.role.clone(),
        hot_reasons,
        duration_ms: message.duration_ms,
        total_tokens: token_total(message.tokens.as_ref()),
        tool_calls: message.tool_count,
        pattern: None,
        sample_text_preview: message
            .text_preview
            .as_deref()
            .map(|text| truncate_text(text, TOOL_PART_PREVIEW_LIMIT)),
    }
}

fn trim_session_hotspot(mut session: SessionHotspot) -> SessionHotspot {
    if session.input_tokens + session.output_tokens + session.reasoning_tokens < HOT_SESSION_TOKEN_THRESHOLD {
        session.input_tokens = 0;
        session.output_tokens = 0;
        session.reasoning_tokens = 0;
    }
    session
}

fn rollup_tools(tools: &[ToolCallDigest]) -> Vec<ToolAggregate> {
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

fn build_file_access_rollup(turns: &[TurnDigest], tools: &[ToolCallDigest]) -> Vec<FileAccessRollupEntry> {
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

fn build_error_patterns(turns: &[TurnDigest], tools: &[ToolCallDigest]) -> Vec<ErrorPatternEntry> {
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

fn build_retry_chains(turns: &[TurnDigest], tools: &[ToolCallDigest]) -> Vec<RetryChainEntry> {
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

fn build_file_transition_rollup(turns: &[TurnDigest], tools: &[ToolCallDigest], session_status: &str) -> Vec<FileTransitionEntry> {
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

fn build_session_deliverables(
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

#[derive(Debug, Clone)]
struct DeliverableSnapshot {
    snapshot_file: String,
    content_sha256: String,
    line_count: usize,
    snapshot_source: String,
}

fn build_deliverable_snapshot(
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

fn resolve_workspace_deliverable_path(path: &str) -> Option<PathBuf> {
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

fn repo_root_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .unwrap_or_else(|| Path::new(env!("CARGO_MANIFEST_DIR")))
        .to_path_buf()
}

fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

fn build_turn_dependency_edges(transitions: &[FileTransitionEntry]) -> Vec<TurnDependencyEdge> {
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

fn build_turn_compact_entries(turns: &[TurnDigest]) -> Vec<TurnCompactEntry> {
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

fn build_message_compact_entries(messages: &[MessageDigest]) -> Vec<MessageCompactEntry> {
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

fn build_token_efficiency(
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

fn build_message_turn_index(turns: &[TurnDigest]) -> HashMap<usize, usize> {
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

fn build_iteration_meta(base_dir: &Path, root_name: &str, export_root: &Path) -> Result<IterationMeta> {
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

fn export_group_sort_key(root_name: &str, name: &str) -> (usize, usize, String) {
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

fn json_number_i64(value: Option<&Value>) -> i64 {
    value
        .and_then(|value| value.as_i64().or_else(|| value.as_u64().and_then(|num| i64::try_from(num).ok())))
        .unwrap_or_default()
}

fn json_number_f64(value: Option<&Value>) -> f64 {
    value.and_then(Value::as_f64).unwrap_or_default()
}

fn build_totals_delta(previous_obj: &Map<String, Value>, current: &ExportIndexFile) -> Option<TotalsDelta> {
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

fn build_tool_rollup_deltas(previous_obj: &Map<String, Value>, current: &ExportIndexFile) -> Vec<ToolRollupDelta> {
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

fn read_jsonl_typed<T: DeserializeOwned>(path: &Path) -> Result<Vec<T>> {
    let text = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    text.lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| serde_json::from_str(line).with_context(|| format!("parse {}", path.display())))
        .collect()
}

fn total_tokens_from_turn_delta(turn: &TurnDeltaDigest) -> u64 {
    turn.input_tokens + turn.output_tokens + turn.reasoning_tokens + turn.cache_read_tokens + turn.cache_write_tokens
}

fn build_turn_deltas(base_dir: &Path, export_root: &Path, previous_obj: &Map<String, Value>, current: &ExportIndexFile) -> Result<Vec<TurnDeltaEntry>> {
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

fn build_delta_from_previous(base_dir: &Path, export_root: &Path, current: &ExportIndexFile) -> Result<Option<DeltaFromPrevious>> {
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

fn average_u64(total: u64, count: usize) -> Option<f64> {
    (count > 0).then_some(total as f64 / count as f64)
}

fn average_usize(total: usize, count: usize) -> Option<f64> {
    (count > 0).then_some(total as f64 / count as f64)
}

fn infer_session_status(
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

fn classify_tool_error(tool: &str, status: &str, error_text: &str) -> Option<String> {
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

fn extract_reasoning_themes(text: &str) -> Vec<String> {
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

fn summarize_reasoning(text: &str, themes: &[String]) -> Option<String> {
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
    if lines.len() > 1 {
        if let Some(last) = lines.iter().rev().find(|line| line.as_str() != lines[0]) {
            parts.push(format!("end: {}", truncate_text(last, 120)));
        }
    }

    Some(truncate_text(&parts.join(" | "), REASONING_SUMMARY_LIMIT))
}

fn classify_message_kind(role: &str, has_text: bool, has_reasoning: bool, has_activity: bool) -> String {
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

fn summarize_activity_items(items: &[String]) -> Option<String> {
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

fn summarize_tool_activity(
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

fn tool_read_paths(tool: &str, input_value: Option<&Value>) -> Vec<String> {
    if tool != "read" {
        return Vec::new();
    }
    input_value
        .and_then(|value| value.get("filePath"))
        .and_then(Value::as_str)
        .map(|path| vec![normalize_tool_path(path)])
        .unwrap_or_default()
}

fn patch_paths_from_text(text: &str) -> Vec<String> {
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

fn patch_path_presence_from_text(text: &str) -> HashMap<String, bool> {
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

fn sample_strings(mut items: Vec<String>, limit: usize) -> Vec<String> {
    items.sort();
    items.truncate(limit);
    items
}

fn normalize_tool_path(path: &str) -> String {
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

fn normalize_tool_input_preview(tool: &str, mut preview: Value) -> Value {
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

fn classify_tool_call_purpose(
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

fn to_json_values<T: Serialize>(items: &[T]) -> Result<Vec<Value>> {
    items
        .iter()
        .map(serde_json::to_value)
        .collect::<Result<Vec<_>, _>>()
        .map_err(|err| anyhow::anyhow!(err.to_string()))
}

fn token_total(tokens: Option<&TokenStatsExport>) -> u64 {
    tokens
        .map(|tokens| tokens.total.unwrap_or(tokens.input + tokens.output + tokens.reasoning + tokens.cache_read + tokens.cache_write))
        .unwrap_or_default()
}

fn join_blocks(blocks: &[&str]) -> String {
    blocks
        .iter()
        .map(|block| block.trim())
        .filter(|block| !block.is_empty())
        .collect::<Vec<_>>()
        .join("\n\n")
}

fn ensure_artifacts_dir(created: &mut bool, dir: &Path) -> Result<()> {
    if !*created {
        fs::create_dir_all(dir).with_context(|| format!("create {}", dir.display()))?;
        *created = true;
    }
    Ok(())
}

fn write_text_artifact(
    artifacts_dir: &Path,
    artifacts_rel_dir: &Path,
    artifacts_created: &mut bool,
    artifact_count: &mut usize,
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
    ensure_artifacts_dir(artifacts_created, artifacts_dir)?;
    let path = artifacts_dir.join(file_name);
    fs::write(&path, text).with_context(|| format!("write {}", path.display()))?;
    *artifact_count += 1;
    Ok(Some(path_string(&artifacts_rel_dir.join(file_name))))
}

fn write_json_artifact(
    artifacts_dir: &Path,
    artifacts_rel_dir: &Path,
    artifacts_created: &mut bool,
    artifact_count: &mut usize,
    file_name: &str,
    value: &Value,
    force: bool,
    threshold: usize,
) -> Result<Option<String>> {
    let text = serde_json::to_string(value).map_err(|err| anyhow::anyhow!(err.to_string()))?;
    if !force && text.len() <= threshold {
        return Ok(None);
    }
    ensure_artifacts_dir(artifacts_created, artifacts_dir)?;
    let path = artifacts_dir.join(file_name);
    fs::write(&path, text).with_context(|| format!("write {}", path.display()))?;
    *artifact_count += 1;
    Ok(Some(path_string(&artifacts_rel_dir.join(file_name))))
}

fn write_artifacts_manifest(
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

fn classify_artifact_manifest_entry(name: &str) -> (String, Option<usize>, Option<usize>) {
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

fn path_string(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

fn compact_part(part: &LoadedPart) -> Option<CompactPart> {
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

fn compute_session_stats(
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

fn session_totals(stats: &SessionStats) -> SessionTotals {
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

fn parent_session_path(session_path: &str) -> Option<String> {
    session_path.rsplit_once('.').map(|(parent, _)| parent.to_string())
}

fn build_export_schema() -> Value {
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

fn build_export_fields_catalog() -> Value {
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

fn render_export_readme(index: &ExportIndexFile) -> String {
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

fn is_zero_usize(value: &usize) -> bool {
    *value == 0
}

fn is_one_usize(value: &usize) -> bool {
    *value == 1
}

fn is_zero_u64(value: &u64) -> bool {
    *value == 0
}

fn is_zero_i64(value: &i64) -> bool {
    *value == 0
}

fn is_zero_f64(value: &f64) -> bool {
    *value == 0.0
}

fn is_completed_status(value: &str) -> bool {
    value == "completed"
}

fn non_empty_owned(value: Option<&str>) -> Option<String> {
    value.map(str::trim).filter(|value| !value.is_empty()).map(str::to_string)
}

fn write_json_pretty(path: PathBuf, value: &impl Serialize) -> Result<()> {
    let file = File::create(&path).with_context(|| format!("create {}", path.display()))?;
    let writer = BufWriter::new(file);
    serde_json::to_writer_pretty(writer, value).with_context(|| format!("write {}", path.display()))?;
    Ok(())
}

fn write_text(path: PathBuf, text: &str) -> Result<()> {
    fs::write(&path, text).with_context(|| format!("write {}", path.display()))?;
    Ok(())
}

fn write_jsonl(path: PathBuf, lines: &[Value]) -> Result<()> {
    let file = File::create(&path).with_context(|| format!("create {}", path.display()))?;
    let mut writer = BufWriter::new(file);
    for line in lines {
        serde_json::to_writer(&mut writer, line).with_context(|| format!("write line to {}", path.display()))?;
        writer.write_all(b"\n")?;
    }
    writer.flush()?;
    Ok(())
}

fn unique_child_dir(base: &Path, name: &str) -> Result<PathBuf> {
    let mut candidate = base.join(name);
    let mut suffix = 2usize;
    while candidate.exists() {
        candidate = base.join(format!("{name}-{suffix}"));
        suffix += 1;
    }
    fs::create_dir_all(&candidate).with_context(|| format!("create {}", candidate.display()))?;
    Ok(candidate)
}

fn default_export_base_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("exports")
}

fn session_folder_name(
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

fn sanitize_filename(input: &str) -> String {
    let mut out = String::new();
    for ch in input.chars().flat_map(char::to_lowercase) {
        if ch.is_ascii_alphanumeric() {
            out.push(ch);
        } else if matches!(ch, ' ' | '-' | '_' | '.') {
            if !out.ends_with('-') {
                out.push('-');
            }
        }
    }
    let trimmed = out.trim_matches('-');
    if trimmed.is_empty() {
        String::from("untitled")
    } else {
        trimmed.chars().take(80).collect()
    }
}

fn extract_subagent_from_title(title: &str) -> Option<String> {
    let start = title.rfind("(@")? + 2;
    let rest = &title[start..];
    let end = rest.find(" subagent)")?;
    let agent = rest[..end].trim();
    (!agent.is_empty()).then(|| agent.to_string())
}

fn short_id(session_id: &str) -> String {
    session_id.chars().take(12).collect()
}

fn truncate_text(text: &str, limit: usize) -> String {
    let text = text.trim();
    let count = text.chars().count();
    if count <= limit {
        return text.to_string();
    }
    let prefix = text.chars().take(limit).collect::<String>();
    format!("{}…(+{} chars)", prefix, count.saturating_sub(limit))
}

fn shrink_json(value: &Value, max_string: usize, max_items: usize, depth: usize) -> Value {
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

fn format_duration(duration_ms: i64) -> String {
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

fn format_local_timestamp(ms: i64) -> String {
    Local
        .timestamp_millis_opt(ms)
        .single()
        .map(|value| value.format("%Y-%m-%d %H:%M:%S").to_string())
        .unwrap_or_else(|| format!("{ms}"))
}

fn format_timestamp_slug(ms: i64) -> String {
    Utc.timestamp_millis_opt(ms)
        .single()
        .map(|value| value.format("%Y%m%d-%H%M%S").to_string())
        .unwrap_or_else(|| String::from("unknown-time"))
}

fn format_system_time(time: std::time::SystemTime) -> String {
    let datetime: DateTime<Local> = time.into();
    datetime.format("%Y-%m-%d %H:%M:%S").to_string()
}

fn format_bytes(bytes: u64) -> String {
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
