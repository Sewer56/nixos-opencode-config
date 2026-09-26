//! Loads the v2 session overview (`session_v2`) for the tree, TUI, export,
//! and search.

use crate::cli::*;
use crate::models::*;
use anyhow::{Context, Result, bail};
use rusqlite::Connection;
use std::collections::HashMap;

/// Loads all unarchived sessions with project info and message counts.
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
        from session_v2 s
        left join project p on p.id = s.project_id
        left join (
          select session_id, count(*) as message_count
          from session_message
          where type in ('user', 'assistant')
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
            directory: optional_string(row.get(3)?),
            title: optional_string(row.get(4)?),
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
            children
                .entry(parent_id.clone())
                .or_default()
                .push(session.id.clone());
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

/// Resolves export args (exact id, search text, or latest) to one session id.
pub(crate) fn resolve_target_session_id(
    index: &OverviewIndex,
    args: &ExportArgs,
) -> Result<String> {
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

/// Returns session ids matching a query across id, title, directory, project,
/// and agent fields.
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

/// Reads a v2 column that may be NULL (`title`, `directory`) as empty string.
fn optional_string(value: Option<String>) -> String {
    value.unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::{Connection, params};

    const BASE_MS: i64 = 1771102727767;

    fn test_conn() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            r#"
            create table session_v2 (
              id text primary key,
              project_id text not null,
              parent_id text,
              directory text,
              title text,
              time_created integer not null,
              time_updated integer not null,
              time_archived integer
            );
            create table project (
              id text primary key,
              worktree text,
              name text
            );
            create table session_message (
              id text primary key,
              session_id text not null,
              type text not null,
              seq integer not null,
              time_created integer not null,
              time_updated integer,
              data text not null
            );
            "#,
        )
        .unwrap();
        conn
    }

    fn insert_session(
        conn: &Connection,
        id: &str,
        title: Option<&str>,
        directory: Option<&str>,
        time_updated: i64,
    ) {
        conn.execute(
            "insert into session_v2 (id, project_id, directory, title, time_created, time_updated)
             values (?1, 'proj_1', ?2, ?3, ?4, ?5)",
            params![id, directory, title, BASE_MS, time_updated],
        )
        .unwrap();
    }

    fn insert_row(conn: &Connection, session_id: &str, kind: &str) {
        conn.execute(
            "insert into session_message (id, session_id, type, seq, time_created, data)
             values (?1, ?2, ?3, 0, ?4, '{}')",
            params![format!("msg_{kind}_{session_id}"), session_id, kind, BASE_MS],
        )
        .unwrap();
    }

    // Core behavior: counts, project join, ordering, archived filter.

    #[test]
    fn load_overview_should_count_only_conversation_rows_when_control_rows_exist() {
        let conn = test_conn();
        conn.execute(
            "insert into project (id, worktree, name) values ('proj_1', '/wt', 'my-project')",
            [],
        )
        .unwrap();
        insert_session(&conn, "ses_old", Some("older"), Some("/repo"), BASE_MS);
        insert_session(&conn, "ses_new", Some("newest"), Some("/repo"), BASE_MS + 1000);
        insert_session(&conn, "ses_archived", Some("hidden"), Some("/repo"), BASE_MS + 2000);
        conn.execute(
            "update session_v2 set time_archived = ?1 where id = 'ses_archived'",
            params![BASE_MS + 3000],
        )
        .unwrap();

        // Arrange: conversation rows plus control rows that must not count.
        for kind in ["user", "assistant", "idle", "compaction"] {
            insert_row(&conn, "ses_new", kind);
        }
        insert_row(&conn, "ses_old", "user");

        // Act.
        let index = load_overview(&conn).unwrap();

        // Assert.
        assert_eq!(index.ordered_ids, vec!["ses_new", "ses_old"]);
        let newest = index.sessions.get("ses_new").unwrap();
        assert_eq!(newest.message_count, 2);
        assert_eq!(newest.project_name.as_deref(), Some("my-project"));
        assert_eq!(newest.project_worktree.as_deref(), Some("/wt"));
    }

    // Edge case: v2 title and directory are nullable.

    #[test]
    fn load_overview_should_default_title_and_directory_when_null() {
        let conn = test_conn();
        insert_session(&conn, "ses_null", None, None, BASE_MS);

        let index = load_overview(&conn).unwrap();

        let session = index.sessions.get("ses_null").unwrap();
        assert_eq!(session.title, "");
        assert_eq!(session.directory, "");
    }
}
