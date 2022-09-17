pub mod args;
pub mod branch;
pub mod cli;
pub mod git_commands;
pub mod template;
pub mod try_convert;

use anyhow::{anyhow, Context};
use clap::Parser;
use directories::ProjectDirs;
use rusqlite::Connection;

fn main() -> anyhow::Result<()> {
    let args = cli::Cli::parse();

    let project_dir = ProjectDirs::from("dev", "xsv24", "git-kit")
        .context("Failed to retrieve 'git-kit' config")?;

    let conn = Connection::open(project_dir.config_dir().join("db"))?;

    // Could move into build script ?
    conn.execute(
        "CREATE TABLE IF NOT EXISTS branch (
            name TEXT NOT NULL PRIMARY KEY,
            ticket TEXT,
            data BLOB,
            created TEXT NOT NULL
        )",
        (),
    )?;

    // TODO: Add Context<> type to avoid passing in props everywhere.

    match args.command {
        cli::Command::Commit(template) => template.commit(&conn, project_dir),
        cli::Command::Checkout(checkout) => checkout.checkout(&conn),
    }?;

    conn.close()
        .map_err(|_| anyhow!("Failed to close 'git-kit' connection"))?;

    Ok(())
}
