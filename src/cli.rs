use crate::{branch::Branch, git_commands, template::Template};
use anyhow;
use clap::{Args, Parser, Subcommand};
use rusqlite::Connection;

#[derive(Debug, Parser)]
#[clap(name = "git-kit")]
#[clap(bin_name = "git-kit")]
#[clap(about = "git cli containing templates & utilities.", long_about = None)]
pub struct Cli {
    #[clap(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Commit staged changes via git with a template message.
    #[clap(subcommand)]
    Commit(Template),
    /// Checkout an existing branch or create a new branch and add a ticket number as context for future commits.
    Checkout(Checkout),
}

#[derive(Debug, Args)]
pub struct Checkout {
    /// Name of the branch to checkout or create.
    #[clap(value_parser)]
    pub name: String,

    /// Issue ticket number related to the branch.
    #[clap(short, long, value_parser)]
    pub ticket: Option<String>,
}

impl Checkout {
    pub fn checkout(&self, conn: &Connection) -> anyhow::Result<()> {
        // We want to store the branch name against and ticket number
        // So whenever we commit we get the ticket number from the branch
        let branch = Branch::new(&self.name, self.ticket.clone())?;
        branch.insert_or_update(&conn)?;

        // Attempt to create branch
        let create = git_commands::checkout(&self.name, true).output()?;

        // If the branch exists check it out
        if !create.status.success() {
            git_commands::checkout(&self.name, false).status()?;
        }

        Ok(())
    }
}
