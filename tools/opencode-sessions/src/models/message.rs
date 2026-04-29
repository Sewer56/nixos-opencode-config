use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::models::overview::*;
use crate::models::export::*;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub(crate) struct MessageTime {
    #[serde(default)]
    pub(crate) created: Option<i64>,
    #[serde(default)]
    pub(crate) completed: Option<i64>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub(crate) struct ModelRef {
    #[serde(rename = "providerID", default)]
    pub(crate) provider_id: Option<String>,
    #[serde(rename = "modelID", default)]
    pub(crate) model_id: Option<String>,
    #[serde(default)]
    pub(crate) variant: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub(crate) struct CacheTokens {
    #[serde(default)]
    pub(crate) read: u64,
    #[serde(default)]
    pub(crate) write: u64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub(crate) struct Tokens {
    #[serde(default)]
    pub(crate) total: Option<u64>,
    #[serde(default)]
    pub(crate) input: u64,
    #[serde(default)]
    pub(crate) output: u64,
    #[serde(default)]
    pub(crate) reasoning: u64,
    #[serde(default)]
    pub(crate) cache: CacheTokens,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub(crate) struct MessageInfo {
    #[serde(default)]
    pub(crate) role: String,
    #[serde(default)]
    pub(crate) time: MessageTime,
    #[serde(rename = "parentID", default)]
    pub(crate) parent_id: Option<String>,
    #[serde(default)]
    pub(crate) agent: Option<String>,
    #[serde(rename = "modelID", default)]
    pub(crate) model_id: Option<String>,
    #[serde(rename = "providerID", default)]
    pub(crate) provider_id: Option<String>,
    #[serde(default)]
    pub(crate) model: Option<ModelRef>,
    #[serde(default)]
    pub(crate) cost: Option<f64>,
    #[serde(default)]
    pub(crate) tokens: Option<Tokens>,
    #[serde(default)]
    pub(crate) error: Option<Value>,
    #[serde(default)]
    pub(crate) finish: Option<String>,
    #[serde(default)]
    pub(crate) mode: Option<String>,
    #[serde(default)]
    pub(crate) variant: Option<String>,
}

impl MessageInfo {
    pub(crate) fn created_ms(&self, fallback: i64) -> i64 {
        self.time.created.unwrap_or(fallback)
    }

    pub(crate) fn completed_ms(&self) -> Option<i64> {
        self.time.completed
    }

    pub(crate) fn duration_ms(&self) -> Option<i64> {
        self.time
            .created
            .zip(self.time.completed)
            .map(|(start, end)| end.saturating_sub(start))
    }

    pub(crate) fn model_name(&self) -> Option<String> {
        if let Some(model_id) = &self.model_id && !model_id.is_empty() {
            return Some(model_id.clone());
        }
        self.model
            .as_ref()
            .and_then(|model| model.model_id.clone())
            .filter(|value| !value.is_empty())
    }

    pub(crate) fn provider_name(&self) -> Option<String> {
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
pub(crate) struct LoadedPart {
    pub(crate) raw: Value,
}

#[derive(Debug, Clone)]
pub(crate) struct LoadedMessage {
    pub(crate) id: String,
    pub(crate) time_created: i64,
    pub(crate) info: MessageInfo,
    pub(crate) parts: Vec<LoadedPart>,
}

#[derive(Debug, Clone)]
pub(crate) struct LoadedSession {
    pub(crate) meta: SessionOverview,
    pub(crate) messages: Vec<LoadedMessage>,
    pub(crate) children: Vec<LoadedSession>,
}

impl LoadedSession {
    pub(crate) fn agent(&self) -> Option<String> {
        self.meta
            .agent_hint()
            .or_else(|| self.messages.iter().find_map(|message| message.info.agent.clone()))
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type")]
pub(crate) enum CompactPart {
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
pub(crate) struct CompactMessage {
    pub(crate) session_path: String,
    pub(crate) depth: usize,
    pub(crate) message_index: usize,
    pub(crate) message_id: String,
    pub(crate) role: String,
    pub(crate) agent: Option<String>,
    pub(crate) parent_message_id: Option<String>,
    pub(crate) model: Option<String>,
    pub(crate) provider: Option<String>,
    pub(crate) created_ms: i64,
    pub(crate) completed_ms: Option<i64>,
    pub(crate) duration_ms: Option<i64>,
    pub(crate) finish: Option<String>,
    pub(crate) cost: Option<f64>,
    pub(crate) tokens: Option<TokenStatsExport>,
    pub(crate) parts: Vec<CompactPart>,
}
