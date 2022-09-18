use crate::{branch::Branch, git_commands::{GitCommands, CheckoutStatus}, template::Template, context::Context};
use anyhow;
use clap::{Args, Parser, Subcommand};

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
    pub fn checkout<C: GitCommands>(&self, context: &Context<C>) -> anyhow::Result<()> {
        // We want to store the branch name against and ticket number
        // So whenever we commit we get the ticket number from the branch
        let repo_name = context.commands.get_repo_name()?;
        let branch = Branch::new(&self.name, &repo_name, self.ticket.clone())?;
        branch.insert_or_update(&context.connection)?;

        // Attempt to create branch
        let create = context.commands.checkout(&self.name, CheckoutStatus::New);

        // If the branch exists check it out
        if !create.is_err() {
            context.commands.checkout(&self.name, CheckoutStatus::Existing)?;
        }

        Ok(())
    }
}
