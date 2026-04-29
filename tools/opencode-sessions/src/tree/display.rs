use anyhow::Result;
use serde_json::json;
use std::path::Path;

use crate::cli::*;
use crate::format::*;
use crate::models::*;
use crate::tree::browse::*;

pub(crate) fn run_tree_command(db_path: &Path, index: &OverviewIndex, args: TreeArgs) -> Result<()> {
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

pub(crate) fn build_tree_nodes(index: &OverviewIndex, search: &str, limit: Option<usize>) -> Vec<TreeNode> {
    limit_roots(&index.roots, limit)
        .iter()
        .filter_map(|root_id| build_tree_node(index, root_id, search))
        .collect()
}

pub(crate) fn build_tree_node(index: &OverviewIndex, session_id: &str, search: &str) -> Option<TreeNode> {
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

pub(crate) fn build_text_tree(index: &OverviewIndex, search: &str, limit: Option<usize>) -> Vec<String> {
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

pub(crate) fn subtree_matches(index: &OverviewIndex, session_id: &str, search: &str) -> bool {
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

pub(crate) fn push_text_tree_lines(
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
