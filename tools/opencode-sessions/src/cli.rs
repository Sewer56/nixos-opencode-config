use clap::{Args, Parser, Subcommand};
use std::path::PathBuf;


#[derive(Parser)]
#[command(name = "opencode-sessions")]
#[command(version)]
#[command(about = "Browse and export OpenCode conversations from local SQLite")]
pub(crate) struct Cli {
    #[arg(long, global = true, value_name = "PATH")]
    pub(crate) db: Option<PathBuf>,

    #[command(subcommand)]
    pub(crate) command: Option<Command>,
}

#[derive(Subcommand)]
pub(crate) enum Command {
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
pub(crate) struct TuiArgs {
    #[arg(long)]
    pub(crate) search: Option<String>,

    #[arg(long)]
    pub(crate) limit: Option<usize>,
}

#[derive(Args, Clone)]
pub(crate) struct TreeArgs {
    #[arg(long)]
    pub(crate) search: Option<String>,

    #[arg(long)]
    pub(crate) limit: Option<usize>,

    #[arg(long)]
    pub(crate) json: bool,
}

#[derive(Args, Clone)]
pub(crate) struct ExportArgs {
    /// Session id, or search text if exact id not found
    pub(crate) target: Option<String>,

    #[arg(long)]
    pub(crate) search: Option<String>,

    /// Base output dir. Tool creates one bundle folder inside.
    #[arg(long, value_name = "DIR")]
    pub(crate) out: Option<PathBuf>,

    #[arg(long)]
    pub(crate) latest: bool,

    /// Export all root sessions (ignores target/search/latest)
    #[arg(long, conflicts_with_all = ["target", "search", "latest", "since"])]
    pub(crate) all: bool,

    /// Export root sessions updated since this timestamp (ignores target/search/latest)
    /// Accepts ISO8601, "YYYY-MM-DD", or "YYYY-MM-DD HH:MM:SS" (UTC)
    #[arg(long, conflicts_with_all = ["target", "search", "latest", "all"])]
    pub(crate) since: Option<String>,
}
