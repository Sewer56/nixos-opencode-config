use anyhow::{Context, Result, bail};
use serde::Serialize;
use std::collections::{HashMap, HashSet};

use crate::format::*;

#[derive(Debug, Clone)]
pub(crate) struct SessionOverview {
    pub(crate) id: String,
    pub(crate) project_id: String,
    pub(crate) project_name: Option<String>,
    pub(crate) project_worktree: Option<String>,
    pub(crate) parent_id: Option<String>,
    pub(crate) directory: String,
    pub(crate) title: String,
    pub(crate) time_created: i64,
    pub(crate) time_updated: i64,
    pub(crate) message_count: usize,
}

impl SessionOverview {
    pub(crate) fn duration_ms(&self) -> i64 {
        self.time_updated.saturating_sub(self.time_created)
    }

    pub(crate) fn agent_hint(&self) -> Option<String> {
        extract_subagent_from_title(&self.title)
    }
}

#[derive(Debug, Clone)]
pub(crate) struct OverviewIndex {
    pub(crate) ordered_ids: Vec<String>,
    pub(crate) roots: Vec<String>,
    pub(crate) sessions: HashMap<String, SessionOverview>,
    pub(crate) children: HashMap<String, Vec<String>>,
}

impl OverviewIndex {
    pub(crate) fn get(&self, session_id: &str) -> Result<&SessionOverview> {
        self.sessions
            .get(session_id)
            .with_context(|| format!("session not found in overview: {session_id}"))
    }

    pub(crate) fn children_of(&self, session_id: &str) -> &[String] {
        self.children
            .get(session_id)
            .map(Vec::as_slice)
            .unwrap_or(&[])
    }

    pub(crate) fn latest_root(&self) -> Result<&str> {
        self.roots
            .first()
            .map(String::as_str)
            .context("no root sessions found")
    }

    pub(crate) fn all_expandable_ids(&self) -> HashSet<String> {
        self.children.keys().cloned().collect()
    }

    pub(crate) fn root_id(&self, session_id: &str) -> Result<String> {
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

#[derive(Debug, Clone, Serialize)]
pub(crate) struct TreeNode {
    pub(crate) session_id: String,
    pub(crate) parent_session_id: Option<String>,
    pub(crate) title: String,
    pub(crate) agent: Option<String>,
    pub(crate) directory: String,
    pub(crate) project_name: Option<String>,
    pub(crate) project_worktree: Option<String>,
    pub(crate) created_ms: i64,
    pub(crate) updated_ms: i64,
    pub(crate) duration_ms: i64,
    pub(crate) message_count: usize,
    pub(crate) child_count: usize,
    pub(crate) children: Vec<TreeNode>,
}

#[derive(Debug, Clone)]
pub(crate) struct VisibleRow {
    pub(crate) session_id: String,
    pub(crate) depth: usize,
}
