pub mod adapters;
pub mod app_context;
pub mod cli;
pub mod config;
pub mod domain;
pub mod utils;

use cli::{checkout, commit, context, templates};
use domain::commands::CommandActions;

use adapters::{sqlite::Sqlite, Git};
use anyhow::{Context, Ok};
use app_context::AppContext;
use clap::{Parser, Subcommand};
use directories::ProjectDirs;
use rusqlite::Connection;

use crate::config::Config;

#[derive(Debug, Clone, Parser)]
#[clap(name = "git-kit")]
#[clap(bin_name = "git-kit")]
#[clap(about = "git cli containing templates & utilities.", long_about = None)]
#[clap(version)]
pub struct Cli {
    #[clap(subcommand)]
    commands: Commands,
}

#[derive(Debug, Clone, Subcommand)]
pub enum Commands {
    /// Commit staged changes via git with a template message.
    Commit(commit::Arguments),
    /// Checkout an existing branch or create a new branch and add a ticket number as context for future commits.
    Checkout(checkout::Arguments),
    /// Add or update the ticket number related to the current branch.
    Context(context::Arguments),
    // Display a list of configured templates.
    Templates,
}

fn init() -> anyhow::Result<(ProjectDirs, Sqlite, Git)> {
    let project_dir = ProjectDirs::from("dev", "xsv24", "git-kit")
        .context("Failed to retrieve 'git-kit' config")?;

    let connection = Connection::open(project_dir.config_dir().join("db"))
        .context("Failed to open sqlite connection")?;

    let store = Sqlite::new(connection).context("Failed to initialize 'Sqlite'")?;

    Ok((project_dir, store, Git))
}

fn main() -> anyhow::Result<()> {
    let (project_dir, store, git) = init()?;

    let config = Config::new::<Git>(project_dir.config_dir(), &git)?;

    let cli = Cli::parse();

    let context = AppContext::new(git, store, project_dir, config.clone())?;
    let actions = CommandActions::new(&context);

    let result = match cli.commands {
        Commands::Checkout(args) => checkout::handler(&actions, args),
        Commands::Context(args) => context::handler(&actions, args),
        Commands::Commit(args) => commit::handler(&actions, &config, args),
        Commands::Templates => templates::handler(&config),
    };

    // close the connection no matter if we error or not.
    context.close()?;

    Ok(result?)
}

#[test]
fn verify_app() {
    // Simple test to assure cli builds correctly
    use clap::CommandFactory;
    Cli::command().debug_assert()
}
