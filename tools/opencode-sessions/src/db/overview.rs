use anyhow::{Context, Result, bail};
use rusqlite::Connection;
use std::collections::HashMap;

use crate::cli::*;
use crate::models::*;

pub(crate) fn load_overview(conn: &Connection) -> Result<OverviewIndex> {
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

pub(crate) fn resolve_target_session_id(index: &OverviewIndex, args: &ExportArgs) -> Result<String> {
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

pub(crate) fn search_session_ids(index: &OverviewIndex, query: &str) -> Vec<String> {
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
