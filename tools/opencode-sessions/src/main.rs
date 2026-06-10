#![recursion_limit = "512"]

mod constants;
mod cli;
mod format;
mod models;
mod db;
mod tui;
mod tree;
mod export;

use anyhow::{Result, bail};
use clap::Parser;
use cli::{Cli, Command, TuiArgs, TreeArgs};
use db::discover::{resolve_db_path, open_db, print_discovered_dbs};
use db::overview::{load_overview, resolve_target_session_id};
use tree::display::run_tree_command;
use export::bundle::export_bundle;
use tui::input::run_tui;
use std::io::{self, IsTerminal};
use chrono::{Utc, TimeZone, NaiveDate, NaiveDateTime};

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

            if args.all {
                let roots = index.roots.clone();
                eprintln!("Exporting {} root sessions...", roots.len());
                for (i, root_id) in roots.iter().enumerate() {
                    eprintln!("[{}/{}] {}", i + 1, roots.len(), root_id);
                    match export_bundle(&conn, &index, root_id, args.out.clone()) {
                        Ok(export_root) => println!("{}", export_root.display()),
                        Err(e) => eprintln!("  ERROR: {e:#}"),
                    }
                }
                Ok(())
            } else if let Some(since) = &args.since {
                let since_ms = parse_since_timestamp(since)?;
                let matched: Vec<_> = index
                    .roots
                    .iter()
                    .filter(|root_id| {
                        index.sessions.get(*root_id).is_some_and(|s| s.time_updated >= since_ms)
                    })
                    .cloned()
                    .collect();
                if matched.is_empty() {
                    bail!("no root sessions updated since {}", since);
                }
                eprintln!("Exporting {} root sessions updated >= {}...", matched.len(), since);
                for (i, root_id) in matched.iter().enumerate() {
                    eprintln!("[{}/{}] {}", i + 1, matched.len(), root_id);
                    match export_bundle(&conn, &index, root_id, args.out.clone()) {
                        Ok(export_root) => println!("{}", export_root.display()),
                        Err(e) => eprintln!("  ERROR: {e:#}"),
                    }
                }
                Ok(())
            } else {
                let session_id = resolve_target_session_id(&index, &args)?;
                let export_root = export_bundle(&conn, &index, &session_id, args.out)?;
                println!("{}", export_root.display());
                Ok(())
            }
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

fn parse_since_timestamp(raw: &str) -> Result<i64> {
    let trimmed = raw.trim();

    // Try ISO8601 / RFC3339 with timezone
    if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(trimmed) {
        return Ok(dt.timestamp_millis());
    }

    // Try "YYYY-MM-DD HH:MM:SS" (naive → UTC)
    if let Ok(naive) = NaiveDateTime::parse_from_str(trimmed, "%Y-%m-%d %H:%M:%S") {
        return Ok(Utc.from_utc_datetime(&naive).timestamp_millis());
    }

    // Try "YYYY-MM-DD" (naive date → UTC)
    if let Ok(naive) = NaiveDate::parse_from_str(trimmed, "%Y-%m-%d") {
        let dt = naive.and_hms_opt(0, 0, 0).unwrap();
        return Ok(Utc.from_utc_datetime(&dt).timestamp_millis());
    }

    bail!(
        "cannot parse --since value {raw:?}. Expected ISO8601, \"YYYY-MM-DD\", or \"YYYY-MM-DD HH:MM:SS\" (UTC)"
    )
}
