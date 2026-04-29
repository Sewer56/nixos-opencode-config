#![recursion_limit = "512"]

mod constants;
mod cli;
mod format;
mod models;
mod db;
mod tui;
mod tree;
mod export;

use anyhow::Result;
use clap::Parser;
use cli::{Cli, Command, TuiArgs, TreeArgs};
use db::discover::{resolve_db_path, open_db, print_discovered_dbs};
use db::overview::{load_overview, resolve_target_session_id};
use tree::display::run_tree_command;
use export::bundle::export_bundle;
use tui::input::run_tui;
use std::io::{self, IsTerminal};

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
