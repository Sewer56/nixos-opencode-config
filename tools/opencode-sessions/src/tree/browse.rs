use std::collections::HashSet;

use crate::models::*;

pub(crate) fn build_visible_rows(
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

pub(crate) fn limit_roots(roots: &[String], limit: Option<usize>) -> &[String] {
    match limit {
        Some(limit) => &roots[..roots.len().min(limit)],
        None => roots,
    }
}

pub(crate) fn push_rows_normal(
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

pub(crate) fn push_rows_filtered(
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

pub(crate) fn session_matches_query(session: &SessionOverview, search: &str) -> bool {
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
