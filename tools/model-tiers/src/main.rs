mod cli;
mod config;
mod env;
mod models;
mod rewrite;
mod sort;
mod tui;
mod types;

use anyhow::Result;

fn main() -> Result<()> {
    let env = env::find_env()?;
    let args: Vec<String> = std::env::args().skip(1).collect();
    cli::run_cli(&env, &args)
}
