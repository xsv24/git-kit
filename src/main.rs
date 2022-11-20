pub mod adapters;
pub mod app_context;
pub mod cli;
pub mod config;
pub mod domain;
pub mod utils;

use std::fmt::Debug;

use cli::{checkout, commit, context, templates};
use domain::commands::{CommandActions, GitCommands};

use adapters::{sqlite::Sqlite, Git};
use anyhow::{Context, Ok};
use app_context::AppContext;
use clap::{clap_derive::ArgEnum, Args, Parser, Subcommand};
use directories::ProjectDirs;
use log::LevelFilter;
use rusqlite::Connection;

use crate::config::Config;

#[derive(Clone, Debug, ArgEnum, Default)]
pub enum LogLevel {
    #[default]
    None,
    Debug,
    Info,
    Warn,
    Error,
}

#[derive(Debug, Parser)]
#[clap(name = "git-kit")]
#[clap(bin_name = "git-kit")]
#[clap(about = "git cli containing templates & utilities.", long_about = None)]
#[clap(version)]
pub struct Cli {
    /// File path to your 'git-kit' config file
    #[clap(short, long)]
    config: Option<String>,

    /// Log level
    #[clap(arg_enum, long, default_value_t=LogLevel::None)]
    log: LogLevel,

    /// Commands
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
    /// Display a list of configured templates.
    Templates,
}

impl Cli {
    fn init(&self) -> anyhow::Result<AppContext<Git, Sqlite>> {
        log::set_max_level(log::LevelFilter::Debug);

        let project_dir = ProjectDirs::from("dev", "xsv24", "git-kit")
            .context("Failed to retrieve 'git-kit' config")?;

        let connection = Connection::open(project_dir.config_dir().join("db"))
            .context("Failed to open sqlite connection")?;

        let store = Sqlite::new(connection).context("Failed to initialize 'Sqlite'")?;

        let git = Git;

        // use custom user config if provided or default.
        let config = Config::new(
            self.config.clone(),
            git.root_directory()?,
            project_dir.config_dir(),
        )?;

        let context = AppContext::new(git, store, config)?;

        Ok(context)
    }
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let log = match cli.log {
        LogLevel::Debug => LevelFilter::Debug,
        LogLevel::Info => LevelFilter::Info,
        LogLevel::Warn => LevelFilter::Warn,
        LogLevel::Error => LevelFilter::Error,
        LogLevel::None => LevelFilter::Off,
    };

    env_logger::Builder::new().filter_level(log).init();

    let context = cli.init()?;
    let actions = CommandActions::new(&context);

    let result = match cli.commands {
        Commands::Checkout(args) => checkout::handler(&actions, args),
        Commands::Context(args) => context::handler(&actions, args),
        Commands::Commit(args) => commit::handler(&actions, &context.config, args),
        Commands::Templates => templates::handler(&context.config),
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
